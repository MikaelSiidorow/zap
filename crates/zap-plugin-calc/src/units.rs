pub struct UnitResult {
    pub title: String,
    pub subtitle: String,
}

/// Try to parse input as a unit conversion.
/// Pattern: <number> <unit> to/in <unit>
pub fn try_convert(input: &str) -> Option<UnitResult> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let lower = input.to_lowercase();

    // Split on " to " or " in "
    let (left, target_str) = if let Some(idx) = lower.find(" to ") {
        (&input[..idx], input[idx + 4..].trim())
    } else if let Some(idx) = lower.find(" in ") {
        (&input[..idx], input[idx + 4..].trim())
    } else {
        return None;
    };

    let left = left.trim();
    if left.is_empty() || target_str.is_empty() {
        return None;
    }

    // Parse number and source unit from left side
    let (value, source_unit) = parse_value_and_unit(left)?;
    let target_unit = normalize_unit(target_str)?;
    // Find conversion
    let result = convert(value, &source_unit, &target_unit)?;

    let formatted = format_result(result);
    let target_display = display_unit(&target_unit, result);
    let source_display = display_unit(&source_unit, value);

    let title = format!("{formatted} {target_display}");
    let subtitle = format!(
        "{} {} → {}",
        format_result(value),
        source_display,
        target_display,
    );

    Some(UnitResult { title, subtitle })
}

fn parse_value_and_unit(input: &str) -> Option<(f64, String)> {
    let input = input.trim();

    // Handle "72°F", "100°C" — degree symbol attached
    if let Some(idx) = input.find('°') {
        let num_str = input[..idx].trim();
        let unit_str = input[idx + '°'.len_utf8()..].trim();
        let value: f64 = num_str.parse().ok()?;
        let unit = normalize_unit(&format!("°{unit_str}"))?;
        return Some((value, unit));
    }

    // Find where number ends and unit begins
    let bytes = input.as_bytes();
    let mut i = 0;

    // Skip optional negative sign
    if i < bytes.len() && bytes[i] == b'-' {
        i += 1;
    }
    // Consume digits and decimal point
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
        i += 1;
    }
    if i == 0 || (i == 1 && bytes[0] == b'-') {
        return None;
    }

    let num_str = &input[..i];
    let unit_str = input[i..].trim();

    let value: f64 = num_str.parse().ok()?;
    let unit = normalize_unit(unit_str)?;

    Some((value, unit))
}

fn normalize_unit(input: &str) -> Option<String> {
    let lower = input.trim().to_lowercase();
    let unit = match lower.as_str() {
        // Length
        "m" | "meter" | "meters" | "metre" | "metres" => "m",
        "km" | "kilometer" | "kilometers" | "kilometre" | "kilometres" => "km",
        "cm" | "centimeter" | "centimeters" | "centimetre" | "centimetres" => "cm",
        "mm" | "millimeter" | "millimeters" | "millimetre" | "millimetres" => "mm",
        "mi" | "mile" | "miles" => "mi",
        "yd" | "yard" | "yards" => "yd",
        "ft" | "foot" | "feet" => "ft",
        "in" | "inch" | "inches" => "in",
        "nm" | "nmi" | "nautical mile" | "nautical miles" => "nmi",

        // Weight
        "kg" | "kilogram" | "kilograms" | "kilo" | "kilos" => "kg",
        "g" | "gram" | "grams" => "g",
        "mg" | "milligram" | "milligrams" => "mg",
        "lb" | "lbs" | "pound" | "pounds" => "lb",
        "oz" | "ounce" | "ounces" => "oz",
        "st" | "stone" | "stones" => "st",
        "t" | "ton" | "tons" | "tonne" | "tonnes" | "metric ton" | "metric tons" => "t",

        // Temperature
        "c" | "celsius" | "°c" => "°C",
        "f" | "fahrenheit" | "°f" => "°F",
        "k" | "kelvin" => "K",

        // Volume
        "l" | "liter" | "liters" | "litre" | "litres" => "L",
        "ml" | "milliliter" | "milliliters" | "millilitre" | "millilitres" => "mL",
        "gal" | "gallon" | "gallons" => "gal",
        "qt" | "quart" | "quarts" => "qt",
        "pt" | "pint" | "pints" => "pt",
        "cup" | "cups" => "cup",
        "fl oz" | "fluid ounce" | "fluid ounces" => "fl_oz",
        "tbsp" | "tablespoon" | "tablespoons" => "tbsp",
        "tsp" | "teaspoon" | "teaspoons" => "tsp",

        // Speed
        "mph" | "miles per hour" | "mi/h" => "mph",
        "km/h" | "kmh" | "kph" | "kilometers per hour" => "km/h",
        "m/s" | "meters per second" => "m/s",
        "knot" | "knots" | "kn" | "kt" => "knot",

        // Area
        "sqm" | "m2" | "m²" | "square meter" | "square meters" | "square metre"
        | "square metres" => "m²",
        "sqft" | "ft2" | "ft²" | "square foot" | "square feet" => "ft²",
        "sqmi" | "mi2" | "mi²" | "square mile" | "square miles" => "mi²",
        "sqkm" | "km2" | "km²" | "square kilometer" | "square kilometers" => "km²",
        "ha" | "hectare" | "hectares" => "ha",
        "acre" | "acres" | "ac" => "acre",

        // Data
        "b" | "byte" | "bytes" => "B",
        "kb" | "kilobyte" | "kilobytes" => "KB",
        "mb" | "megabyte" | "megabytes" => "MB",
        "gb" | "gigabyte" | "gigabytes" => "GB",
        "tb" | "terabyte" | "terabytes" => "TB",

        // Time
        "s" | "sec" | "second" | "seconds" => "s",
        "min" | "minute" | "minutes" => "min",
        "h" | "hr" | "hour" | "hours" => "h",
        "d" | "day" | "days" => "d",
        "wk" | "week" | "weeks" => "wk",

        // Energy
        "j" | "joule" | "joules" => "J",
        "kj" | "kilojoule" | "kilojoules" => "kJ",
        "cal" | "calorie" | "calories" => "cal",
        "kcal" | "kilocalorie" | "kilocalories" => "kcal",
        "kwh" | "kilowatt hour" | "kilowatt hours" => "kWh",

        _ => return None,
    };
    Some(unit.to_string())
}

fn convert(value: f64, from: &str, to: &str) -> Option<f64> {
    // Temperature is special (not linear scaling)
    if is_temp(from) && is_temp(to) {
        return Some(convert_temp(value, from, to));
    }

    // For everything else: convert to base unit, then to target
    let from_factor = to_base_factor(from)?;
    let to_factor = to_base_factor(to)?;

    // Check same dimension (both factors must be for the same base unit)
    if dimension(from) != dimension(to) {
        return None;
    }

    Some(value * from_factor / to_factor)
}

fn is_temp(unit: &str) -> bool {
    matches!(unit, "°C" | "°F" | "K")
}

fn convert_temp(value: f64, from: &str, to: &str) -> f64 {
    // Convert to Celsius first
    let celsius = match from {
        "°C" => value,
        "°F" => (value - 32.0) * 5.0 / 9.0,
        "K" => value - 273.15,
        _ => unreachable!(),
    };
    // Convert from Celsius to target
    match to {
        "°C" => celsius,
        "°F" => celsius * 9.0 / 5.0 + 32.0,
        "K" => celsius + 273.15,
        _ => unreachable!(),
    }
}

/// Returns (dimension, factor_to_base_unit)
fn dimension(unit: &str) -> &'static str {
    match unit {
        "m" | "km" | "cm" | "mm" | "mi" | "yd" | "ft" | "in" | "nmi" => "length",
        "kg" | "g" | "mg" | "lb" | "oz" | "st" | "t" => "mass",
        "L" | "mL" | "gal" | "qt" | "pt" | "cup" | "fl_oz" | "tbsp" | "tsp" => "volume",
        "mph" | "km/h" | "m/s" | "knot" => "speed",
        "m²" | "ft²" | "mi²" | "km²" | "ha" | "acre" => "area",
        "B" | "KB" | "MB" | "GB" | "TB" => "data",
        "s" | "min" | "h" | "d" | "wk" => "time",
        "J" | "kJ" | "cal" | "kcal" | "kWh" => "energy",
        _ => "unknown",
    }
}

/// Factor to multiply by to convert to the base unit of this dimension.
fn to_base_factor(unit: &str) -> Option<f64> {
    let factor = match unit {
        // Length → meters
        "m" => 1.0,
        "km" => 1000.0,
        "cm" => 0.01,
        "mm" => 0.001,
        "mi" => 1609.344,
        "yd" => 0.9144,
        "ft" => 0.3048,
        "in" => 0.0254,
        "nmi" => 1852.0,

        // Mass → kilograms
        "kg" => 1.0,
        "g" => 0.001,
        "mg" => 0.000001,
        "lb" => 0.45359237,
        "oz" => 0.028349523125,
        "st" => 6.35029318,
        "t" => 1000.0,

        // Volume → liters
        "L" => 1.0,
        "mL" => 0.001,
        "gal" => 3.785411784,
        "qt" => 0.946352946,
        "pt" => 0.473176473,
        "cup" => 0.2365882365,
        "fl_oz" => 0.0295735295625,
        "tbsp" => 0.01478676478125,
        "tsp" => 0.00492892159375,

        // Speed → m/s
        "m/s" => 1.0,
        "km/h" => 1.0 / 3.6,
        "mph" => 0.44704,
        "knot" => 0.514444,

        // Area → m²
        "m²" => 1.0,
        "ft²" => 0.09290304,
        "mi²" => 2_589_988.110336,
        "km²" => 1_000_000.0,
        "ha" => 10_000.0,
        "acre" => 4046.8564224,

        // Data → bytes
        "B" => 1.0,
        "KB" => 1024.0,
        "MB" => 1_048_576.0,
        "GB" => 1_073_741_824.0,
        "TB" => 1_099_511_627_776.0,

        // Time → seconds
        "s" => 1.0,
        "min" => 60.0,
        "h" => 3600.0,
        "d" => 86400.0,
        "wk" => 604800.0,

        // Energy → joules
        "J" => 1.0,
        "kJ" => 1000.0,
        "cal" => 4.184,
        "kcal" => 4184.0,
        "kWh" => 3_600_000.0,

        _ => return None,
    };
    Some(factor)
}

fn format_result(value: f64) -> String {
    if value == value.trunc() && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        // Up to 6 decimal places, trim trailing zeros
        let s = format!("{value:.6}");
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    }
}

fn display_unit(unit: &str, value: f64) -> &str {
    let plural = value.abs() != 1.0;
    match unit {
        // Length
        "m" => {
            if plural {
                "meters"
            } else {
                "meter"
            }
        }
        "km" => "km",
        "cm" => "cm",
        "mm" => "mm",
        "mi" => {
            if plural {
                "miles"
            } else {
                "mile"
            }
        }
        "yd" => {
            if plural {
                "yards"
            } else {
                "yard"
            }
        }
        "ft" => "ft",
        "in" => "in",
        "nmi" => "nmi",

        // Mass
        "kg" => "kg",
        "g" => "g",
        "mg" => "mg",
        "lb" => {
            if plural {
                "lbs"
            } else {
                "lb"
            }
        }
        "oz" => "oz",
        "st" => "st",
        "t" => {
            if plural {
                "tonnes"
            } else {
                "tonne"
            }
        }

        // Temperature
        "°C" => "°C",
        "°F" => "°F",
        "K" => "K",

        // Volume
        "L" => "L",
        "mL" => "mL",
        "gal" => {
            if plural {
                "gallons"
            } else {
                "gallon"
            }
        }
        "qt" => "qt",
        "pt" => "pt",
        "cup" => {
            if plural {
                "cups"
            } else {
                "cup"
            }
        }
        "fl_oz" => "fl oz",
        "tbsp" => "tbsp",
        "tsp" => "tsp",

        // Speed
        "mph" => "mph",
        "km/h" => "km/h",
        "m/s" => "m/s",
        "knot" => {
            if plural {
                "knots"
            } else {
                "knot"
            }
        }

        // Area
        "m²" => "m²",
        "ft²" => "ft²",
        "mi²" => "mi²",
        "km²" => "km²",
        "ha" => "ha",
        "acre" => {
            if plural {
                "acres"
            } else {
                "acre"
            }
        }

        // Data
        "B" => {
            if plural {
                "bytes"
            } else {
                "byte"
            }
        }
        "KB" => "KB",
        "MB" => "MB",
        "GB" => "GB",
        "TB" => "TB",

        // Time
        "s" => "s",
        "min" => "min",
        "h" => "h",
        "d" => {
            if plural {
                "days"
            } else {
                "day"
            }
        }
        "wk" => {
            if plural {
                "weeks"
            } else {
                "week"
            }
        }

        // Energy
        "J" => "J",
        "kJ" => "kJ",
        "cal" => "cal",
        "kcal" => "kcal",
        "kWh" => "kWh",

        _ => unit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_miles_to_km() {
        let r = try_convert("5 miles to km").unwrap();
        assert_eq!(r.title, "8.04672 km");
    }

    #[test]
    fn test_km_to_miles() {
        let r = try_convert("10 km to miles").unwrap();
        assert!(r.title.starts_with("6.21"));
        assert!(r.title.ends_with("miles"));
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        let r = try_convert("72°F to C").unwrap();
        assert!(r.title.starts_with("22.2"));
        assert!(r.title.ends_with("°C"));
    }

    #[test]
    fn test_celsius_to_fahrenheit() {
        let r = try_convert("100 celsius to fahrenheit").unwrap();
        assert_eq!(r.title, "212 °F");
    }

    #[test]
    fn test_pounds_to_kg() {
        let r = try_convert("150 lbs to kg").unwrap();
        assert!(r.title.starts_with("68.0"));
        assert!(r.title.ends_with("kg"));
    }

    #[test]
    fn test_gallons_to_liters() {
        let r = try_convert("1 gallon to liters").unwrap();
        assert!(r.title.starts_with("3.78"));
        assert!(r.title.ends_with("L"));
    }

    #[test]
    fn test_inches_to_cm() {
        let r = try_convert("12 inches to cm").unwrap();
        assert!(r.title.starts_with("30.48"));
    }

    #[test]
    fn test_feet_to_meters() {
        let r = try_convert("6 ft to m").unwrap();
        assert!(r.title.starts_with("1.8288"));
    }

    #[test]
    fn test_mph_to_kmh() {
        let r = try_convert("60 mph to km/h").unwrap();
        assert!(r.title.starts_with("96.56"));
    }

    #[test]
    fn test_gb_to_mb() {
        let r = try_convert("2 GB to MB").unwrap();
        assert_eq!(r.title, "2048 MB");
    }

    #[test]
    fn test_hours_to_minutes() {
        let r = try_convert("2.5 hours to minutes").unwrap();
        assert_eq!(r.title, "150 min");
    }

    #[test]
    fn test_kcal_to_kj() {
        let r = try_convert("100 kcal to kj").unwrap();
        assert_eq!(r.title, "418.4 kJ");
    }

    #[test]
    fn test_celsius_to_kelvin() {
        let r = try_convert("0 C to K").unwrap();
        assert_eq!(r.title, "273.15 K");
    }

    #[test]
    fn test_sqft_to_sqm() {
        let r = try_convert("1000 sqft to sqm").unwrap();
        assert!(r.title.starts_with("92.90"));
    }

    #[test]
    fn test_in_keyword() {
        let r = try_convert("5 miles in km").unwrap();
        assert_eq!(r.title, "8.04672 km");
    }

    #[test]
    fn test_incompatible_units() {
        assert!(try_convert("5 miles to kg").is_none());
    }

    #[test]
    fn test_invalid_input() {
        assert!(try_convert("hello to world").is_none());
        assert!(try_convert("").is_none());
        assert!(try_convert("5 to km").is_none());
    }

    #[test]
    fn test_negative_temperature() {
        let r = try_convert("-40 C to F").unwrap();
        assert_eq!(r.title, "-40 °F");
    }

    #[test]
    fn test_hectares_to_acres() {
        let r = try_convert("1 ha to acres").unwrap();
        assert!(r.title.starts_with("2.47"));
    }
}
