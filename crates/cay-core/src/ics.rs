use crate::models::CalendarMeeting;

/// Parses RFC 5545 iCalendar (`.ics`) text and returns meetings starting within
/// `[current_epoch_secs - 300, current_epoch_secs + lookahead_minutes * 60]`.
pub fn parse_ics_meetings(
    ics_content: &str,
    current_epoch_secs: i64,
    lookahead_minutes: u32,
) -> Vec<CalendarMeeting> {
    let unfolded = unfold_ics_lines(ics_content);
    let mut meetings = Vec::new();

    let mut in_event = false;
    let mut uid = String::new();
    let mut summary = String::new();
    let mut dtstart: Option<i64> = None;
    let mut dtend: Option<i64> = None;
    let mut location = String::new();
    let mut description = String::new();
    let mut url = String::new();

    let window_start = current_epoch_secs.saturating_sub(300);
    let window_end = current_epoch_secs.saturating_add(i64::from(lookahead_minutes) * 60);

    for line in unfolded {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("BEGIN:VEVENT") {
            in_event = true;
            uid.clear();
            summary.clear();
            dtstart = None;
            dtend = None;
            location.clear();
            description.clear();
            url.clear();
            continue;
        }

        if trimmed.eq_ignore_ascii_case("END:VEVENT") {
            if in_event {
                if let Some(start_ts) = dtstart {
                    if start_ts >= window_start && start_ts <= window_end {
                        let duration_minutes = dtend
                            .and_then(|end_ts| u32::try_from((end_ts - start_ts).max(0) / 60).ok())
                            .filter(|&d| d > 0)
                            .unwrap_or(30);

                        let combined_text = format!("{location} {description} {url}");
                        let join_url = extract_join_url(&combined_text);
                        let service_name =
                            detect_service_name(&location, &description, join_url.as_deref());

                        let id = if uid.is_empty() {
                            format!("evt-{start_ts}")
                        } else {
                            uid.clone()
                        };
                        let title = if summary.is_empty() {
                            "Upcoming meeting".to_string()
                        } else {
                            summary.clone()
                        };

                        meetings.push(CalendarMeeting {
                            id,
                            title,
                            start_timestamp: start_ts,
                            duration_minutes,
                            service_name,
                            join_url,
                        });
                    }
                }
            }
            in_event = false;
            continue;
        }

        if !in_event {
            continue;
        }

        if let Some((prop, val)) = trimmed.split_once(':') {
            let prop_name = prop.split(';').next().unwrap_or(prop).trim();
            let clean_val = unescape_ics_text(val.trim());

            if prop_name.eq_ignore_ascii_case("UID") {
                uid = clean_val;
            } else if prop_name.eq_ignore_ascii_case("SUMMARY") {
                summary = clean_val;
            } else if prop_name.eq_ignore_ascii_case("DTSTART") {
                dtstart = parse_ics_datetime(&clean_val);
            } else if prop_name.eq_ignore_ascii_case("DTEND") {
                dtend = parse_ics_datetime(&clean_val);
            } else if prop_name.eq_ignore_ascii_case("LOCATION") {
                location = clean_val;
            } else if prop_name.eq_ignore_ascii_case("DESCRIPTION") {
                description = clean_val;
            } else if prop_name.eq_ignore_ascii_case("URL") {
                url = clean_val;
            }
        }
    }

    meetings.sort_by_key(|m| m.start_timestamp);
    meetings
}

/// Unfolds RFC 5545 folded lines (lines starting with a single space or horizontal tab
/// are continuations of the previous line).
fn unfold_ics_lines(raw: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for raw_line in raw.lines() {
        let line = raw_line.trim_end_matches('\r');
        if let Some(rest) = line.strip_prefix(' ').or_else(|| line.strip_prefix('\t')) {
            if let Some(last) = lines.last_mut() {
                last.push_str(rest);
                continue;
            }
        }
        lines.push(line.to_string());
    }
    lines
}

fn unescape_ics_text(s: &str) -> String {
    s.replace("\\n", " ")
        .replace("\\N", " ")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")
}

/// Parses an iCalendar timestamp (`YYYYMMDDTHHMMSSZ` or `YYYYMMDDTHHMMSS`) into Unix epoch seconds.
pub fn parse_ics_datetime(val: &str) -> Option<i64> {
    let clean = val.trim().trim_end_matches('Z');
    let (date_part, time_part) = clean.split_once('T').unwrap_or((clean, "000000"));
    if date_part.len() < 8 {
        return None;
    }

    let year: i64 = date_part.get(0..4)?.parse().ok()?;
    let month: i64 = date_part.get(4..6)?.parse().ok()?;
    let day: i64 = date_part.get(6..8)?.parse().ok()?;

    let hour: i64 = time_part.get(0..2).unwrap_or("00").parse().ok()?;
    let minute: i64 = time_part.get(2..4).unwrap_or("00").parse().ok()?;
    let second: i64 = time_part.get(4..6).unwrap_or("00").parse().ok()?;

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let days = days_from_civil(year, month, day);
    Some(days * 86_400 + hour * 3_600 + minute * 60 + second)
}

/// Howard Hinnant's civil-to-days algorithm (days since 1970-01-01 UTC).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * m + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn extract_join_url(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        let candidate =
            token.trim_matches(|c: char| matches!(c, '<' | '>' | '"' | '\'' | '(' | ')' | ','));
        if candidate.starts_with("https://") || candidate.starts_with("http://") {
            return Some(candidate.to_string());
        }
    }
    None
}

fn detect_service_name(location: &str, description: &str, join_url: Option<&str>) -> String {
    let combined = format!(
        "{} {} {}",
        location.to_ascii_lowercase(),
        description.to_ascii_lowercase(),
        join_url.unwrap_or("").to_ascii_lowercase()
    );

    if combined.contains("teams.microsoft.com") || combined.contains("microsoft teams") {
        "Microsoft Teams".to_string()
    } else if combined.contains("meet.google.com") || combined.contains("google meet") {
        "Google Meet".to_string()
    } else if combined.contains("zoom.us") || combined.contains("zoom") {
        "Zoom".to_string()
    } else if !location.is_empty() && !location.starts_with("http") {
        location.to_string()
    } else {
        "Calendar".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ics_meetings_with_teams_link() {
        let sample_ics = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
BEGIN:VEVENT\r\n\
UID:design-review-123\r\n\
SUMMARY:Design review\r\n\
DTSTART:20261116T190000Z\r\n\
DTEND:20261116T194000Z\r\n\
DESCRIPTION:Join our sync at https://teams.microsoft.com/l/meetup-join/123\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let event_ts = parse_ics_datetime("20261116T190000Z").unwrap();
        let now_ts = event_ts - 12 * 60; // 12 minutes before meeting

        let meetings = parse_ics_meetings(sample_ics, now_ts, 15);
        assert_eq!(meetings.len(), 1);
        assert_eq!(meetings[0].id, "design-review-123");
        assert_eq!(meetings[0].title, "Design review");
        assert_eq!(meetings[0].duration_minutes, 40);
        assert_eq!(meetings[0].service_name, "Microsoft Teams");
        assert_eq!(
            meetings[0].join_url.as_deref(),
            Some("https://teams.microsoft.com/l/meetup-join/123")
        );
    }

    #[test]
    fn test_parse_ics_folded_lines_and_lookahead_filter() {
        let sample_ics = concat!(
            "BEGIN:VCALENDAR\n",
            "BEGIN:VEVENT\n",
            "UID:folded-1\n",
            "SUMMARY:Architecture \n",
            " Deep Dive\n",
            "DTSTART:20261116T200000Z\n",
            "DTEND:20261116T210000Z\n",
            "LOCATION:https://meet.google.com/abc-defg-hij\n",
            "END:VEVENT\n",
            "END:VCALENDAR"
        );

        let event_ts = parse_ics_datetime("20261116T200000Z").unwrap();

        // 60 minutes before: outside 15-minute lookahead
        let early = parse_ics_meetings(sample_ics, event_ts - 3600, 15);
        assert!(early.is_empty());

        // 10 minutes before: inside 15-minute lookahead
        let within = parse_ics_meetings(sample_ics, event_ts - 600, 15);
        assert_eq!(within.len(), 1);
        assert_eq!(within[0].title, "Architecture Deep Dive");
        assert_eq!(within[0].service_name, "Google Meet");
    }
}
