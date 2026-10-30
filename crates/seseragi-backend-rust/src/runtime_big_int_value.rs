struct NativeBigInt {
    negative: bool,
    // Canonical unsigned magnitude, little-endian base 2^32. Zero has one word.
    limbs: Vec<u32>,
    decimal: std::cell::OnceCell<String>,
}

impl TraceValue for NativeBigInt {
    fn edges(&self, _: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        Ok(())
    }
}

impl NativeBigInt {
    fn from_limbs(negative: bool, mut limbs: Vec<u32>) -> Self {
        while limbs.len() > 1 && limbs.last() == Some(&0) {
            limbs.pop();
        }
        if limbs.is_empty() {
            limbs.push(0);
        }
        Self {
            negative: negative && !(limbs.len() == 1 && limbs[0] == 0),
            limbs,
            decimal: std::cell::OnceCell::new(),
        }
    }

    fn from_i64(value: i64) -> Self {
        let magnitude = value.unsigned_abs();
        Self::from_limbs(value < 0, vec![magnitude as u32, (magnitude >> 32) as u32])
    }

    fn is_zero(&self) -> bool {
        self.limbs.len() == 1 && self.limbs[0] == 0
    }

    fn radix_group(radix: u32) -> (usize, u32) {
        assert!((2..=36).contains(&radix), "Rust ABI: invalid BigInt radix");
        let (mut width, mut factor) = (0, 1_u32);
        while factor <= u32::MAX / radix {
            factor *= radix;
            width += 1;
        }
        (width, factor)
    }

    fn from_digits(negative: bool, digits: &[u8], radix: u32) -> Self {
        let (width, _) = Self::radix_group(radix);
        let mut limbs = vec![0_u32];
        for chunk in digits.chunks(width) {
            let (mut factor, mut carry) = (1_u64, 0_u64);
            for byte in chunk {
                factor *= u64::from(radix);
                carry = carry * u64::from(radix)
                    + u64::from(native_big_int_digit(*byte).expect("validated BigInt digit"));
            }
            for limb in &mut limbs {
                let value = u64::from(*limb) * factor + carry;
                *limb = value as u32;
                carry = value >> 32;
            }
            if carry != 0 {
                limbs.push(carry as u32);
            }
        }
        let value = Self::from_limbs(negative, limbs);
        if radix == 10 {
            let digits = std::str::from_utf8(digits)
                .expect("ASCII decimal digits")
                .trim_start_matches('0');
            let canonical = if digits.is_empty() {
                "0".into()
            } else if value.negative {
                format!("-{digits}")
            } else {
                digits.into()
            };
            value.decimal.set(canonical).expect("new decimal cache");
        }
        value
    }

    fn format_uncached(&self, radix: u32) -> String {
        if self.is_zero() {
            return "0".into();
        }
        let (width, divisor) = Self::radix_group(radix);
        let mut limbs = self.limbs.clone();
        let mut output = Vec::new();
        while !(limbs.len() == 1 && limbs[0] == 0) {
            let mut remainder = 0_u64;
            for limb in limbs.iter_mut().rev() {
                let value = (remainder << 32) | u64::from(*limb);
                *limb = (value / u64::from(divisor)) as u32;
                remainder = value % u64::from(divisor);
            }
            while limbs.len() > 1 && limbs.last() == Some(&0) {
                limbs.pop();
            }
            let last = limbs.len() == 1 && limbs[0] == 0;
            for _ in 0..width {
                output.push(
                    b"0123456789abcdefghijklmnopqrstuvwxyz"
                        [(remainder % u64::from(radix)) as usize],
                );
                remainder /= u64::from(radix);
                if last && remainder == 0 {
                    break;
                }
            }
        }
        if self.negative {
            output.push(b'-');
        }
        output.reverse();
        String::from_utf8(output).expect("ASCII BigInt output")
    }

    fn decimal(&self) -> &str {
        self.decimal.get_or_init(|| self.format_uncached(10))
    }

    fn format_radix(&self, radix: u32) -> String {
        if radix == 10 {
            self.decimal().to_owned()
        } else {
            self.format_uncached(radix)
        }
    }

    fn to_safe_int(&self) -> Option<i64> {
        if self.limbs.len() > 2 {
            return None;
        }
        let magnitude =
            u64::from(self.limbs[0]) | (u64::from(*self.limbs.get(1).unwrap_or(&0)) << 32);
        if magnitude > 9_007_199_254_740_991 {
            return None;
        }
        Some(if self.negative {
            -(magnitude as i64)
        } else {
            magnitude as i64
        })
    }

    fn equal(&self, other: &Self) -> bool {
        self.negative == other.negative && self.limbs == other.limbs
    }

    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        if self.negative != other.negative {
            return if self.negative {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        let magnitude = self
            .limbs
            .len()
            .cmp(&other.limbs.len())
            .then_with(|| self.limbs.iter().rev().cmp(other.limbs.iter().rev()));
        if self.negative {
            magnitude.reverse()
        } else {
            magnitude
        }
    }

    fn hash(&self) -> i64 {
        let mut state = (0x811c9dc5_u32 ^ u32::from(self.negative)).wrapping_mul(0x01000193);
        for limb in &self.limbs {
            state = (state ^ limb).wrapping_mul(0x01000193);
        }
        i64::from(state as i32)
    }
}
