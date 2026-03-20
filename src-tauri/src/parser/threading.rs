use std::collections::HashMap;

use crate::store::db::Message;

// ---------------------------------------------------------------------------
// Subject normalization
// ---------------------------------------------------------------------------

/// Strip Re:, Fwd:, Fw:, Re[N]:, Fwd[N]: prefixes and lowercase.
pub fn normalize_subject(subject: &str) -> String {
    let mut s = subject.trim().to_string();

    // Repeatedly strip known prefixes
    loop {
        let trimmed = s.trim_start().to_string();
        // Try matching: Re:, Fwd:, Fw:, Re[N]:, Fwd[N]:, Fw[N]:
        if let Some(rest) = strip_prefix_ci(&trimmed, "re:") {
            s = rest;
        } else if let Some(rest) = strip_prefix_ci(&trimmed, "fwd:") {
            s = rest;
        } else if let Some(rest) = strip_prefix_ci(&trimmed, "fw:") {
            s = rest;
        } else if let Some(rest) = strip_bracketed_prefix_ci(&trimmed, "re") {
            s = rest;
        } else if let Some(rest) = strip_bracketed_prefix_ci(&trimmed, "fwd") {
            s = rest;
        } else if let Some(rest) = strip_bracketed_prefix_ci(&trimmed, "fw") {
            s = rest;
        } else {
            break;
        }
    }

    s.trim().to_lowercase()
}

/// Case-insensitive prefix strip for simple prefixes like "re:" or "fwd:".
fn strip_prefix_ci(s: &str, prefix: &str) -> Option<String> {
    if s.len() >= prefix.len()
        && s[..prefix.len()].eq_ignore_ascii_case(prefix)
    {
        Some(s[prefix.len()..].to_string())
    } else {
        None
    }
}

/// Strip prefixes like "Re[2]:" or "Fwd[3]:" — case insensitive.
fn strip_bracketed_prefix_ci(s: &str, keyword: &str) -> Option<String> {
    let klen = keyword.len();
    if s.len() <= klen {
        return None;
    }
    if !s[..klen].eq_ignore_ascii_case(keyword) {
        return None;
    }
    let rest = &s[klen..];
    if !rest.starts_with('[') {
        return None;
    }
    if let Some(bracket_end) = rest.find(']') {
        // Verify digits inside brackets
        let inside = &rest[1..bracket_end];
        if inside.chars().all(|c| c.is_ascii_digit()) && !inside.is_empty() {
            let after_bracket = &rest[bracket_end + 1..];
            if let Some(stripped) = after_bracket.strip_prefix(':') {
                return Some(stripped.to_string());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Thread assignment
// ---------------------------------------------------------------------------

/// Parse a space-separated ref_headers string into individual message-IDs.
fn parse_refs(refs: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut current = String::new();
    let mut inside = false;

    for ch in refs.chars() {
        if ch == '<' {
            inside = true;
            current.clear();
            current.push(ch);
        } else if ch == '>' {
            current.push(ch);
            if inside {
                ids.push(current.clone());
            }
            inside = false;
        } else if inside {
            current.push(ch);
        }
    }
    ids
}

/// Assign thread_id to each message based on References/In-Reply-To headers,
/// with a fallback to normalized subject grouping.
pub fn assign_threads(messages: &mut Vec<Message>) {
    // Map from message_id -> thread_id
    let mut mid_to_thread: HashMap<String, String> = HashMap::new();
    // Map from normalized subject -> thread_id (fallback)
    let mut subject_to_thread: HashMap<String, String> = HashMap::new();

    for msg in messages.iter_mut() {
        let mut assigned_thread: Option<String> = None;

        // First pass: check References and In-Reply-To for existing thread membership
        let ref_ids = msg
            .ref_headers
            .as_deref()
            .map(|r| parse_refs(r))
            .unwrap_or_default();

        let reply_to_ids = msg
            .in_reply_to
            .as_deref()
            .map(|r| parse_refs(r))
            .unwrap_or_default();

        // Check all referenced message IDs for existing thread assignment
        for rid in ref_ids.iter().chain(reply_to_ids.iter()) {
            if let Some(tid) = mid_to_thread.get(rid) {
                assigned_thread = Some(tid.clone());
                break;
            }
        }

        // Fallback: normalized subject grouping
        if assigned_thread.is_none() {
            if let Some(subj) = &msg.subject {
                let norm = normalize_subject(subj);
                if !norm.is_empty() {
                    if let Some(tid) = subject_to_thread.get(&norm) {
                        assigned_thread = Some(tid.clone());
                    }
                }
            }
        }

        // If still no thread, create a new one
        let thread_id = assigned_thread.unwrap_or_else(|| {
            msg.message_id
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
        });

        // Register this message's message_id in the lookup
        if let Some(mid) = &msg.message_id {
            mid_to_thread.insert(mid.clone(), thread_id.clone());
        }

        // Register all referenced IDs as belonging to this thread
        for rid in ref_ids.iter().chain(reply_to_ids.iter()) {
            mid_to_thread.entry(rid.clone()).or_insert_with(|| thread_id.clone());
        }

        // Register subject -> thread
        if let Some(subj) = &msg.subject {
            let norm = normalize_subject(subj);
            if !norm.is_empty() {
                subject_to_thread.entry(norm).or_insert_with(|| thread_id.clone());
            }
        }

        msg.thread_id = Some(thread_id);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::db::Message;

    fn make_message(
        uid: u32,
        message_id: Option<&str>,
        subject: Option<&str>,
        refs: Option<&str>,
        in_reply_to: Option<&str>,
    ) -> Message {
        Message {
            uid,
            message_id: message_id.map(|s| s.to_string()),
            folder: "INBOX".to_string(),
            subject: subject.map(|s| s.to_string()),
            from_addr: Some("test@example.com".to_string()),
            to_addr: Some("other@example.com".to_string()),
            cc: None,
            date: Some("2026-01-15T10:00:00Z".to_string()),
            body_text: None,
            body_html: None,
            flags: None,
            thread_id: None,
            ref_headers: refs.map(|s| s.to_string()),
            in_reply_to: in_reply_to.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_thread_by_references() {
        let mut messages = vec![
            make_message(1, Some("<a@ex.com>"), Some("Hello"), None, None),
            make_message(
                2,
                Some("<b@ex.com>"),
                Some("Re: Hello"),
                Some("<a@ex.com>"),
                Some("<a@ex.com>"),
            ),
            make_message(
                3,
                Some("<c@ex.com>"),
                Some("Re: Re: Hello"),
                Some("<a@ex.com> <b@ex.com>"),
                Some("<b@ex.com>"),
            ),
        ];

        assign_threads(&mut messages);

        // All three should share the same thread_id
        let t1 = messages[0].thread_id.as_ref().unwrap();
        let t2 = messages[1].thread_id.as_ref().unwrap();
        let t3 = messages[2].thread_id.as_ref().unwrap();
        assert_eq!(t1, t2);
        assert_eq!(t2, t3);
    }

    #[test]
    fn test_thread_by_subject_fallback() {
        let mut messages = vec![
            make_message(1, Some("<x@ex.com>"), Some("Project Update"), None, None),
            make_message(
                2,
                Some("<y@ex.com>"),
                Some("Re: Project Update"),
                None,
                None,
            ),
        ];

        assign_threads(&mut messages);

        let t1 = messages[0].thread_id.as_ref().unwrap();
        let t2 = messages[1].thread_id.as_ref().unwrap();
        assert_eq!(t1, t2, "Subject fallback should group them");
    }

    #[test]
    fn test_normalize_subject() {
        assert_eq!(normalize_subject("Re: Hello"), "hello");
        assert_eq!(normalize_subject("Fwd: Hello"), "hello");
        assert_eq!(normalize_subject("Fw: Hello"), "hello");
        assert_eq!(normalize_subject("Re[2]: Hello"), "hello");
        assert_eq!(normalize_subject("Fwd[3]: Hello"), "hello");
        assert_eq!(normalize_subject("RE: FW: Re: Hello"), "hello");
        assert_eq!(normalize_subject("  Re:  Fwd:  Test  "), "test");
        assert_eq!(normalize_subject("Hello"), "hello");
    }

    #[test]
    fn test_unrelated_messages_different_threads() {
        let mut messages = vec![
            make_message(1, Some("<p@ex.com>"), Some("Topic A"), None, None),
            make_message(2, Some("<q@ex.com>"), Some("Topic B"), None, None),
        ];

        assign_threads(&mut messages);

        let t1 = messages[0].thread_id.as_ref().unwrap();
        let t2 = messages[1].thread_id.as_ref().unwrap();
        assert_ne!(t1, t2, "Unrelated messages should be in different threads");
    }

    #[test]
    fn test_no_message_id_gets_uuid_thread() {
        let mut messages = vec![make_message(1, None, Some("Orphan"), None, None)];

        assign_threads(&mut messages);

        let tid = messages[0].thread_id.as_ref().unwrap();
        assert!(!tid.is_empty(), "Should get a UUID thread_id");
    }
}
