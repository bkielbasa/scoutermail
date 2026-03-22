use serde::Deserialize;

use crate::store::db::{Database, Message};

#[derive(Debug, Deserialize)]
pub struct Condition {
    pub field: String,
    pub op: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub action: Option<String>, // for ai_prompt: what to do with result
}

/// Check if a message matches all conditions of a rule
pub fn matches_conditions(msg: &Message, conditions: &[Condition]) -> bool {
    conditions.iter().all(|cond| {
        let field_value = match cond.field.as_str() {
            "from" => msg.from_addr.as_deref().unwrap_or(""),
            "to" => msg.to_addr.as_deref().unwrap_or(""),
            "subject" => msg.subject.as_deref().unwrap_or(""),
            "body" => msg.body_text.as_deref().unwrap_or(""),
            "has_attachment" => return cond.value == "true", // simplified
            "has_calendar" => return false, // would need event data
            _ => return false,
        };

        match cond.op.as_str() {
            "contains" => field_value.to_lowercase().contains(&cond.value.to_lowercase()),
            "not_contains" => !field_value.to_lowercase().contains(&cond.value.to_lowercase()),
            "equals" => field_value.eq_ignore_ascii_case(&cond.value),
            "not_equals" => !field_value.eq_ignore_ascii_case(&cond.value),
            "regex" => {
                regex::Regex::new(&cond.value)
                    .map(|re| re.is_match(field_value))
                    .unwrap_or(false)
            }
            _ => false,
        }
    })
}

/// Execute built-in actions on a message. Returns list of action descriptions for logging.
pub fn execute_builtin_actions(
    db: &Database,
    msg: &Message,
    actions: &[Action],
) -> Vec<String> {
    let mut log = Vec::new();

    for action in actions {
        match action.action_type.as_str() {
            "add_label" => {
                if let Some(label_name) = &action.value {
                    // Find or create label
                    let label_id = db.create_label(label_name, "").ok()
                        .or_else(|| {
                            db.get_labels().ok()
                                .and_then(|labels| labels.iter().find(|l| l.name == *label_name).map(|l| l.label_id))
                        });
                    if let Some(lid) = label_id {
                        if let Err(e) = db.add_label_to_message(msg.uid, &msg.folder, lid) {
                            log::error!("failed to add label '{}' to uid={}: {}", label_name, msg.uid, e);
                        } else {
                            log.push(format!("Added label '{}'", label_name));
                        }
                    }
                }
            }
            "remove_label" => {
                if let Some(label_name) = &action.value {
                    if let Ok(labels) = db.get_labels() {
                        if let Some(label) = labels.iter().find(|l| l.name == *label_name) {
                            if let Err(e) = db.remove_label_from_message(msg.uid, &msg.folder, label.label_id) {
                                log::error!("failed to remove label '{}' from uid={}: {}", label_name, msg.uid, e);
                            } else {
                                log.push(format!("Removed label '{}'", label_name));
                            }
                        }
                    }
                }
            }
            "mark_read" => {
                let flags = msg.flags.as_deref().unwrap_or("");
                if !flags.contains("Seen") {
                    let new_flags = format!("{} Seen", flags).trim().to_string();
                    if let Err(e) = db.update_flags(msg.uid, &msg.folder, &new_flags) {
                        log::error!("failed to mark uid={} as read: {}", msg.uid, e);
                    } else {
                        log.push("Marked as read".into());
                    }
                }
            }
            "mark_unread" => {
                let flags = msg.flags.as_deref().unwrap_or("");
                let new_flags = flags.replace("Seen", "").trim().to_string();
                if let Err(e) = db.update_flags(msg.uid, &msg.folder, &new_flags) {
                    log::error!("failed to mark uid={} as unread: {}", msg.uid, e);
                } else {
                    log.push("Marked as unread".into());
                }
            }
            "star" => {
                let flags = msg.flags.as_deref().unwrap_or("");
                if !flags.contains("Flagged") {
                    let new_flags = format!("{} Flagged", flags).trim().to_string();
                    if let Err(e) = db.update_flags(msg.uid, &msg.folder, &new_flags) {
                        log::error!("failed to star uid={}: {}", msg.uid, e);
                    } else {
                        log.push("Starred".into());
                    }
                }
            }
            "delete" => {
                if let Err(e) = db.delete_message(msg.uid, &msg.folder) {
                    log::error!("failed to delete uid={}: {}", msg.uid, e);
                } else {
                    log.push("Deleted".into());
                }
            }
            // webhook, shell, ai_prompt are handled separately (async)
            _ => {}
        }
    }
    log
}

/// Execute extended actions (webhook, shell). Call from async context.
pub async fn execute_extended_actions(
    msg: &Message,
    actions: &[Action],
) -> Vec<String> {
    let mut log = Vec::new();

    for action in actions {
        match action.action_type.as_str() {
            "webhook" => {
                if let Some(url) = &action.url {
                    let method = action.method.as_deref().unwrap_or("POST");
                    let payload = serde_json::json!({
                        "from": msg.from_addr,
                        "to": msg.to_addr,
                        "subject": msg.subject,
                        "body_snippet": msg.body_text.as_deref().unwrap_or("").chars().take(500).collect::<String>(),
                        "date": msg.date,
                        "message_id": msg.message_id,
                    });

                    let client = reqwest::Client::new();
                    let result = if method.eq_ignore_ascii_case("GET") {
                        client.get(url).send().await
                    } else {
                        client.post(url).json(&payload).send().await
                    };

                    match result {
                        Ok(resp) => log.push(format!("Webhook {} -> {}", url, resp.status())),
                        Err(e) => log.push(format!("Webhook {} failed: {}", url, e)),
                    }
                }
            }
            "shell" => {
                if let Some(cmd) = &action.command {
                    let output = tokio::process::Command::new("sh")
                        .arg("-c")
                        .arg(cmd)
                        .env("MAIL_FROM", msg.from_addr.as_deref().unwrap_or(""))
                        .env("MAIL_TO", msg.to_addr.as_deref().unwrap_or(""))
                        .env("MAIL_SUBJECT", msg.subject.as_deref().unwrap_or(""))
                        .env("MAIL_BODY", msg.body_text.as_deref().unwrap_or(""))
                        .env("MAIL_DATE", msg.date.as_deref().unwrap_or(""))
                        .env("MAIL_ID", msg.message_id.as_deref().unwrap_or(""))
                        .output()
                        .await;

                    match output {
                        Ok(o) => {
                            let status = o.status;
                            let stdout = String::from_utf8_lossy(&o.stdout);
                            log.push(format!("Shell '{}' -> exit {} ({})", cmd, status, stdout.trim()));
                        }
                        Err(e) => log.push(format!("Shell '{}' failed: {}", cmd, e)),
                    }
                }
            }
            "ai_prompt" => {
                // AI processing requires an API key configured via :set ai_api_key
                // and :set ai_api_url (defaults to Anthropic)
                // For now, log that it would execute
                if let Some(prompt) = &action.prompt {
                    log.push(format!("AI prompt (not yet configured): {}", prompt));
                }
            }
            _ => {}
        }
    }
    log
}

/// Run all enabled rules against a set of messages. Returns log of actions taken.
pub fn run_rules_on_messages(db: &Database, messages: &[Message]) -> Vec<String> {
    let rules = match db.get_enabled_rules() {
        Ok(r) => r,
        Err(e) => return vec![format!("Failed to load rules: {}", e)],
    };

    if rules.is_empty() {
        return Vec::new();
    }

    let mut all_logs = Vec::new();

    for msg in messages {
        for rule in &rules {
            let conditions: Vec<Condition> = match serde_json::from_str(&rule.conditions) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let actions: Vec<Action> = match serde_json::from_str(&rule.actions) {
                Ok(a) => a,
                Err(_) => continue,
            };

            if matches_conditions(msg, &conditions) {
                let subject = msg.subject.as_deref().unwrap_or("(no subject)");
                all_logs.push(format!(
                    "Rule '{}' matched message uid={} subject='{}'",
                    rule.name, msg.uid, subject
                ));

                // Execute built-in (sync) actions
                let builtin_logs = execute_builtin_actions(db, msg, &actions);
                all_logs.extend(builtin_logs);

                // Note: extended actions (webhook, shell, ai_prompt) require async context
                // They are handled separately when called from async code
            }
        }
    }

    all_logs
}

/// Async version that also handles webhook/shell/ai_prompt actions.
pub async fn run_rules_on_messages_async(db: &Database, messages: &[Message]) -> Vec<String> {
    let rules = match db.get_enabled_rules() {
        Ok(r) => r,
        Err(e) => return vec![format!("Failed to load rules: {}", e)],
    };

    if rules.is_empty() {
        return Vec::new();
    }

    let mut all_logs = Vec::new();

    for msg in messages {
        for rule in &rules {
            let conditions: Vec<Condition> = match serde_json::from_str(&rule.conditions) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let actions: Vec<Action> = match serde_json::from_str(&rule.actions) {
                Ok(a) => a,
                Err(_) => continue,
            };

            if matches_conditions(msg, &conditions) {
                let subject = msg.subject.as_deref().unwrap_or("(no subject)");
                all_logs.push(format!(
                    "Rule '{}' matched message uid={} subject='{}'",
                    rule.name, msg.uid, subject
                ));

                // Execute built-in (sync) actions
                let builtin_logs = execute_builtin_actions(db, msg, &actions);
                all_logs.extend(builtin_logs);

                // Execute extended (async) actions
                let extended_logs = execute_extended_actions(msg, &actions).await;
                all_logs.extend(extended_logs);
            }
        }
    }

    all_logs
}
