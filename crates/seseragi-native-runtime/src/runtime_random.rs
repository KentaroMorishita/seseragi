#[derive(Default)]
struct NativeRandom {
    state: Option<[u64; 4]>,
}

impl NativeRandom {
    fn seeded(mut seed: u64) -> Self {
        let state = std::array::from_fn(|_| {
            seed = seed.wrapping_add(0x9e3779b97f4a7c15);
            let mut value = seed;
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
            value ^ (value >> 31)
        });
        Self { state: Some(state) }
    }

    fn next(&mut self) -> u64 {
        if self.state.is_none() {
            let seed = match std::env::var("SESERAGI_RANDOM_SEED") {
                Ok(text) => text.parse::<i64>().unwrap_or_else(|_| {
                    panic!("SESERAGI_RANDOM_SEED must be a signed 64-bit integer")
                }) as u64,
                Err(std::env::VarError::NotPresent) => {
                    native_entropy_hash_seed().unwrap_or_else(|_| {
                        panic!("secure entropy is unavailable for the Random seed")
                    })
                }
                Err(_) => panic!("SESERAGI_RANDOM_SEED must be a signed 64-bit integer"),
            };
            *self = Self::seeded(seed);
        }
        let state = self.state.as_mut().unwrap();
        let output = state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let shifted = state[1] << 17;
        state[2] ^= state[0];
        state[3] ^= state[1];
        state[1] ^= state[2];
        state[0] ^= state[3];
        state[2] ^= shifted;
        state[3] = state[3].rotate_left(45);
        output
    }

    fn sample(&mut self, width: u64) -> u64 {
        native_random_sample(width, || self.next())
    }

    fn unit_float(&mut self) -> f64 {
        (self.next() >> 11) as f64 / 9_007_199_254_740_992.0
    }
}

fn native_random_sample(width: u64, mut next: impl FnMut() -> u64) -> u64 {
    assert!(width > 0, "Random sample length must be positive");
    let space = 1_u128 << 64;
    let limit = space - space % u128::from(width);
    loop {
        let output = next();
        if u128::from(output) < limit {
            return output % width;
        }
    }
}

fn native_random_run(captures: &[V], context: &mut NativeContext) -> NativeOutcome {
    let name = native_text(&captures[0]);
    let arguments = &captures[1..];
    if let Some(environment) = &context.test {
        return native_test_random(name, arguments, &mut environment.borrow_mut());
    }
    match (name, arguments) {
        ("std/random::algorithmId", [V::Unit]) => Ok(V::text("seseragi-xoshiro256ss-v1")),
        ("std/random::nextBool", [V::Unit]) => {
            Ok(V::Bool(context.random.borrow_mut().next() & 1 == 1))
        }
        ("std/random::nextInt", [V::Unit]) => {
            let value = (context.random.borrow_mut().next() >> 11) as i64;
            Ok(V::Int(if value >= 1_i64 << 52 {
                value - (1_i64 << 53)
            } else {
                value
            }))
        }
        ("std/random::unitFloat", [V::Unit]) => {
            Ok(V::Float(context.random.borrow_mut().unit_float()))
        }
        ("std/random::intBetween", [V::Int(lower), V::Int(upper)]) => {
            checked_int_value(*lower);
            checked_int_value(*upper);
            if lower >= upper {
                return Err(NativeExit::failure(V::adt(
                    "std/random::EmptyRandomIntRange",
                    V::Record(Shared::new(BTreeMap::from([
                        ("lower".into(), V::Int(*lower)),
                        ("upperExclusive".into(), V::Int(*upper)),
                    ]))),
                )));
            }
            let offset = context.random.borrow_mut().sample((upper - lower) as u64);
            Ok(V::Int(lower + offset as i64))
        }
        ("std/random::chance", [V::Float(probability)]) => {
            if !(0.0..=1.0).contains(probability) {
                return Err(NativeExit::failure(V::adt(
                    "std/random::InvalidProbability",
                    V::Float(*probability),
                )));
            }
            Ok(V::Bool(if *probability == 0.0 {
                false
            } else if *probability == 1.0 {
                true
            } else {
                context.random.borrow_mut().unit_float() < *probability
            }))
        }
        ("std/random::randomBytes", [size]) => {
            assert!(
                size.tagged("std/random::RandomSize"),
                "Rust ABI: expected RandomSize"
            );
            let V::Int(size) = size.payload() else {
                panic!("Rust ABI: invalid RandomSize")
            };
            assert!(
                (1..=1_048_576).contains(&size),
                "Random byte size is outside the standard range"
            );
            let mut random = context.random.borrow_mut();
            let mut bytes = vec![0; size as usize];
            for chunk in bytes.chunks_mut(8) {
                chunk.copy_from_slice(&random.next().to_le_bytes()[..chunk.len()]);
            }
            Ok(V::Bytes(Shared::new(bytes)))
        }
        ("std/random::choose", [values]) => {
            let entries = native_non_empty_values(values);
            let index = context.random.borrow_mut().sample(entries.len() as u64) as usize;
            Ok(entries[index].clone())
        }
        ("std/random::shuffle", [V::Array(values)]) => {
            let mut entries = values.get().clone();
            let mut random = context.random.borrow_mut();
            for index in (1..entries.len()).rev() {
                let swap = random.sample((index + 1) as u64) as usize;
                entries.swap(index, swap);
            }
            Ok(V::Array(Shared::new(entries)))
        }
        _ => panic!("Rust ABI: unadmitted Random operation"),
    }
}

pub fn native_random_operation(name: &str, mut arguments: Vec<V>) -> V {
    if name == "std/random::randomSize" {
        let [V::Int(size)] = arguments.as_slice() else {
            panic!("Rust ABI: expected Random size Int")
        };
        let size = checked_int_value(*size);
        return if size <= 0 {
            native_left(V::adt("std/random::NonPositiveRandomSize", V::Int(size)))
        } else if size > 1_048_576 {
            native_left(V::adt("std/random::RandomSizeTooLarge", V::Int(size)))
        } else {
            native_right(V::adt("std/random::RandomSize", V::Int(size)))
        };
    }
    arguments.insert(0, V::text(name));
    V::leaf(arguments, native_random_run)
}

#[cfg(test)]
mod random_tests {
    use super::*;

    fn context(seed: u64) -> NativeContext {
        NativeContext {
            random: Rc::new(RefCell::new(NativeRandom::seeded(seed))),
            gc_every: 1,
            ..NativeContext::default()
        }
    }

    fn run(context: &mut NativeContext, name: &str, arguments: Vec<V>) -> V {
        context
            .root(native_random_operation(name, arguments))
            .unwrap_or_else(|_| panic!("{name}"))
    }

    #[test]
    fn seed_vectors_and_repeated_cold_plans_keep_one_execution_state() {
        let mut context = context(42);
        let plan = native_random_operation("std/random::nextInt", vec![V::Unit]);
        assert_eq!(
            context.random.borrow().state,
            NativeRandom::seeded(42).state
        );
        for expected in [755370490430936, 3413550631330343] {
            let value = context.root(plan.clone()).unwrap_or_else(|_| panic!());
            assert!(matches!(value, V::Int(actual) if actual == expected));
        }
        let mut separate = self::context(42);
        assert_eq!(
            run(&mut separate, "std/random::nextInt", vec![V::Unit]).render(),
            "755370490430936"
        );
        drop(plan);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn invalid_arguments_and_constant_operations_leave_the_lazy_seed_unread() {
        let mut context = NativeContext::default();
        assert_eq!(
            native_text(&run(&mut context, "std/random::algorithmId", vec![V::Unit])),
            "seseragi-xoshiro256ss-v1"
        );
        for (lower, upper) in [
            (0, 0),
            (4, -2),
            (9_007_199_254_740_991, -9_007_199_254_740_991),
        ] {
            let Err(NativeExit::Failure(error)) = context.root(native_random_operation(
                "std/random::intBetween",
                vec![V::Int(lower), V::Int(upper)],
            )) else {
                panic!()
            };
            assert!(error.value.tagged("std/random::EmptyRandomIntRange"));
            assert_eq!(
                error.value.payload().field("lower").render(),
                lower.to_string()
            );
            assert_eq!(
                error.value.payload().field("upperExclusive").render(),
                upper.to_string()
            );
        }
        for probability in [-0.1, 1.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let Err(NativeExit::Failure(error)) = context.root(native_random_operation(
                "std/random::chance",
                vec![V::Float(probability)],
            )) else {
                panic!()
            };
            assert!(error.value.tagged("std/random::InvalidProbability"));
            let V::Float(actual) = error.value.payload() else {
                panic!()
            };
            assert_eq!(actual.to_bits(), probability.to_bits());
        }
        for probability in [0.0, -0.0, 1.0] {
            assert_eq!(
                run(
                    &mut context,
                    "std/random::chance",
                    vec![V::Float(probability)]
                )
                .boolean(),
                probability == 1.0
            );
        }
        for values in [vec![], vec![V::Int(1)]] {
            let input = V::Array(Shared::new(values));
            assert_eq!(
                run(&mut context, "std/random::shuffle", vec![input.clone()]).render(),
                input.render()
            );
        }
        assert!(context.random.borrow().state.is_none());
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn sizes_keep_typed_errors_and_bytes_keep_little_endian_partial_words() {
        for (size, tag) in [
            (-9_007_199_254_740_991, "std/random::NonPositiveRandomSize"),
            (-1, "std/random::NonPositiveRandomSize"),
            (0, "std/random::NonPositiveRandomSize"),
            (1_048_577, "std/random::RandomSizeTooLarge"),
            (9_007_199_254_740_991, "std/random::RandomSizeTooLarge"),
        ] {
            let error = native_random_operation("std/random::randomSize", vec![V::Int(size)]);
            assert!(error.tagged("std/prelude::Left"));
            assert!(error.payload().tagged(tag));
            assert_eq!(error.payload().payload().render(), size.to_string());
        }
        for size in [1, 7, 8, 9, 16, 17, 1_048_576] {
            let mut context = context(42);
            let configured = native_random_operation("std/random::randomSize", vec![V::Int(size)]);
            assert!(configured.tagged("std/prelude::Right"));
            let bytes = run(
                &mut context,
                "std/random::randomBytes",
                vec![configured.payload()],
            );
            let mut reference = NativeRandom::seeded(42);
            let expected = (0..(size + 7) / 8)
                .flat_map(|_| reference.next().to_le_bytes())
                .take(size as usize)
                .collect::<Vec<_>>();
            assert_eq!(native_bytes(&bytes), expected);
            assert_eq!(context.random.borrow_mut().next(), reference.next());
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn rejection_sampling_rejects_the_upper_tail_and_consumes_singletons() {
        for width in [1, 2, 3, 7, 1_u64 << 53, 18_014_398_509_481_982] {
            let mut calls = 0;
            let result = native_random_sample(width, || {
                calls += 1;
                if calls == 1 {
                    u64::MAX
                } else {
                    42
                }
            });
            if (1_u128 << 64) % u128::from(width) == 0 {
                assert_eq!(calls, 1);
                assert_eq!(result, u64::MAX % width);
            } else {
                assert_eq!(calls, 2);
                assert_eq!(result, 42 % width);
            }
        }
    }

    #[test]
    fn collection_results_retain_values_preserve_inputs_and_release_cycles() {
        enable_cycle_collection();
        let mut context = context(42);
        let reference = V::reference(V::Unit);
        let callback = V::fun(vec![reference.clone()], |c, _| c[0].read_reference());
        reference.write_reference(callback.clone());
        let input = V::Array(Shared::new(vec![callback.clone(), callback.clone()]));
        let shuffled = run(&mut context, "std/random::shuffle", vec![input.clone()]);
        let chosen = run(
            &mut context,
            "std/random::choose",
            vec![native_non_empty(vec![callback])],
        );
        assert_eq!(input.index(0).apply(V::Unit).render(), "<function>");
        drop((input, reference));
        collect_cycles().unwrap();
        assert_eq!(shuffled.index(1).apply(V::Unit).render(), "<function>");
        assert_eq!(chosen.apply(V::Unit).render(), "<function>");
        drop((shuffled, chosen));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn forked_fibers_advance_the_same_random_state() {
        let mut context = context(42);
        let plan = native_random_operation("std/random::nextInt", vec![V::Unit]);
        let fork = native_operation("std/effect::fork", vec![plan]);
        let joined = V::bind(
            fork,
            V::fun(vec![], |_, child| {
                native_operation("std/effect::join", vec![child])
            }),
        );
        assert_eq!(
            context.root(joined).unwrap_or_else(|_| panic!()).render(),
            "755370490430936"
        );
        assert_eq!(
            run(&mut context, "std/random::nextInt", vec![V::Unit]).render(),
            "3413550631330343"
        );
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
