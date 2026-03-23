mod eval;
mod timezone;
mod units;

use zap_core::{Action, KeyboardHint, Plugin, PluginMeta, PluginResult};

pub struct CalcPlugin;

impl Plugin for CalcPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta::new("calc", "Calculator")
            .description("Math expressions and timezone conversions")
            .example("= 2+2 or = 9am tokyo to new york")
            .prefix("=")
    }

    fn search(&self, query: &str) -> Vec<PluginResult> {
        let input = query.trim();
        if input.is_empty() {
            return vec![];
        }

        if let Ok(value) = eval::evaluate(input) {
            let formatted = format_number(value);
            return vec![PluginResult::new("calc", "result", &formatted)
                .subtitle(format!("= {input}"))
                .score(100)
                .action(Action::Copy { content: formatted })];
        }

        if let Some(unit_result) = units::try_convert(input) {
            return vec![PluginResult::new("calc", "unit_result", &unit_result.title)
                .subtitle(unit_result.subtitle)
                .score(100)
                .action(Action::Copy {
                    content: unit_result.title,
                })];
        }

        if let Some(tz_result) = timezone::try_convert(input) {
            return vec![PluginResult::new("calc", "tz_result", &tz_result.title)
                .subtitle(tz_result.subtitle)
                .score(100)
                .action(Action::Copy {
                    content: tz_result.title,
                })];
        }

        vec![]
    }

    fn execute(&self, _result_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn hints(&self) -> Vec<KeyboardHint> {
        vec![KeyboardHint {
            key: "Enter".into(),
            label: "Copy result".into(),
        }]
    }
}

fn format_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value.is_infinite() {
        return if value.is_sign_positive() {
            "Infinity".into()
        } else {
            "-Infinity".into()
        };
    }
    if value == value.trunc() && value.abs() < 1e15 {
        return format!("{}", value as i64);
    }
    let s = format!("{:.10e}", value);
    let parts: Vec<&str> = s.split('e').collect();
    if parts.len() == 2 {
        let mantissa: f64 = parts[0].parse().unwrap_or(value);
        let exp: i32 = parts[1].parse().unwrap_or(0);
        let precision = 10usize.saturating_sub(1);
        let reconstructed = mantissa * 10f64.powi(exp);
        let formatted = format!("{reconstructed:.prec$}", prec = precision);
        if formatted.contains('.') {
            let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
            return trimmed.to_string();
        }
        return formatted;
    }
    format!("{value}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_integer_valued() {
        assert_eq!(format_number(4.0), "4");
        assert_eq!(format_number(1024.0), "1024");
        assert_eq!(format_number(-5.0), "-5");
        assert_eq!(format_number(0.0), "0");
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn format_decimal() {
        assert_eq!(format_number(3.14), "3.14");
        assert_eq!(format_number(std::f64::consts::PI * 2.0), "6.283185307");
    }

    #[test]
    fn format_special() {
        assert_eq!(format_number(f64::NAN), "NaN");
        assert_eq!(format_number(f64::INFINITY), "Infinity");
        assert_eq!(format_number(f64::NEG_INFINITY), "-Infinity");
    }

    #[test]
    fn plugin_search() {
        let plugin = CalcPlugin;
        let results = plugin.search("2+2");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "4");
        assert_eq!(results[0].subtitle.as_deref(), Some("= 2+2"));
        assert!(matches!(
            &results[0].action,
            Action::Copy { content } if content == "4"
        ));
    }

    #[test]
    fn plugin_empty_query() {
        let plugin = CalcPlugin;
        assert!(plugin.search("").is_empty());
        assert!(plugin.search("  ").is_empty());
    }

    #[test]
    fn plugin_invalid_query() {
        let plugin = CalcPlugin;
        assert!(plugin.search("abc").is_empty());
    }
}
