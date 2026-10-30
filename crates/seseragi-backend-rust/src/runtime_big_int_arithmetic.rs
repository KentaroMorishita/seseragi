impl NativeBigInt {
    fn magnitude_order(left: &[u32], right: &[u32]) -> std::cmp::Ordering {
        left.len()
            .cmp(&right.len())
            .then_with(|| left.iter().rev().cmp(right.iter().rev()))
    }

    fn add_magnitudes(left: &[u32], right: &[u32]) -> Vec<u32> {
        let length = left.len().max(right.len());
        let mut result =
            Vec::with_capacity(length.checked_add(1).expect("BigInt allocation overflow"));
        let mut carry = 0_u64;
        for index in 0..length {
            let value = u64::from(*left.get(index).unwrap_or(&0))
                + u64::from(*right.get(index).unwrap_or(&0))
                + carry;
            result.push(value as u32);
            carry = value >> 32;
        }
        if carry != 0 {
            result.push(carry as u32);
        }
        result
    }

    // left >= right. Subtraction and normalization each visit a word once.
    fn subtract_magnitudes(left: &mut Vec<u32>, right: &[u32]) {
        let mut borrow = 0_i64;
        for (index, word) in left.iter_mut().enumerate() {
            let value = i64::from(*word) - i64::from(*right.get(index).unwrap_or(&0)) - borrow;
            *word = value as u32;
            borrow = i64::from(value < 0);
        }
        assert_eq!(borrow, 0, "Rust ABI: negative BigInt magnitude difference");
        while left.len() > 1 && left.last() == Some(&0) {
            left.pop();
        }
    }

    fn add(&self, right: &Self) -> Self {
        if self.negative == right.negative {
            return Self::from_limbs(
                self.negative,
                Self::add_magnitudes(&self.limbs, &right.limbs),
            );
        }
        let (larger, smaller) = if Self::magnitude_order(&self.limbs, &right.limbs).is_lt() {
            (right, self)
        } else {
            (self, right)
        };
        let mut result = larger.limbs.clone();
        Self::subtract_magnitudes(&mut result, &smaller.limbs);
        Self::from_limbs(larger.negative, result)
    }

    fn subtract(&self, right: &Self) -> Self {
        self.add(&Self::from_limbs(!right.negative, right.limbs.clone()))
    }

    fn multiply(&self, right: &Self) -> Self {
        if self.is_zero() || right.is_zero() {
            return Self::from_i64(0);
        }
        let length = self
            .limbs
            .len()
            .checked_add(right.limbs.len())
            .expect("BigInt allocation overflow");
        let mut result = vec![0_u32; length];
        for (i, left) in self.limbs.iter().enumerate() {
            let mut carry = 0_u64;
            for (j, right) in right.limbs.iter().enumerate() {
                let value = u64::from(*left) * u64::from(*right) + u64::from(result[i + j]) + carry;
                result[i + j] = value as u32;
                carry = value >> 32;
            }
            result[i + right.limbs.len()] = carry as u32;
        }
        Self::from_limbs(self.negative != right.negative, result)
    }

    fn quotient_remainder(&self, divisor: &Self) -> (Self, Self) {
        assert!(!divisor.is_zero(), "BigInt division by zero");
        if Self::magnitude_order(&self.limbs, &divisor.limbs).is_lt() {
            return (
                Self::from_i64(0),
                Self::from_limbs(self.negative, self.limbs.clone()),
            );
        }
        let mut quotient = vec![0_u32; self.limbs.len()];
        let mut remainder = vec![0_u32];
        // Restoring division: exactly 32 steps per dividend word. The
        // remainder is shorter than the divisor before each shift, so this
        // is O(n*m) word operations and never converts through host Float.
        for index in (0..self.limbs.len()).rev() {
            for bit in (0..32).rev() {
                let mut carry = u64::from((self.limbs[index] >> bit) & 1);
                for word in &mut remainder {
                    let value = (u64::from(*word) << 1) | carry;
                    *word = value as u32;
                    carry = value >> 32;
                }
                if carry != 0 {
                    remainder.push(carry as u32);
                }
                if !Self::magnitude_order(&remainder, &divisor.limbs).is_lt() {
                    Self::subtract_magnitudes(&mut remainder, &divisor.limbs);
                    quotient[index] |= 1_u32 << bit;
                }
            }
        }
        (
            Self::from_limbs(self.negative != divisor.negative, quotient),
            Self::from_limbs(self.negative, remainder),
        )
    }

    fn power(&self, exponent: i64) -> Self {
        let exponent = checked_int_value(exponent);
        assert!(exponent >= 0, "BigInt negative exponent");
        let mut remaining = exponent as u64;
        let mut result = Self::from_i64(1);
        let mut factor = Self::from_limbs(self.negative, self.limbs.clone());
        while remaining != 0 {
            if remaining & 1 != 0 {
                result = result.multiply(&factor);
            }
            remaining >>= 1;
            if remaining != 0 {
                factor = factor.multiply(&factor);
            }
        }
        result
    }
}

fn native_big_int_arithmetic(operator: &str, left: V, right: V) -> V {
    let V::BigInt(left) = left else {
        panic!("Rust ABI: expected BigInt left operand")
    };
    if operator == "**" {
        let V::Int(exponent) = right else {
            panic!("Rust ABI: expected BigInt Int exponent")
        };
        return V::BigInt(Shared::new(left.get().power(exponent)));
    }
    let V::BigInt(right) = right else {
        panic!("Rust ABI: expected BigInt right operand")
    };
    let (left, right) = (left.get(), right.get());
    let value = match operator {
        "+" => left.add(right),
        "-" => left.subtract(right),
        "*" => left.multiply(right),
        "/" => {
            assert!(!right.is_zero(), "BigInt division by zero");
            left.quotient_remainder(right).0
        }
        "%" => {
            assert!(!right.is_zero(), "BigInt remainder by zero");
            left.quotient_remainder(right).1
        }
        _ => panic!("Rust ABI: unsupported BigInt arithmetic operator"),
    };
    V::BigInt(Shared::new(value))
}

fn native_big_int_arithmetic_dictionary(identity: &'static str) -> V {
    let (method, operator) = match identity {
        "std/big-int::Add" => ("add", "+"),
        "std/big-int::Sub" => ("sub", "-"),
        "std/big-int::Mul" => ("mul", "*"),
        "std/big-int::Div" => ("div", "/"),
        "std/big-int::Rem" => ("rem", "%"),
        "std/big-int::Pow" => ("pow", "**"),
        _ => panic!("Rust ABI: unsupported BigInt arithmetic dictionary"),
    };
    V::dict(vec![(
        method,
        V::fun(vec![V::text(operator)], |captures, left| {
            V::fun(vec![captures[0].clone(), left], |captures, right| {
                native_big_int_arithmetic(native_text(&captures[0]), captures[1].clone(), right)
            })
        }),
    )])
}

fn native_big_int_constant_dictionary(one: bool) -> V {
    V::dict(vec![(
        if one { "one" } else { "zero" },
        V::fun(vec![V::Bool(one)], |captures, unit| {
            assert!(
                matches!(unit, V::Unit),
                "Rust ABI: expected BigInt constant Unit"
            );
            V::BigInt(Shared::new(NativeBigInt::from_i64(i64::from(
                captures[0].boolean(),
            ))))
        }),
    )])
}

#[cfg(test)]
mod big_int_arithmetic_tests {
    use super::*;

    #[test]
    fn signed_arithmetic_matches_independent_i128_results_at_word_boundaries() {
        let values = [
            i64::MIN,
            -9_007_199_254_740_991,
            -4_294_967_296,
            -1,
            0,
            1,
            4_294_967_295,
            4_294_967_296,
            9_007_199_254_740_991,
            i64::MAX,
        ];
        for left in values {
            for right in values {
                let (a, b) = (NativeBigInt::from_i64(left), NativeBigInt::from_i64(right));
                let (x, y) = (i128::from(left), i128::from(right));
                assert_eq!(a.add(&b).decimal(), (x + y).to_string());
                assert_eq!(a.subtract(&b).decimal(), (x - y).to_string());
                assert_eq!(a.multiply(&b).decimal(), (x * y).to_string());
                if right != 0 {
                    let (quotient, remainder) = a.quotient_remainder(&b);
                    assert_eq!(quotient.decimal(), (x / y).to_string());
                    assert_eq!(remainder.decimal(), (x % y).to_string());
                    assert!(quotient.multiply(&b).add(&remainder).equal(&a));
                }
            }
        }
    }

    #[test]
    fn multiword_division_matches_python_vectors_and_truncates_toward_zero() {
        // Expected values were computed independently with Python integers.
        for left_negative in [false, true] {
            for right_negative in [false, true] {
                let a = NativeBigInt::from_digits(left_negative, b"231584178474632390847141970017375815707220534065123005005841917230689795715129", 10);
                let b =
                    NativeBigInt::from_digits(right_negative, b"79228162514264337593543950433", 10);
                let (q, r) = a.quotient_remainder(&b);
                let expected_q = if left_negative != right_negative {
                    "-2923003274661805836407369661853897689020802007040"
                } else {
                    "2923003274661805836407369661853897689020802007040"
                };
                let expected_r = if left_negative {
                    "-347130829978233118666809"
                } else {
                    "347130829978233118666809"
                };
                assert_eq!(q.decimal(), expected_q);
                assert_eq!(r.decimal(), expected_r);
                assert!(q.multiply(&b).add(&r).equal(&a));
                assert!(NativeBigInt::magnitude_order(&r.limbs, &b.limbs).is_lt());
            }
        }
    }

    #[test]
    fn power_is_exact_and_keeps_logarithmic_safe_int_exponents_and_typed_errors() {
        assert_eq!(NativeBigInt::from_i64(0).power(0).decimal(), "1");
        assert_eq!(
            NativeBigInt::from_i64(-2).power(129).decimal(),
            "-680564733841876926926749214863536422912"
        );
        for (base, exponent, expected) in [
            (-1, 9_007_199_254_740_991, "-1"),
            (-1, 9_007_199_254_740_990, "1"),
            (0, 9_007_199_254_740_991, "0"),
        ] {
            assert_eq!(
                NativeBigInt::from_i64(base).power(exponent).decimal(),
                expected
            );
        }
        {
            let zero = native_big_int_constant_dictionary(false)
                .field("zero")
                .apply(V::Unit);
            let one = native_big_int_constant_dictionary(true)
                .field("one")
                .apply(V::Unit);
            assert_eq!(zero.render(), "0");
            assert_eq!(one.render(), "1");
            for operation in [
                "std/big-int::checkedDivide",
                "std/big-int::checkedRemainder",
            ] {
                let error =
                    native_big_int_standard_operation(operation, vec![zero.clone(), one.clone()]);
                assert!(error.tagged("std/prelude::Left"));
                assert!(error.payload().tagged("std/big-int::BigIntDivisionByZero"));
            }
            let error = native_big_int_standard_operation(
                "std/big-int::checkedPower",
                vec![V::Int(-2), one.clone()],
            );
            assert!(error.tagged("std/prelude::Left"));
            assert!(error
                .payload()
                .tagged("std/big-int::NegativeBigIntExponent"));
            assert!(matches!(error.payload().payload(), V::Int(-2)));
        }
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn saved_arithmetic_methods_keep_explicit_managed_captures_and_collect_cycles() {
        enable_cycle_collection();
        let left = native_big_int_parse("4294967296").payload();
        let cell = V::reference(left.clone());
        let add = native_big_int_arithmetic_dictionary("std/big-int::Add")
            .field("add")
            .apply(left.clone());
        assert_eq!(add.apply(left.clone()).render(), "8589934592");
        let capture = V::fun(vec![cell.clone(), add.clone()], |c, _| {
            c[1].apply(c[0].read_reference())
        });
        cell.write_reference(V::Tuple(Shared::new(vec![
            cell.clone(),
            left.clone(),
            capture.clone(),
        ])));
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop((left, cell, add, capture));
        assert!(collect_cycles().unwrap().reclaimed > 0);
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
