use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use mailparse::{MailHeaderMap, ParsedMail, parse_mail};

// ---------------------------------------------------------------------------
// Domain structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct InlineImage {
    pub content_id: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ParsedEmail {
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub cc: Option<String>,
    pub date: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub references: Vec<String>,
    pub in_reply_to: Option<String>,
    pub attachments: Vec<Attachment>,
    pub inline_images: Vec<InlineImage>,
    pub raw_headers: String,
    pub calendar_data: Vec<String>,
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Parse raw email bytes into a structured `ParsedEmail`.
pub fn parse_email(raw: &[u8]) -> Result<ParsedEmail, String> {
    let parsed = parse_mail(raw).map_err(|e| format!("mailparse error: {}", e))?;

    let headers = &parsed.headers;

    let message_id = headers.get_first_value("Message-ID");
    let subject = headers.get_first_value("Subject");
    let from = headers.get_first_value("From");
    let to = headers.get_first_value("To");
    let cc = headers.get_first_value("Cc");
    let date = headers.get_first_value("Date");
    let in_reply_to = headers.get_first_value("In-Reply-To");

    let references = headers
        .get_first_value("References")
        .map(|r| parse_message_id_list(&r))
        .unwrap_or_default();

    // Build raw_headers string
    let raw_headers = headers
        .iter()
        .map(|h| format!("{}: {}", h.get_key(), h.get_value()))
        .collect::<Vec<_>>()
        .join("\n");

    // Extract MIME parts
    let mut body_text = None;
    let mut body_html = None;
    let mut attachments = Vec::new();
    let mut inline_images = Vec::new();
    let mut calendar_data = Vec::new();

    extract_parts(
        &parsed,
        &mut body_text,
        &mut body_html,
        &mut attachments,
        &mut inline_images,
        &mut calendar_data,
    );

    Ok(ParsedEmail {
        message_id,
        subject,
        from,
        to,
        cc,
        date,
        body_text,
        body_html,
        references,
        in_reply_to,
        attachments,
        inline_images,
        raw_headers,
        calendar_data,
    })
}

/// Parse a space/comma separated list of message-IDs (angle-bracket format).
fn parse_message_id_list(value: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut current = String::new();
    let mut inside = false;

    for ch in value.chars() {
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

/// Recursively extract MIME parts from a parsed email.
fn extract_parts(
    part: &ParsedMail,
    body_text: &mut Option<String>,
    body_html: &mut Option<String>,
    attachments: &mut Vec<Attachment>,
    inline_images: &mut Vec<InlineImage>,
    calendar_data: &mut Vec<String>,
) {
    let content_type = part.ctype.mimetype.to_lowercase();

    if !part.subparts.is_empty() {
        for sub in &part.subparts {
            extract_parts(sub, body_text, body_html, attachments, inline_images, calendar_data);
        }
        return;
    }

    // Check for inline image (has Content-ID)
    let content_id = part.headers.get_first_value("Content-ID");
    let content_disposition = part
        .headers
        .get_first_value("Content-Disposition")
        .unwrap_or_default()
        .to_lowercase();

    if content_type == "text/calendar" {
        if let Ok(body) = part.get_body() {
            calendar_data.push(body);
        }
    } else if content_type == "text/plain" && body_text.is_none() {
        if let Ok(body) = part.get_body() {
            *body_text = Some(body);
        }
    } else if content_type == "text/html" && body_html.is_none() {
        if let Ok(body) = part.get_body() {
            *body_html = Some(body);
        }
    } else if let Some(cid) = content_id {
        // Inline image
        if let Ok(data) = part.get_body_raw() {
            let cid_clean = cid.trim_matches(|c| c == '<' || c == '>').to_string();
            inline_images.push(InlineImage {
                content_id: cid_clean,
                content_type: content_type.clone(),
                data,
            });
        }
    } else if content_disposition.starts_with("attachment")
        || !content_type.starts_with("text/")
    {
        // Attachment
        if let Ok(data) = part.get_body_raw() {
            let filename = part
                .ctype
                .params
                .get("name")
                .cloned()
                .or_else(|| {
                    // Try Content-Disposition filename param
                    extract_disposition_filename(&content_disposition)
                })
                .unwrap_or_else(|| "unnamed".to_string());

            let size = data.len();
            attachments.push(Attachment {
                filename,
                content_type,
                size,
                data,
            });
        }
    }
}

/// Try to extract filename from a Content-Disposition header value.
fn extract_disposition_filename(disposition: &str) -> Option<String> {
    for part in disposition.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("filename=") {
            return Some(rest.trim_matches('"').to_string());
        }
    }
    None
}

/// Replace `cid:xxx` references in HTML with base64 data URIs using inline images.
pub fn resolve_cid_images(html: &str, inline_images: &[InlineImage]) -> String {
    let mut result = html.to_string();
    for img in inline_images {
        let cid_ref = format!("cid:{}", img.content_id);
        let b64 = BASE64.encode(&img.data);
        let data_uri = format!("data:{};base64,{}", img.content_type, b64);
        result = result.replace(&cid_ref, &data_uri);
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn build_simple_email() -> Vec<u8> {
        let raw = b"From: alice@example.com\r\n\
To: bob@example.com\r\n\
Subject: Hello World\r\n\
Message-ID: <msg1@example.com>\r\n\
Date: Mon, 1 Jan 2026 10:00:00 +0000\r\n\
In-Reply-To: <msg0@example.com>\r\n\
References: <msg0@example.com> <msg-1@example.com>\r\n\
Content-Type: text/plain\r\n\
\r\n\
This is a test email.";
        raw.to_vec()
    }

    fn build_multipart_email() -> Vec<u8> {
        let raw = b"From: alice@example.com\r\n\
To: bob@example.com\r\n\
Subject: Multipart Test\r\n\
Message-ID: <multi@example.com>\r\n\
Date: Mon, 1 Jan 2026 10:00:00 +0000\r\n\
Content-Type: multipart/alternative; boundary=\"boundary1\"\r\n\
\r\n\
--boundary1\r\n\
Content-Type: text/plain\r\n\
\r\n\
Plain text body\r\n\
--boundary1\r\n\
Content-Type: text/html\r\n\
\r\n\
<html><body>HTML body</body></html>\r\n\
--boundary1--\r\n";
        raw.to_vec()
    }

    #[test]
    fn test_parse_simple_email() {
        let raw = build_simple_email();
        let parsed = parse_email(&raw).unwrap();

        assert_eq!(parsed.message_id.as_deref(), Some("<msg1@example.com>"));
        assert_eq!(parsed.subject.as_deref(), Some("Hello World"));
        assert_eq!(parsed.from.as_deref(), Some("alice@example.com"));
        assert_eq!(parsed.to.as_deref(), Some("bob@example.com"));
        assert_eq!(
            parsed.in_reply_to.as_deref(),
            Some("<msg0@example.com>")
        );
        assert_eq!(parsed.references.len(), 2);
        assert_eq!(parsed.references[0], "<msg0@example.com>");
        assert_eq!(parsed.references[1], "<msg-1@example.com>");
        assert!(parsed.body_text.is_some());
        assert!(parsed.body_text.unwrap().contains("test email"));
    }

    #[test]
    fn test_parse_multipart_email() {
        let raw = build_multipart_email();
        let parsed = parse_email(&raw).unwrap();

        assert!(parsed.body_text.is_some());
        assert!(parsed.body_text.as_deref().unwrap().contains("Plain text"));
        assert!(parsed.body_html.is_some());
        assert!(parsed.body_html.as_deref().unwrap().contains("HTML body"));
    }

    #[test]
    fn test_resolve_cid_images() {
        let html = r#"<html><body><img src="cid:image001"></body></html>"#;
        let images = vec![InlineImage {
            content_id: "image001".to_string(),
            content_type: "image/png".to_string(),
            data: vec![0x89, 0x50, 0x4e, 0x47],
        }];

        let resolved = resolve_cid_images(html, &images);
        assert!(!resolved.contains("cid:image001"));
        assert!(resolved.contains("data:image/png;base64,"));
    }

    #[test]
    fn test_parse_message_id_list() {
        let refs = "<a@b.com> <c@d.com> <e@f.com>";
        let ids = parse_message_id_list(refs);
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0], "<a@b.com>");
        assert_eq!(ids[1], "<c@d.com>");
        assert_eq!(ids[2], "<e@f.com>");
    }

    #[test]
    fn test_raw_headers_built() {
        let raw = build_simple_email();
        let parsed = parse_email(&raw).unwrap();
        assert!(parsed.raw_headers.contains("Message-ID"));
        assert!(parsed.raw_headers.contains("Subject"));
    }
}
