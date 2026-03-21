use ical::IcalParser;
use serde::{Deserialize, Serialize};
use std::io::BufReader;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub event_uid: String,
    pub summary: Option<String>,
    pub dtstart: i64,
    pub dtend: Option<i64>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub organizer: Option<String>,
    pub attendees: String,
    pub sequence: i32,
    pub method: Option<String>,
    pub raw_ics: String,
}

/// Parse an ICS datetime string into a unix epoch timestamp.
/// Handles:
///   - `20260325T100000Z` (UTC)
///   - `20260325T100000` (local, treated as UTC)
///   - Values with TZID parameter (best effort, treated as UTC)
fn parse_ics_datetime(value: &str) -> Option<i64> {
    // Strip any trailing Z and parse the basic format
    let clean = value.trim().trim_end_matches('Z');

    // Try YYYYMMDDTHHMMSS
    if clean.len() == 15 && clean.contains('T') {
        let dt = chrono::NaiveDateTime::parse_from_str(clean, "%Y%m%dT%H%M%S").ok()?;
        return Some(dt.and_utc().timestamp());
    }

    // Try YYYYMMDD (all-day event)
    if clean.len() == 8 {
        let date = chrono::NaiveDate::parse_from_str(clean, "%Y%m%d").ok()?;
        let dt = date.and_hms_opt(0, 0, 0)?;
        return Some(dt.and_utc().timestamp());
    }

    None
}

/// Extract the value from a property, stripping any TZID or other parameters.
/// For DTSTART;TZID=America/New_York:20260325T100000, the property value
/// from the ical crate is already just "20260325T100000".
fn extract_property_value(prop: &ical::property::Property) -> Option<String> {
    prop.value.clone()
}

/// Parse ICS data from raw text and return a list of CalendarEvent structs.
pub fn parse_ics(raw: &str) -> Vec<CalendarEvent> {
    let reader = BufReader::new(raw.as_bytes());
    let parser = IcalParser::new(reader);
    let mut events = Vec::new();

    for calendar_result in parser {
        let calendar = match calendar_result {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Extract VCALENDAR-level METHOD property
        let method = calendar.properties.iter().find_map(|p| {
            if p.name == "METHOD" {
                p.value.clone()
            } else {
                None
            }
        });

        for vevent in &calendar.events {
            let mut event_uid = String::new();
            let mut summary = None;
            let mut dtstart: Option<i64> = None;
            let mut dtend: Option<i64> = None;
            let mut location = None;
            let mut description = None;
            let mut organizer = None;
            let mut sequence: i32 = 0;
            let mut attendees: Vec<serde_json::Value> = Vec::new();

            for prop in &vevent.properties {
                match prop.name.as_str() {
                    "UID" => {
                        if let Some(v) = extract_property_value(prop) {
                            event_uid = v;
                        }
                    }
                    "SUMMARY" => {
                        summary = extract_property_value(prop);
                    }
                    "DTSTART" => {
                        if let Some(v) = extract_property_value(prop) {
                            dtstart = parse_ics_datetime(&v);
                        }
                    }
                    "DTEND" => {
                        if let Some(v) = extract_property_value(prop) {
                            dtend = parse_ics_datetime(&v);
                        }
                    }
                    "LOCATION" => {
                        location = extract_property_value(prop);
                    }
                    "DESCRIPTION" => {
                        description = extract_property_value(prop);
                    }
                    "ORGANIZER" => {
                        // Value is typically "mailto:user@example.com"
                        let val = extract_property_value(prop)
                            .unwrap_or_default()
                            .trim_start_matches("mailto:")
                            .trim_start_matches("MAILTO:")
                            .to_string();
                        if !val.is_empty() {
                            organizer = Some(val);
                        }
                    }
                    "SEQUENCE" => {
                        if let Some(v) = extract_property_value(prop) {
                            sequence = v.parse().unwrap_or(0);
                        }
                    }
                    "ATTENDEE" => {
                        let email = extract_property_value(prop)
                            .unwrap_or_default()
                            .trim_start_matches("mailto:")
                            .trim_start_matches("MAILTO:")
                            .to_string();

                        let params = prop.params.as_ref();
                        let name = params.and_then(|ps| {
                            ps.iter().find_map(|(k, vals)| {
                                if k == "CN" {
                                    vals.first().cloned()
                                } else {
                                    None
                                }
                            })
                        });
                        let partstat = params
                            .and_then(|ps| {
                                ps.iter().find_map(|(k, vals)| {
                                    if k == "PARTSTAT" {
                                        vals.first().cloned()
                                    } else {
                                        None
                                    }
                                })
                            })
                            .unwrap_or_else(|| "NEEDS-ACTION".to_string());

                        attendees.push(serde_json::json!({
                            "email": email,
                            "name": name,
                            "partstat": partstat,
                        }));
                    }
                    _ => {}
                }
            }

            // Skip events without UID or DTSTART
            if event_uid.is_empty() || dtstart.is_none() {
                continue;
            }

            let attendees_json =
                serde_json::to_string(&attendees).unwrap_or_else(|_| "[]".to_string());

            events.push(CalendarEvent {
                event_uid,
                summary,
                dtstart: dtstart.unwrap(),
                dtend,
                location,
                description,
                organizer,
                attendees: attendees_json,
                sequence,
                method: method.clone(),
                raw_ics: raw.to_string(),
            });
        }
    }

    events
}

/// Build an ICS REPLY for an event (ACCEPTED, DECLINED, or TENTATIVE).
pub fn build_ics_reply(event: &CalendarEvent, user_email: &str, partstat: &str) -> String {
    let now = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let dtstart = chrono::DateTime::from_timestamp(event.dtstart, 0)
        .map(|dt| dt.format("%Y%m%dT%H%M%SZ").to_string())
        .unwrap_or_default();
    let dtend = event
        .dtend
        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
        .map(|dt| dt.format("%Y%m%dT%H%M%SZ").to_string());

    let mut ics = String::new();
    ics.push_str("BEGIN:VCALENDAR\r\n");
    ics.push_str("VERSION:2.0\r\n");
    ics.push_str("PRODID:-//ScouterMail//EN\r\n");
    ics.push_str("METHOD:REPLY\r\n");
    ics.push_str("BEGIN:VEVENT\r\n");
    ics.push_str(&format!("UID:{}\r\n", event.event_uid));
    ics.push_str(&format!("SEQUENCE:{}\r\n", event.sequence));
    ics.push_str(&format!("DTSTAMP:{}\r\n", now));
    ics.push_str(&format!("DTSTART:{}\r\n", dtstart));
    if let Some(ref end) = dtend {
        ics.push_str(&format!("DTEND:{}\r\n", end));
    }
    if let Some(ref summary) = event.summary {
        ics.push_str(&format!("SUMMARY:{}\r\n", summary));
    }
    if let Some(ref organizer) = event.organizer {
        ics.push_str(&format!("ORGANIZER:mailto:{}\r\n", organizer));
    }
    ics.push_str(&format!(
        "ATTENDEE;PARTSTAT={}:mailto:{}\r\n",
        partstat, user_email
    ));
    ics.push_str("END:VEVENT\r\n");
    ics.push_str("END:VCALENDAR\r\n");

    ics
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ICS: &str = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
METHOD:REQUEST\r\n\
BEGIN:VEVENT\r\n\
UID:test-uid-123@example.com\r\n\
SUMMARY:Team Meeting\r\n\
DTSTART:20260325T100000Z\r\n\
DTEND:20260325T110000Z\r\n\
LOCATION:Conference Room A\r\n\
DESCRIPTION:Weekly sync\r\n\
ORGANIZER;CN=Alice:mailto:alice@example.com\r\n\
ATTENDEE;CN=Bob;PARTSTAT=NEEDS-ACTION:mailto:bob@example.com\r\n\
ATTENDEE;CN=Carol;PARTSTAT=ACCEPTED:mailto:carol@example.com\r\n\
SEQUENCE:0\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    #[test]
    fn test_parse_ics_basic() {
        let events = parse_ics(SAMPLE_ICS);
        assert_eq!(events.len(), 1);

        let ev = &events[0];
        assert_eq!(ev.event_uid, "test-uid-123@example.com");
        assert_eq!(ev.summary.as_deref(), Some("Team Meeting"));
        assert_eq!(ev.location.as_deref(), Some("Conference Room A"));
        assert_eq!(ev.description.as_deref(), Some("Weekly sync"));
        assert_eq!(ev.organizer.as_deref(), Some("alice@example.com"));
        assert_eq!(ev.method.as_deref(), Some("REQUEST"));
        assert_eq!(ev.sequence, 0);

        // Check DTSTART = 2026-03-25T10:00:00Z => 1774447200
        assert_eq!(ev.dtstart, 1774432800);

        // Check attendees JSON
        let attendees: Vec<serde_json::Value> =
            serde_json::from_str(&ev.attendees).expect("valid JSON");
        assert_eq!(attendees.len(), 2);
        assert_eq!(attendees[0]["email"], "bob@example.com");
        assert_eq!(attendees[0]["partstat"], "NEEDS-ACTION");
        assert_eq!(attendees[1]["email"], "carol@example.com");
        assert_eq!(attendees[1]["partstat"], "ACCEPTED");
    }

    #[test]
    fn test_parse_ics_datetime_formats() {
        // UTC format
        assert!(parse_ics_datetime("20260325T100000Z").is_some());
        // Local format (treated as UTC)
        assert!(parse_ics_datetime("20260325T100000").is_some());
        // All-day
        assert!(parse_ics_datetime("20260325").is_some());
        // Both should parse to the same value
        assert_eq!(
            parse_ics_datetime("20260325T100000Z"),
            parse_ics_datetime("20260325T100000")
        );
    }

    #[test]
    fn test_build_ics_reply() {
        let events = parse_ics(SAMPLE_ICS);
        let ev = &events[0];
        let reply = build_ics_reply(ev, "bob@example.com", "ACCEPTED");

        assert!(reply.contains("METHOD:REPLY"));
        assert!(reply.contains("UID:test-uid-123@example.com"));
        assert!(reply.contains("ATTENDEE;PARTSTAT=ACCEPTED:mailto:bob@example.com"));
        assert!(reply.contains("DTSTAMP:"));
    }
}
