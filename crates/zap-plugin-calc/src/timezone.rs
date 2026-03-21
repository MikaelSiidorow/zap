use chrono::{Local, NaiveTime, TimeZone, Timelike};
use chrono_tz::Tz;

pub struct TimezoneResult {
    pub title: String,
    pub subtitle: String,
}

/// Try to parse input as a timezone conversion query.
/// Patterns:
///   "9am helsinki to new york"
///   "9:30 pm EST to PST"
///   "14:00 UTC to helsinki"
///   "now in tokyo"
///   "3pm pacific to eastern"
pub fn try_convert(input: &str) -> Option<TimezoneResult> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let lower = input.to_lowercase();

    // Split on " to " or " in "
    let (time_and_src, target_str) = if let Some(idx) = lower.find(" to ") {
        (&input[..idx], input[idx + 4..].trim())
    } else if let Some(idx) = lower.find(" in ") {
        (&input[..idx], input[idx + 4..].trim())
    } else {
        return None;
    };

    let time_and_src = time_and_src.trim();
    if time_and_src.is_empty() || target_str.is_empty() {
        return None;
    }

    // Parse target timezone
    let target_tz = resolve_timezone(target_str)?;

    // Parse time and source timezone from the left part
    let (time, source_tz) = parse_time_and_source(time_and_src)?;

    // Build the datetime in source timezone
    let today = Local::now().date_naive();
    let naive_dt = today.and_time(time);
    let source_dt = source_tz.from_local_datetime(&naive_dt).earliest()?;

    // Convert to target timezone
    let target_dt = source_dt.with_timezone(&target_tz);

    let src_abbr = source_dt.format("%Z").to_string();
    let tgt_abbr = target_dt.format("%Z").to_string();

    let title = format_time(&target_dt.time(), &tgt_abbr);
    let subtitle = format!(
        "{} {} → {}",
        format_time_short(&source_dt.time()),
        src_abbr,
        tgt_abbr,
    );

    Some(TimezoneResult { title, subtitle })
}

fn parse_time_and_source(input: &str) -> Option<(NaiveTime, Tz)> {
    let lower = input.to_lowercase();

    // Try "now <tz>" or just "now" (uses local)
    if lower.starts_with("now") {
        let rest = input[3..].trim();
        let tz = if rest.is_empty() {
            local_tz()
        } else {
            resolve_timezone(rest)?
        };
        let now = chrono::Utc::now().with_timezone(&tz);
        return Some((now.time(), tz));
    }

    // Try to extract time from the beginning, rest is timezone
    // Patterns: "9am", "9 am", "9:30pm", "9:30 pm", "14:00", "noon", "midnight"
    let (time, rest) = parse_time_prefix(&lower, input)?;

    let rest = rest.trim();
    let tz = if rest.is_empty() {
        local_tz()
    } else {
        // Strip "time" suffix: "helsinki time" → "helsinki"
        let tz_str = rest
            .strip_suffix(" time")
            .or_else(|| rest.strip_suffix(" Time"))
            .unwrap_or(rest);
        resolve_timezone(tz_str)?
    };

    Some((time, tz))
}

/// Parse a time from the beginning of the string, return (time, remaining_str).
/// `lower` is the lowercased version, `original` preserves case for the remainder.
fn parse_time_prefix<'a>(lower: &str, original: &'a str) -> Option<(NaiveTime, &'a str)> {
    // "noon"
    if lower.starts_with("noon") {
        return Some((NaiveTime::from_hms_opt(12, 0, 0)?, &original[4..]));
    }
    // "midnight"
    if lower.starts_with("midnight") {
        return Some((NaiveTime::from_hms_opt(0, 0, 0)?, &original[8..]));
    }

    // Try HH:MM am/pm or HH:MM (24h) or Ham/Hpm
    let mut i = 0;
    let bytes = lower.as_bytes();

    // Consume digits for hour
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 {
        return None;
    }
    let hour_str = &lower[..i];
    let mut hour: u32 = hour_str.parse().ok()?;
    let mut minute: u32 = 0;

    // Optional :MM
    let mut j = i;
    if j < bytes.len() && bytes[j] == b':' {
        j += 1;
        let min_start = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j > min_start {
            minute = lower[min_start..j].parse().ok()?;
        }
        i = j;
    }

    // Optional space
    if i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }

    // Check for am/pm
    if lower[i..].starts_with("am") {
        if hour == 12 {
            hour = 0;
        }
        i += 2;
    } else if lower[i..].starts_with("pm") {
        if hour != 12 {
            hour += 12;
        }
        i += 2;
    }

    if hour > 23 || minute > 59 {
        return None;
    }

    let time = NaiveTime::from_hms_opt(hour, minute, 0)?;
    Some((time, &original[i..]))
}

fn format_time(time: &NaiveTime, tz_name: &str) -> String {
    let (h12, ampm) = if time.hour() == 0 {
        (12, "AM")
    } else if time.hour() < 12 {
        (time.hour(), "AM")
    } else if time.hour() == 12 {
        (12, "PM")
    } else {
        (time.hour() - 12, "PM")
    };

    if time.minute() == 0 {
        format!("{h12} {ampm} {tz_name}")
    } else {
        format!("{h12}:{:02} {ampm} {tz_name}", time.minute())
    }
}

fn format_time_short(time: &NaiveTime) -> String {
    let (h12, ampm) = if time.hour() == 0 {
        (12, "AM")
    } else if time.hour() < 12 {
        (time.hour(), "AM")
    } else if time.hour() == 12 {
        (12, "PM")
    } else {
        (time.hour() - 12, "PM")
    };

    if time.minute() == 0 {
        format!("{h12} {ampm}")
    } else {
        format!("{h12}:{:02} {ampm}", time.minute())
    }
}

fn local_tz() -> Tz {
    // Try to detect system timezone via the TZ env var or /etc/localtime
    if let Ok(tz_str) = std::env::var("TZ") {
        if let Ok(tz) = tz_str.parse::<Tz>() {
            return tz;
        }
    }
    // Try reading /etc/timezone (Debian/Ubuntu)
    if let Ok(content) = std::fs::read_to_string("/etc/timezone") {
        if let Ok(tz) = content.trim().parse::<Tz>() {
            return tz;
        }
    }
    // Try reading /etc/localtime symlink target
    if let Ok(link) = std::fs::read_link("/etc/localtime") {
        let path = link.to_string_lossy();
        if let Some(tz_part) = path.strip_prefix("/usr/share/zoneinfo/") {
            if let Ok(tz) = tz_part.parse::<Tz>() {
                return tz;
            }
        }
    }
    // Fallback to UTC
    Tz::UTC
}

fn resolve_timezone(input: &str) -> Option<Tz> {
    let lower = input.trim().to_lowercase();

    // Try direct IANA parse first (e.g., "Europe/Helsinki")
    if let Ok(tz) = input.trim().parse::<Tz>() {
        return Some(tz);
    }

    // Common abbreviations and names
    let tz = match lower.as_str() {
        // UTC
        "utc" | "gmt" | "z" => Tz::UTC,

        // US timezones
        "est" | "eastern" | "east coast" | "us eastern" => Tz::US__Eastern,
        "cst" | "central" | "us central" => Tz::US__Central,
        "mst" | "mountain" | "us mountain" => Tz::US__Mountain,
        "pst" | "pacific" | "west coast" | "us pacific" => Tz::US__Pacific,
        "hst" | "hawaii" => Tz::US__Hawaii,
        "akst" | "alaska" => Tz::US__Alaska,

        // Major US cities
        "new york" | "nyc" | "ny" => Tz::America__New_York,
        "chicago" => Tz::America__Chicago,
        "denver" => Tz::America__Denver,
        "los angeles" | "la" => Tz::America__Los_Angeles,
        "san francisco" | "sf" => Tz::America__Los_Angeles,
        "seattle" => Tz::America__Los_Angeles,
        "phoenix" => Tz::America__Phoenix,
        "anchorage" => Tz::America__Anchorage,
        "honolulu" => Tz::US__Hawaii,

        // Canada
        "toronto" => Tz::America__Toronto,
        "vancouver" => Tz::America__Vancouver,
        "montreal" => Tz::America__Montreal,

        // Europe
        "london" | "uk" | "gmt+0" | "bst" | "british" => Tz::Europe__London,
        "paris" => Tz::Europe__Paris,
        "berlin" | "germany" => Tz::Europe__Berlin,
        "amsterdam" => Tz::Europe__Amsterdam,
        "brussels" => Tz::Europe__Brussels,
        "madrid" | "spain" => Tz::Europe__Madrid,
        "rome" | "italy" => Tz::Europe__Rome,
        "zurich" | "switzerland" => Tz::Europe__Zurich,
        "vienna" | "austria" => Tz::Europe__Vienna,
        "stockholm" | "sweden" => Tz::Europe__Stockholm,
        "oslo" | "norway" => Tz::Europe__Oslo,
        "copenhagen" | "denmark" => Tz::Europe__Copenhagen,
        "helsinki" | "finland" => Tz::Europe__Helsinki,
        "warsaw" | "poland" => Tz::Europe__Warsaw,
        "prague" | "czech" => Tz::Europe__Prague,
        "budapest" | "hungary" => Tz::Europe__Budapest,
        "athens" | "greece" => Tz::Europe__Athens,
        "istanbul" | "turkey" => Tz::Europe__Istanbul,
        "moscow" | "russia" => Tz::Europe__Moscow,
        "lisbon" | "portugal" => Tz::Europe__Lisbon,
        "dublin" | "ireland" => Tz::Europe__Dublin,
        "cet" | "central european" => Tz::Europe__Berlin,
        "eet" | "eastern european" => Tz::Europe__Helsinki,
        "wet" | "western european" => Tz::Europe__London,

        // Asia
        "tokyo" | "japan" | "jst" => Tz::Asia__Tokyo,
        "shanghai" | "beijing" | "china" | "cst asia" => Tz::Asia__Shanghai,
        "hong kong" | "hk" | "hkt" => Tz::Asia__Hong_Kong,
        "singapore" | "sg" | "sgt" => Tz::Asia__Singapore,
        "taipei" | "taiwan" => Tz::Asia__Taipei,
        "seoul" | "korea" | "kst" => Tz::Asia__Seoul,
        "mumbai" | "india" | "ist" | "kolkata" => Tz::Asia__Kolkata,
        "delhi" | "new delhi" => Tz::Asia__Kolkata,
        "dubai" | "uae" | "gst" => Tz::Asia__Dubai,
        "bangkok" | "thailand" | "ict" => Tz::Asia__Bangkok,
        "jakarta" | "indonesia" | "wib" => Tz::Asia__Jakarta,
        "kuala lumpur" | "malaysia" => Tz::Asia__Kuala_Lumpur,
        "manila" | "philippines" | "pht" => Tz::Asia__Manila,
        "riyadh" | "saudi" => Tz::Asia__Riyadh,
        "tehran" | "iran" => Tz::Asia__Tehran,
        "karachi" | "pakistan" | "pkt" => Tz::Asia__Karachi,

        // Oceania
        "sydney" | "australia" | "aest" => Tz::Australia__Sydney,
        "melbourne" => Tz::Australia__Melbourne,
        "brisbane" => Tz::Australia__Brisbane,
        "perth" | "awst" => Tz::Australia__Perth,
        "auckland" | "new zealand" | "nz" | "nzst" => Tz::Pacific__Auckland,

        // South America
        "sao paulo" | "brazil" | "brt" => Tz::America__Sao_Paulo,
        "buenos aires" | "argentina" | "art" => Tz::America__Argentina__Buenos_Aires,
        "santiago" | "chile" => Tz::America__Santiago,
        "bogota" | "colombia" | "cot" => Tz::America__Bogota,
        "lima" | "peru" | "pet" => Tz::America__Lima,

        // Africa
        "cairo" | "egypt" => Tz::Africa__Cairo,
        "nairobi" | "kenya" | "eat" => Tz::Africa__Nairobi,
        "lagos" | "nigeria" | "wat" => Tz::Africa__Lagos,
        "johannesburg" | "south africa" | "sast" => Tz::Africa__Johannesburg,
        "casablanca" | "morocco" => Tz::Africa__Casablanca,

        _ => return None,
    };

    Some(tz)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_conversion() {
        let result = try_convert("9am UTC to EST").unwrap();
        // EST/EDT depending on DST
        assert!(result.title.starts_with("4 AM"));
    }

    #[test]
    fn test_city_names() {
        let result = try_convert("9am helsinki to new york").unwrap();
        // Helsinki to NY is always 7 hours difference
        assert!(result.title.contains("AM") || result.title.contains("PM"));
        // Subtitle shows source → target
        assert!(result.subtitle.contains("→"));
    }

    #[test]
    fn test_24h_format() {
        let result = try_convert("14:00 UTC to tokyo").unwrap();
        assert!(result.title.starts_with("11 PM"));
    }

    #[test]
    fn test_with_minutes() {
        let result = try_convert("9:30am UTC to EST").unwrap();
        assert!(result.title.starts_with("4:30 AM") || result.title.starts_with("5:30 AM"));
    }

    #[test]
    fn test_pm() {
        let result = try_convert("3pm UTC to EST").unwrap();
        assert!(result.title.starts_with("10 AM") || result.title.starts_with("11 AM"));
    }

    #[test]
    fn test_in_keyword() {
        let result = try_convert("noon UTC in tokyo").unwrap();
        assert!(result.title.starts_with("9 PM"));
    }

    #[test]
    fn test_time_suffix_stripped() {
        let result = try_convert("9am helsinki time to new york").unwrap();
        assert!(result.subtitle.contains("→"));
    }

    #[test]
    fn test_common_aliases() {
        let result = try_convert("9am pacific to eastern").unwrap();
        // Pacific to Eastern is +3 hours
        assert!(result.title.starts_with("12 PM") || result.title.starts_with("12:00 PM"));
    }

    #[test]
    fn test_midnight() {
        let result = try_convert("midnight UTC to tokyo").unwrap();
        assert!(result.title.starts_with("9 AM"));
    }

    #[test]
    fn test_invalid_input() {
        assert!(try_convert("hello world").is_none());
        assert!(try_convert("").is_none());
        assert!(try_convert("9am to").is_none());
        assert!(try_convert("to tokyo").is_none());
        assert!(try_convert("9am fakezone to tokyo").is_none());
    }
}
