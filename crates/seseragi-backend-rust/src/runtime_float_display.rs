// Seseragi Float spelling: shortest binary64 round trip, ties to an even
// decimal significand, fixed notation in [1e-6, 1e21), and an explicit .0.
// Rust's shortest formatter chooses the other candidate on decimal midpoints.
// Compare that midpoint as an exact odd integer times a power of two; never
// decide a decimal tie by parsing it back into an already-rounded f64.
fn float_decimal_midpoint(value: f64, significand: u64, power: i32) -> bool {
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let mantissa = (bits & ((1_u64 << 52) - 1)) | if exponent == 0 { 0 } else { 1_u64 << 52 };
    let zeros = mantissa.trailing_zeros();
    let binary_power = if exponent == 0 {
        -1074
    } else {
        exponent - 1075
    } + zeros as i32;
    if binary_power != power - 1 {
        return false;
    }
    let mut decimal_odd = 2 * significand - 1;
    for _ in 0..power.max(0) {
        let Some(next) = decimal_odd.checked_mul(5) else {
            return false;
        };
        decimal_odd = next;
    }
    for _ in power.min(0)..0 {
        if decimal_odd % 5 != 0 {
            return false;
        }
        decimal_odd /= 5;
    }
    decimal_odd == mantissa >> zeros
}

fn native_float_format(value: f64) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value == f64::INFINITY {
        return "Infinity".into();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".into();
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0.0"
        } else {
            "0.0"
        }
        .into();
    }
    let shortest = format!("{:e}", value.abs());
    let (coefficient, exponent) = shortest.split_once('e').expect("binary64 exponent");
    let exponent = exponent.parse::<i32>().expect("binary64 exponent");
    let digits = coefficient.replace('.', "");
    let mut significand = digits.parse::<u64>().expect("binary64 significand");
    let power = exponent - digits.len() as i32 + 1;
    if significand % 2 == 1 && float_decimal_midpoint(value.abs(), significand, power)
        // At powers of two the round-trip interval is asymmetric. The even
        // decimal candidate is eligible only if it still denotes this value.
        && format!("{}e{power}", significand - 1).parse::<f64>()
            .is_ok_and(|candidate| candidate.to_bits() == value.abs().to_bits())
    {
        significand -= 1;
    }
    let digits = significand.to_string();
    let digits = digits.trim_end_matches('0');
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if !(-6..21).contains(&exponent) {
        let tail = if digits.len() > 1 {
            format!(".{}", &digits[1..])
        } else {
            String::new()
        };
        return format!("{sign}{}{tail}e{exponent}", &digits[..1]);
    }
    let point = exponent + 1;
    if point <= 0 {
        format!("{sign}0.{}{digits}", "0".repeat((-point) as usize))
    } else if point as usize >= digits.len() {
        format!(
            "{sign}{digits}{}.0",
            "0".repeat(point as usize - digits.len())
        )
    } else {
        format!(
            "{sign}{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    }
}

#[cfg(test)]
mod float_display_tests {
    use super::*;

    #[test]
    fn canonical_float_special_values_and_decimal_boundaries() {
        for (value, expected) in [
            (f64::NAN, "NaN"),
            (f64::INFINITY, "Infinity"),
            (f64::NEG_INFINITY, "-Infinity"),
            (0.0, "0.0"),
            (-0.0, "-0.0"),
            (1e-6, "0.000001"),
            (1e-7, "1e-7"),
            (1e20, "100000000000000000000.0"),
            (1e21, "1e21"),
            (f64::from_bits(1), "5e-324"),
            (f64::MAX, "1.7976931348623157e308"),
        ] {
            assert_eq!(native_float_format(value), expected);
        }
    }

    #[test]
    fn shortest_decimal_midpoints_choose_even_without_changing_neighbours() {
        for (bits, expected) in [
            (0x43040c2d2010d68a, "705360935656145.2"),
            (0x3e60000000000000, "2.9802322387695312e-8"),
            (0xc2ed3e97ef9ae674, "-257237393921843.62"),
            (0x4310000000000001, "1125899906842624.2"),
            (0x3e70000000000000, "5.960464477539063e-8"),
        ] {
            assert_eq!(native_float_format(f64::from_bits(bits)), expected);
        }
        for bits in 0x43040c2d2010d680..=0x43040c2d2010d690 {
            let value = f64::from_bits(bits);
            assert_eq!(
                native_float_format(value).parse::<f64>().unwrap().to_bits(),
                bits
            );
        }
    }
}
