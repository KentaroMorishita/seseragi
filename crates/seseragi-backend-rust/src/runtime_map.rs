// Persistent compressed radix indexes mirror runtime/ts/src/persistent-index.ts.
// Safe-integer addresses bound a path to 53 branches. Point updates copy only
// that path; removal promotes the surviving child and retains no tombstones.
#[derive(Clone)]
struct NativeMapEntry {
    key: V,
    value: V,
    hash: u32,
    previous: Option<u64>,
    next: Option<u64>,
}
struct NativeMap {
    seed: u64,
    buckets: V,
    order: V,
    first: Option<u64>,
    last: Option<u64>,
}
struct NativeMapIndex {
    min: u64,
    max: u64,
    size: u64,
    kind: NativeMapIndexKind,
}
enum NativeMapIndexKind {
    Leaf(V),
    Branch { bit: u32, left: V, right: V },
}
impl NativeMap {
    fn empty(seed: u64) -> Self {
        Self {
            seed,
            buckets: V::Unit,
            order: V::Unit,
            first: None,
            last: None,
        }
    }
}
impl TraceValue for NativeMap {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        visit(&self.buckets);
        visit(&self.order);
        Ok(())
    }
}
impl TraceValue for NativeMapEntry {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        visit(&self.key);
        visit(&self.value);
        Ok(())
    }
}
impl TraceValue for NativeMapIndex {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        match &self.kind {
            NativeMapIndexKind::Leaf(value) => visit(value),
            NativeMapIndexKind::Branch { left, right, .. } => {
                visit(left);
                visit(right);
            }
        }
        Ok(())
    }
}
fn native_map_index(index: &V) -> Option<&NativeMapIndex> {
    match index {
        V::Unit => None,
        V::MapIndex(node) => Some(node.get()),
        _ => panic!("Rust ABI: Map index"),
    }
}
fn native_map_index_size(index: &V) -> u64 {
    native_map_index(index).map_or(0, |node| node.size)
}
fn native_map_index_get(mut index: &V, key: u64) -> Option<&V> {
    loop {
        let node = native_map_index(index)?;
        if key < node.min || key > node.max {
            return None;
        }
        match &node.kind {
            NativeMapIndexKind::Leaf(value) => return Some(value),
            NativeMapIndexKind::Branch { bit, left, right } => {
                index = if (key >> bit) & 1 == 0 { left } else { right }
            }
        }
    }
}
fn native_map_index_branch(bit: u32, left: V, right: V) -> V {
    let l = native_map_index(&left).expect("left index");
    let r = native_map_index(&right).expect("right index");
    let (min, max, size) = (l.min, r.max, l.size + r.size);
    V::MapIndex(Shared::new(NativeMapIndex {
        min,
        max,
        size,
        kind: NativeMapIndexKind::Branch { bit, left, right },
    }))
}
fn native_map_index_put(index: &V, leaf: V) -> V {
    let Some(node) = native_map_index(index) else {
        return leaf;
    };
    let incoming = native_map_index(&leaf).expect("incoming leaf");
    if matches!(node.kind, NativeMapIndexKind::Leaf(_)) && node.min == incoming.min {
        return leaf;
    }
    let different = node.min ^ incoming.min;
    let bit = if different == 0 {
        -1
    } else {
        63 - different.leading_zeros() as i32
    };
    let old_bit = match &node.kind {
        NativeMapIndexKind::Leaf(_) => -1,
        NativeMapIndexKind::Branch { bit, .. } => *bit as i32,
    };
    if old_bit < 0 || bit > old_bit {
        let bit = bit as u32;
        return if (incoming.min >> bit) & 1 == 0 {
            native_map_index_branch(bit, leaf, index.clone())
        } else {
            native_map_index_branch(bit, index.clone(), leaf)
        };
    }
    let NativeMapIndexKind::Branch { bit, left, right } = &node.kind else {
        unreachable!()
    };
    if (incoming.min >> bit) & 1 == 0 {
        native_map_index_branch(*bit, native_map_index_put(left, leaf), right.clone())
    } else {
        native_map_index_branch(*bit, left.clone(), native_map_index_put(right, leaf))
    }
}
fn native_map_index_set(index: &V, key: u64, value: V) -> V {
    assert!(
        key <= 9_007_199_254_740_991,
        "Map index address space exhausted"
    );
    native_map_index_put(
        index,
        V::MapIndex(Shared::new(NativeMapIndex {
            min: key,
            max: key,
            size: 1,
            kind: NativeMapIndexKind::Leaf(value),
        })),
    )
}
fn native_map_same_index(left: &V, right: &V) -> bool {
    match (left, right) {
        (V::Unit, V::Unit) => true,
        (V::MapIndex(a), V::MapIndex(b)) => a.id() == b.id(),
        _ => false,
    }
}
fn native_map_index_remove(index: &V, key: u64) -> V {
    let Some(node) = native_map_index(index) else {
        return index.clone();
    };
    if key < node.min || key > node.max {
        return index.clone();
    }
    let NativeMapIndexKind::Branch { bit, left, right } = &node.kind else {
        return V::Unit;
    };
    if (key >> bit) & 1 == 0 {
        let next = native_map_index_remove(left, key);
        if native_map_same_index(left, &next) {
            index.clone()
        } else if matches!(next, V::Unit) {
            right.clone()
        } else {
            native_map_index_branch(*bit, next, right.clone())
        }
    } else {
        let next = native_map_index_remove(right, key);
        if native_map_same_index(right, &next) {
            index.clone()
        } else if matches!(next, V::Unit) {
            left.clone()
        } else {
            native_map_index_branch(*bit, left.clone(), next)
        }
    }
}
fn native_map_index_vacant(index: &V) -> u64 {
    let Some(mut node) = native_map_index(index) else {
        return 0;
    };
    if node.min > 0 {
        return 0;
    }
    while let NativeMapIndexKind::Branch { left, right, .. } = &node.kind {
        if node.size == node.max - node.min + 1 {
            break;
        }
        let l = native_map_index(left).expect("left index");
        let r = native_map_index(right).expect("right index");
        if l.size != l.max - l.min + 1 {
            node = l;
        } else if l.max + 1 < r.min {
            return l.max + 1;
        } else {
            node = r;
        }
    }
    let vacant = node.max + 1;
    assert!(
        vacant <= 9_007_199_254_740_991,
        "Map index address space exhausted"
    );
    vacant
}
fn native_fixed_hash_seed(text: &str) -> Result<u64, &'static str> {
    let digits = text
        .strip_prefix('+')
        .or_else(|| text.strip_prefix('-'))
        .unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|c| c.is_ascii_digit()) {
        return Err("SESERAGI_HASH_SEED must be a signed 64-bit integer");
    }
    text.parse::<i64>()
        .map(|seed| seed as u64)
        .map_err(|_| "SESERAGI_HASH_SEED must be a signed 64-bit integer")
}
#[cfg(unix)]
fn native_entropy_hash_seed() -> std::io::Result<u64> {
    native_read_hash_seed(std::fs::File::open("/dev/urandom")?)
}
#[cfg(any(unix, test))]
fn native_read_hash_seed(mut source: impl std::io::Read) -> std::io::Result<u64> {
    let mut bytes = [0_u8; 8];
    source.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}
#[cfg(windows)]
fn native_entropy_hash_seed() -> std::io::Result<u64> {
    #[link(name = "bcrypt")]
    unsafe extern "system" {
        fn BCryptGenRandom(
            algorithm: *mut std::ffi::c_void,
            buffer: *mut u8,
            length: u32,
            flags: u32,
        ) -> i32;
    }
    let mut bytes = [0_u8; 8];
    // The system-preferred RNG requires a null algorithm handle. The writable
    // buffer remains alive for the synchronous call and exactly matches length.
    let status = unsafe {
        BCryptGenRandom(
            std::ptr::null_mut(),
            bytes.as_mut_ptr(),
            bytes.len() as u32,
            2,
        )
    };
    if status >= 0 {
        Ok(u64::from_le_bytes(bytes))
    } else {
        Err(std::io::Error::other("secure hash entropy is unavailable"))
    }
}
#[cfg(not(any(unix, windows)))]
fn native_entropy_hash_seed() -> std::io::Result<u64> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "secure hash entropy is unavailable",
    ))
}
fn native_process_hash_seed() -> u64 {
    static SEED: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *SEED.get_or_init(|| match std::env::var("SESERAGI_HASH_SEED") {
        Ok(text) => native_fixed_hash_seed(&text).unwrap_or_else(|message| panic!("{message}")),
        Err(std::env::VarError::NotPresent) => native_entropy_hash_seed().unwrap_or_else(|_| {
            panic!("secure entropy is unavailable for the process-local hash seed")
        }),
        Err(_) => panic!("SESERAGI_HASH_SEED must be a signed 64-bit integer"),
    })
}
fn native_mix_hash(hash: i64, seed: u64) -> u32 {
    checked_int_value(hash);
    let bits = hash as u64;
    let folded = (bits ^ (bits >> 32)) as u32;
    let seed = (seed ^ (seed >> 32)) as u32;
    let mut state = (0x811c9dc5_u32 ^ seed).wrapping_mul(0x01000193);
    state = (state ^ folded).wrapping_mul(0x01000193);
    state ^ (state >> 16)
}
fn native_map_dictionaries(value: &V) -> Vec<V> {
    let V::Tuple(values) = value else {
        panic!("Rust ABI: expected Map dictionary frame")
    };
    values.get().clone()
}
fn native_map_value(value: &V) -> &NativeMap {
    let V::Map(value) = value else {
        panic!("Rust ABI: expected Map")
    };
    value.get()
}
fn native_map_entry(values: &NativeMap, id: u64) -> &NativeMapEntry {
    let Some(V::MapEntry(entry)) = native_map_index_get(&values.order, id) else {
        panic!("Rust ABI: Map order link")
    };
    entry.get()
}
fn native_map_bucket(values: &NativeMap, hash: u32) -> &[V] {
    match native_map_index_get(&values.buckets, u64::from(hash)) {
        None => &[],
        Some(V::Array(ids)) => ids.get(),
        _ => panic!("Rust ABI: Map bucket"),
    }
}
fn native_map_address(value: &V) -> u64 {
    let V::Int(value) = value else {
        panic!("Rust ABI: Map address")
    };
    u64::try_from(checked_int_value(*value)).expect("nonnegative Map address")
}
fn native_map_find(values: &NativeMap, key: &V, hash: u32, eq: &V) -> Option<u64> {
    for id in native_map_bucket(values, hash) {
        let id = native_map_address(id);
        if eq
            .apply(native_map_entry(values, id).key.clone())
            .apply(key.clone())
            .boolean()
        {
            return Some(id);
        }
    }
    None
}
fn native_map_key_hash(values: &NativeMap, key: &V, dictionaries: &[V]) -> (V, u32) {
    assert_eq!(dictionaries.len(), 2, "Rust ABI: Map requires Eq/Hash");
    let hash = dictionaries[1].field("hash").apply(key.clone());
    let V::Int(hash) = hash else {
        panic!("Rust ABI: Hash result must be Int")
    };
    (
        dictionaries[0].field("eq"),
        native_mix_hash(hash, values.seed),
    )
}
fn native_map_append(values: &NativeMap, key: V, value: V, hash: u32) -> V {
    let id = native_map_index_vacant(&values.order);
    let mut order = values.order.clone();
    if let Some(last) = values.last {
        let mut previous = native_map_entry(values, last).clone();
        previous.next = Some(id);
        order = native_map_index_set(&order, last, V::MapEntry(Shared::new(previous)));
    }
    order = native_map_index_set(
        &order,
        id,
        V::MapEntry(Shared::new(NativeMapEntry {
            key,
            value,
            hash,
            previous: values.last,
            next: None,
        })),
    );
    let mut bucket = native_map_bucket(values, hash).to_vec();
    bucket.push(V::Int(id as i64));
    let buckets = native_map_index_set(
        &values.buckets,
        u64::from(hash),
        V::Array(Shared::new(bucket)),
    );
    V::Map(Shared::new(NativeMap {
        seed: values.seed,
        buckets,
        order,
        first: values.first.or(Some(id)),
        last: Some(id),
    }))
}

fn native_map_replace(values: &NativeMap, id: u64, value: V) -> V {
    let mut entry = native_map_entry(values, id).clone();
    entry.value = value;
    let order = native_map_index_set(&values.order, id, V::MapEntry(Shared::new(entry)));
    V::Map(Shared::new(NativeMap {
        seed: values.seed,
        buckets: values.buckets.clone(),
        order,
        first: values.first,
        last: values.last,
    }))
}

// The lookup and update share one mixed hash. All original entries and source
// arguments stay owned by the caller while arbitrary user code runs.
fn native_map_upsert(
    values: &NativeMap,
    dictionaries: &[V],
    key: &V,
    update: impl FnOnce(Option<&V>) -> V,
) -> V {
    let (eq, hash) = native_map_key_hash(values, key, dictionaries);
    let found = native_map_find(values, key, hash, &eq);
    let value = update(found.map(|id| &native_map_entry(values, id).value));
    match found {
        Some(id) => native_map_replace(values, id, value),
        None => native_map_append(values, key.clone(), value, hash),
    }
}
// Rebuild only selected entries, retaining their cached hashes and seed. Build
// each collision bucket once; repeated append would copy a large bucket for
// every selected key. No source container or excluded key enters the result.
fn native_map_select(values: &NativeMap, mut keep: impl FnMut(&V, &V) -> bool) -> V {
    let mut selected = Vec::new();
    let mut id = values.first;
    let mut visited = 0;
    while let Some(current) = id {
        assert!(
            visited < native_map_index_size(&values.order),
            "Rust ABI: cyclic Map order"
        );
        visited += 1;
        let entry = native_map_entry(values, current);
        if keep(&entry.key, &entry.value) {
            selected.push(entry.clone());
        }
        id = entry.next;
    }
    let count = selected.len();
    let mut order = V::Unit;
    let mut bucket_entries = std::collections::BTreeMap::<u32, Vec<V>>::new();
    for (id, mut entry) in selected.into_iter().enumerate() {
        let id = u64::try_from(id).expect("Map address fits u64");
        bucket_entries
            .entry(entry.hash)
            .or_default()
            .push(checked_int(
                i64::try_from(id).expect("Map address fits Int"),
            ));
        entry.previous = id.checked_sub(1);
        entry.next = (id + 1 < count as u64).then_some(id + 1);
        order = native_map_index_set(&order, id, V::MapEntry(Shared::new(entry)));
    }
    let mut buckets = V::Unit;
    for (hash, ids) in bucket_entries {
        buckets = native_map_index_set(&buckets, u64::from(hash), V::Array(Shared::new(ids)));
    }
    V::Map(Shared::new(NativeMap {
        seed: values.seed,
        buckets,
        order,
        first: (count != 0).then_some(0),
        last: count.checked_sub(1).map(|id| id as u64),
    }))
}

fn native_map_operation(name: &str, dictionaries: Vec<V>, arguments: Vec<V>) -> V {
    if name == "std/map::empty" {
        assert!(dictionaries.is_empty());
        assert!(matches!(arguments.as_slice(), [V::Unit]));
        return V::Map(Shared::new(NativeMap::empty(native_process_hash_seed())));
    }
    if name == "std/map::singleton" {
        let [key, value]: [V; 2] = arguments
            .try_into()
            .unwrap_or_else(|_| panic!("Rust ABI: Map singleton arity"));
        let empty = NativeMap::empty(native_process_hash_seed());
        let (_, hash) = native_map_key_hash(&empty, &key, &dictionaries);
        return native_map_append(&empty, key, value, hash);
    }
    let values = native_map_value(arguments.last().expect("Rust ABI: Map source arity"));
    match name {
        "std/map::filter" => {
            assert!(dictionaries.is_empty());
            assert_eq!(arguments.len(), 2, "Rust ABI: Map filter arity");
            native_map_select(values, |key, value| {
                arguments[0]
                    .apply(key.clone())
                    .apply(value.clone())
                    .boolean()
            })
        }
        "std/map::mapValues" => {
            assert!(dictionaries.is_empty());
            assert_eq!(arguments.len(), 2, "Rust ABI: Map mapValues arity");
            let mut order = V::Unit;
            let mut id = values.first;
            let mut visited = 0;
            while let Some(current) = id {
                assert!(
                    visited < native_map_index_size(&values.order),
                    "Rust ABI: cyclic Map order"
                );
                visited += 1;
                let mut entry = native_map_entry(values, current).clone();
                entry.value = arguments[0].apply(entry.value.clone());
                id = entry.next;
                order = native_map_index_set(&order, current, V::MapEntry(Shared::new(entry)));
            }
            V::Map(Shared::new(NativeMap {
                seed: values.seed,
                buckets: values.buckets.clone(),
                order,
                first: values.first,
                last: values.last,
            }))
        }
        "std/map::get" | "std/map::containsKey" => {
            let (eq, hash) = native_map_key_hash(values, &arguments[0], &dictionaries);
            let found = native_map_find(values, &arguments[0], hash, &eq);
            if name == "std/map::containsKey" {
                return V::Bool(found.is_some());
            }
            match found {
                Some(id) => V::adt(
                    "std/prelude::Just",
                    native_map_entry(values, id).value.clone(),
                ),
                None => V::adt("std/prelude::Nothing", V::Unit),
            }
        }
        "std/map::insert" => native_map_upsert(values, &dictionaries, &arguments[0], |_| {
            arguments[1].clone()
        }),
        "std/map::upsert" => {
            assert_eq!(arguments.len(), 3, "Rust ABI: Map upsert arity");
            native_map_upsert(values, &dictionaries, &arguments[0], |current| {
                let current = current.map_or_else(
                    || V::adt("std/prelude::Nothing", V::Unit),
                    |value| V::adt("std/prelude::Just", value.clone()),
                );
                arguments[1].apply(current)
            })
        }
        "std/map::mergeWith" | "std/map::mapKeysWith" => {
            assert_eq!(
                arguments.len(),
                3,
                "Rust ABI: Map merge/key transform arity"
            );
            let rekey = name == "std/map::mapKeysWith";
            let source = if rekey {
                values
            } else {
                native_map_value(&arguments[1])
            };
            let mut result = if rekey {
                V::Map(Shared::new(NativeMap::empty(source.seed)))
            } else {
                arguments[2].clone()
            };
            let mut id = source.first;
            let mut visited = 0;
            while let Some(current) = id {
                assert!(
                    visited < native_map_index_size(&source.order),
                    "Rust ABI: cyclic Map order"
                );
                visited += 1;
                let entry = native_map_entry(source, current);
                let key = if rekey {
                    arguments[1].apply(entry.key.clone())
                } else {
                    entry.key.clone()
                };
                result =
                    native_map_upsert(native_map_value(&result), &dictionaries, &key, |current| {
                        current.map_or_else(
                            || entry.value.clone(),
                            |value| arguments[0].apply(value.clone()).apply(entry.value.clone()),
                        )
                    });
                id = entry.next;
            }
            result
        }
        "std/map::remove" => {
            let (eq, hash) = native_map_key_hash(values, &arguments[0], &dictionaries);
            let Some(id) = native_map_find(values, &arguments[0], hash, &eq) else {
                return arguments[1].clone();
            };
            let entry = native_map_entry(values, id);
            let mut order = native_map_index_remove(&values.order, id);
            if let Some(previous) = entry.previous {
                let mut value = native_map_entry(values, previous).clone();
                value.next = entry.next;
                order = native_map_index_set(&order, previous, V::MapEntry(Shared::new(value)));
            }
            if let Some(next) = entry.next {
                let mut value = native_map_entry(values, next).clone();
                value.previous = entry.previous;
                order = native_map_index_set(&order, next, V::MapEntry(Shared::new(value)));
            }
            let bucket: Vec<_> = native_map_bucket(values, entry.hash)
                .iter()
                .filter(|value| native_map_address(value) != id)
                .cloned()
                .collect();
            let buckets = if bucket.is_empty() {
                native_map_index_remove(&values.buckets, u64::from(entry.hash))
            } else {
                native_map_index_set(
                    &values.buckets,
                    u64::from(entry.hash),
                    V::Array(Shared::new(bucket)),
                )
            };
            V::Map(Shared::new(NativeMap {
                seed: values.seed,
                buckets,
                order,
                first: if values.first == Some(id) {
                    entry.next
                } else {
                    values.first
                },
                last: if values.last == Some(id) {
                    entry.previous
                } else {
                    values.last
                },
            }))
        }
        "std/map::keys" | "std/map::values" | "std/map::entries" => {
            assert!(dictionaries.is_empty());
            let mut output = Vec::new();
            let mut id = values.first;
            while let Some(current) = id {
                assert!(
                    (output.len() as u64) < native_map_index_size(&values.order),
                    "Rust ABI: cyclic Map order"
                );
                let entry = native_map_entry(values, current);
                output.push(if name == "std/map::keys" {
                    entry.key.clone()
                } else if name == "std/map::values" {
                    entry.value.clone()
                } else {
                    V::Tuple(Shared::new(vec![entry.key.clone(), entry.value.clone()]))
                });
                id = entry.next;
            }
            V::Array(Shared::new(output))
        }
        "std/map::size" => {
            assert!(dictionaries.is_empty());
            checked_int(native_map_index_size(&values.order) as i64)
        }
        "std/map::isEmpty" => {
            assert!(dictionaries.is_empty());
            V::Bool(matches!(values.order, V::Unit))
        }
        _ => panic!("Rust ABI: unsupported Map operation"),
    }
}

#[cfg(test)]
mod map_tests {
    use super::*;

    #[test]
    fn upsert_merge_and_rekey_call_hash_once_and_callbacks_in_source_order() {
        enable_cycle_collection();
        let d = dictionaries();
        let original =
            native_map_operation("std/map::singleton", d.clone(), vec![V::Int(1), V::Int(10)]);
        events();
        let update = V::fun(vec![], |_, current| {
            if current.tagged("std/prelude::Nothing") {
                EVENTS.with(|events| events.borrow_mut().push("u:none".into()));
                V::Int(42)
            } else {
                let V::Int(value) = current.payload() else {
                    panic!()
                };
                EVENTS.with(|events| events.borrow_mut().push(format!("u:{value}")));
                V::Int(value + 1)
            }
        });
        let changed = native_map_operation(
            "std/map::upsert",
            d.clone(),
            vec![V::Int(1), update.clone(), original.clone()],
        );
        assert_eq!(events(), ["h1", "e1:1", "u:10"]);
        let changed = native_map_operation(
            "std/map::upsert",
            d.clone(),
            vec![V::Int(3), update.clone(), changed],
        );
        assert_eq!(events(), ["h3", "e1:3", "u:none"]);
        assert_eq!(
            native_map_operation("std/map::entries", vec![], vec![changed.clone()]).render(),
            "[(1, 11), (3, 42)]"
        );
        assert_eq!(
            native_map_operation("std/map::entries", vec![], vec![original.clone()]).render(),
            "[(1, 10)]"
        );
        let left = native_map_operation(
            "std/map::insert",
            d.clone(),
            vec![V::Int(3), V::Int(30), original.clone()],
        );
        let right =
            native_map_operation("std/map::singleton", d.clone(), vec![V::Int(3), V::Int(7)]);
        let right = native_map_operation(
            "std/map::insert",
            d.clone(),
            vec![V::Int(2), V::Int(2), right],
        );
        let resolve = V::fun(vec![], |_, left| {
            let V::Int(left) = left else { panic!() };
            EVENTS.with(|events| events.borrow_mut().push(format!("r:{left}")));
            V::fun(vec![V::Int(left)], |captures, right| {
                let (V::Int(left), V::Int(right)) = (&captures[0], right) else {
                    panic!()
                };
                EVENTS.with(|events| events.borrow_mut().push(format!("i:{right}")));
                V::Int(left * 10 + right)
            })
        });
        events();
        let merged = native_map_operation(
            "std/map::mergeWith",
            d.clone(),
            vec![resolve.clone(), right.clone(), left.clone()],
        );
        assert_eq!(events(), ["h3", "e1:3", "e3:3", "r:30", "i:7", "h2"]);
        assert_eq!(
            native_map_operation("std/map::entries", vec![], vec![merged.clone()]).render(),
            "[(1, 10), (3, 307), (2, 2)]"
        );
        let source = native_map_operation(
            "std/map::insert",
            d.clone(),
            vec![V::Int(2), V::Int(20), left.clone()],
        );
        let key = V::fun(vec![], |_, key| {
            let V::Int(key) = key else { panic!() };
            EVENTS.with(|events| events.borrow_mut().push(format!("k:{key}")));
            V::Int(key % 2)
        });
        events();
        let rekeyed = native_map_operation(
            "std/map::mapKeysWith",
            d.clone(),
            vec![resolve.clone(), key.clone(), source.clone()],
        );
        assert_eq!(
            events(),
            ["k:1", "h1", "k:3", "h1", "e1:1", "r:10", "i:30", "k:2", "h0"]
        );
        assert_eq!(
            native_map_operation("std/map::entries", vec![], vec![rekeyed.clone()]).render(),
            "[(1, 130), (0, 20)]"
        );
        assert_eq!(
            native_map_value(&rekeyed).seed,
            native_map_value(&source).seed
        );
        let empty = V::Map(Shared::new(NativeMap::empty(1234)));
        let never = V::fun(vec![], |_, _| {
            panic!("empty merge/rekey must not call a callback")
        });
        let output = native_map_operation(
            "std/map::mapKeysWith",
            d.clone(),
            vec![never.clone(), never.clone(), empty.clone()],
        );
        assert_eq!(native_map_value(&output).seed, 1234);
        let same = native_map_operation(
            "std/map::mergeWith",
            d.clone(),
            vec![never.clone(), empty.clone(), original.clone()],
        );
        assert_eq!(
            native_map_operation("std/map::entries", vec![], vec![same.clone()]).render(),
            "[(1, 10)]"
        );
        assert!(events().is_empty());
        drop((
            d, original, update, changed, left, right, resolve, merged, source, key, rekeyed,
            empty, never, output, same,
        ));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn update_merge_key_and_resolver_defects_release_partial_maps_and_ref_cycles() {
        enable_cycle_collection();
        for phase in 0..4 {
            let d = dictionaries();
            let cell = V::reference(V::Unit);
            let source = native_map_operation(
                "std/map::singleton",
                d.clone(),
                vec![V::Int(1), cell.clone()],
            );
            let source = native_map_operation(
                "std/map::insert",
                d.clone(),
                vec![V::Int(2), cell.clone(), source],
            );
            let update = V::fun(vec![cell.clone()], |_, _| {
                panic!("deliberate updater defect")
            });
            let resolve = V::fun(
                vec![cell.clone(), V::Bool(phase == 1 || phase == 3)],
                |captures, left| {
                    V::fun(
                        vec![captures[0].clone(), captures[1].clone(), left],
                        |captures, right| {
                            assert!(!captures[1].boolean(), "deliberate resolver defect");
                            right
                        },
                    )
                },
            );
            let key = V::fun(vec![cell.clone(), V::Bool(phase == 2)], |captures, key| {
                assert!(
                    !captures[1].boolean() || !matches!(key, V::Int(2)),
                    "deliberate key mapper defect"
                );
                V::Int(1)
            });
            cell.write_reference(V::Tuple(Shared::new(vec![
                update.clone(),
                resolve.clone(),
                key.clone(),
            ])));
            let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match phase {
                0 => native_map_operation(
                    "std/map::upsert",
                    d.clone(),
                    vec![V::Int(1), update.clone(), source.clone()],
                ),
                1 => native_map_operation(
                    "std/map::mergeWith",
                    d.clone(),
                    vec![resolve.clone(), source.clone(), source.clone()],
                ),
                _ => native_map_operation(
                    "std/map::mapKeysWith",
                    d.clone(),
                    vec![resolve.clone(), key.clone(), source.clone()],
                ),
            }));
            assert!(failed.is_err(), "phase{phase}");
            assert_eq!(
                native_map_operation("std/map::keys", vec![], vec![source.clone()]).render(),
                "[1, 2]"
            );
            drop((d, cell, source, update, resolve, key));
            collect_cycles().unwrap();
            assert_eq!(LIVE.with(Cell::get), 0, "phase{phase}");
        }
    }

    #[test]
    fn filter_and_map_values_keep_order_cached_hashes_gapped_addresses_and_snapshots() {
        enable_cycle_collection();
        let d = dictionaries();
        let source =
            native_map_operation("std/map::singleton", d.clone(), vec![V::Int(3), V::Int(30)]);
        let source = native_map_operation(
            "std/map::insert",
            d.clone(),
            vec![V::Int(1), V::Int(10), source],
        );
        let source = native_map_operation(
            "std/map::insert",
            d.clone(),
            vec![V::Int(2), V::Int(20), source],
        );
        let source = native_map_operation("std/map::remove", d.clone(), vec![V::Int(1), source]);
        events();
        let predicate = V::fun(vec![], |_, key| {
            EVENTS.with(|events| events.borrow_mut().push(format!("k{}", key.render())));
            V::fun(vec![key], |captures, value| {
                EVENTS.with(|events| {
                    events.borrow_mut().push(format!(
                        "v{}:{}",
                        captures[0].render(),
                        value.render()
                    ))
                });
                V::Bool(matches!(captures[0], V::Int(2)))
            })
        });
        let filtered = native_map_operation(
            "std/map::filter",
            vec![],
            vec![predicate.clone(), source.clone()],
        );
        assert_eq!(events(), ["k3", "v3:30", "k2", "v2:20"]);
        assert_eq!(
            native_map_operation("std/map::entries", vec![], vec![filtered.clone()]).render(),
            "[(2, 20)]"
        );
        assert_eq!(
            native_map_value(&filtered).seed,
            native_map_value(&source).seed
        );
        assert!(native_map_operation(
            "std/map::containsKey",
            d.clone(),
            vec![V::Int(2), filtered.clone()]
        )
        .boolean());
        assert_eq!(events(), ["h2", "e2:2"]);
        let mapper = V::fun(vec![], |_, value| {
            EVENTS.with(|events| events.borrow_mut().push(format!("m{}", value.render())));
            V::text(&format!("v{}", value.render()))
        });
        let mapped = native_map_operation(
            "std/map::mapValues",
            vec![],
            vec![mapper.clone(), source.clone()],
        );
        assert_eq!(events(), ["m30", "m20"]);
        assert_eq!(
            native_map_operation("std/map::entries", vec![], vec![mapped.clone()]).render(),
            "[(3, v30), (2, v20)]"
        );
        assert!(native_map_same_index(
            &native_map_value(&mapped).buckets,
            &native_map_value(&source).buckets
        ));
        assert_eq!(
            native_map_operation("std/map::entries", vec![], vec![source.clone()]).render(),
            "[(3, 30), (2, 20)]"
        );
        let empty = native_map_operation("std/map::empty", vec![], vec![V::Unit]);
        let never = V::fun(vec![], |_, _| panic!("empty Map must not call a callback"));
        for name in ["std/map::filter", "std/map::mapValues"] {
            let result = native_map_operation(name, vec![], vec![never.clone(), empty.clone()]);
            assert_eq!(
                native_map_operation("std/map::size", vec![], vec![result]).render(),
                "0"
            );
        }
        assert!(events().is_empty());
        drop((d, source, predicate, filtered, mapper, mapped, empty, never));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn selected_maps_release_excluded_keys_values_and_replaced_values() {
        enable_cycle_collection();
        clear_dictionaries();
        let d = vec![
            native_operator_dictionary("std/string::Eq"),
            native_hash_dictionary("std/string::Hash"),
        ];
        let key = V::text("discarded");
        let value = V::reference(V::Unit);
        let (V::Text(key_node), V::Reference(value_node)) = (&key, &value) else {
            panic!()
        };
        let key_weak = Rc::downgrade(key_node.0.as_ref().unwrap());
        let value_weak = Rc::downgrade(value_node.0.as_ref().unwrap());
        let source = native_map_operation(
            "std/map::singleton",
            d.clone(),
            vec![key.clone(), value.clone()],
        );
        let source = native_map_operation(
            "std/map::insert",
            d.clone(),
            vec![V::text("keep"), V::Int(7), source],
        );
        let source_id = source.node_id().unwrap();
        let predicate = V::fun(vec![], |_, key| {
            V::fun(vec![key], |captures, _| {
                V::Bool(native_text(&captures[0]) == "keep")
            })
        });
        let filtered =
            native_map_operation("std/map::filter", vec![], vec![predicate, source.clone()]);
        drop((key, value, source));
        collect_cycles().unwrap();
        assert!(key_weak.upgrade().is_none() && value_weak.upgrade().is_none());
        assert!(!NODES.with(|nodes| nodes.borrow().contains_key(&source_id)));
        assert_eq!(
            native_map_operation("std/map::entries", vec![], vec![filtered.clone()]).render(),
            "[(keep, 7)]"
        );
        drop(filtered);
        let old = V::reference(V::Unit);
        let V::Reference(old_node) = &old else {
            panic!()
        };
        let old_weak = Rc::downgrade(old_node.0.as_ref().unwrap());
        let source = native_map_operation(
            "std/map::singleton",
            d.clone(),
            vec![V::text("same-key"), old.clone()],
        );
        let source_id = source.node_id().unwrap();
        let mapper = V::fun(vec![], |_, _| V::Int(42));
        let mapped =
            native_map_operation("std/map::mapValues", vec![], vec![mapper, source.clone()]);
        drop((source, old));
        collect_cycles().unwrap();
        assert!(old_weak.upgrade().is_none());
        assert!(!NODES.with(|nodes| nodes.borrow().contains_key(&source_id)));
        assert_eq!(
            native_map_operation("std/map::entries", vec![], vec![mapped.clone()]).render(),
            "[(same-key, 42)]"
        );
        drop((mapped, d));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn filter_and_mapper_defects_unwind_partial_results_and_captured_ref_cycles() {
        enable_cycle_collection();
        for name in ["std/map::filter", "std/map::mapValues"] {
            let d = dictionaries();
            let source =
                native_map_operation("std/map::singleton", d.clone(), vec![V::Int(1), V::Int(1)]);
            let source = native_map_operation(
                "std/map::insert",
                d.clone(),
                vec![V::Int(2), V::Int(2), source],
            );
            let cell = V::reference(V::Unit);
            let callback = if name == "std/map::filter" {
                V::fun(vec![cell.clone()], |captures, key| {
                    V::fun(vec![captures[0].clone(), key], |captures, _| {
                        assert!(
                            !matches!(captures[1], V::Int(2)),
                            "deliberate predicate defect"
                        );
                        V::Bool(true)
                    })
                })
            } else {
                V::fun(vec![cell.clone()], |captures, value| {
                    assert!(!matches!(value, V::Int(2)), "deliberate mapper defect");
                    captures[0].clone()
                })
            };
            cell.write_reference(callback.clone());
            let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                native_map_operation(name, vec![], vec![callback.clone(), source.clone()])
            }));
            assert!(failed.is_err());
            drop((d, source, cell, callback));
            collect_cycles().unwrap();
            assert_eq!(LIVE.with(Cell::get), 0, "{name}");
        }
    }
    #[test]
    fn hash_entropy_requires_all_bytes_and_never_uses_a_fallback() {
        assert_eq!(
            native_read_hash_seed(&[0, 1, 2, 3, 4, 5, 6, 7][..]).unwrap(),
            0x0706050403020100
        );
        assert!(native_read_hash_seed(&[0; 7][..]).is_err());
        struct Denied;
        impl std::io::Read for Denied {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::ErrorKind::PermissionDenied.into())
            }
        }
        assert!(native_read_hash_seed(Denied).is_err());
        native_entropy_hash_seed().expect("supported host must provide secure entropy");
    }
    #[test]
    fn compressed_indexes_match_sparse_safe_integer_updates_and_reclaim_removed_paths() {
        clear_dictionaries();
        let mut index = V::Unit;
        let mut expected = BTreeMap::new();
        let mut state = 7_u64;
        for step in 0..2000_i64 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let id = state % 128;
            let key = if id < 64 {
                id
            } else {
                9_007_199_254_740_991 - (id - 64)
            };
            if (state >> 32) & 3 != 0 {
                index = native_map_index_set(&index, key, V::Int(step));
                expected.insert(key, step);
            } else {
                index = native_map_index_remove(&index, key);
                expected.remove(&key);
            }
            assert_eq!(native_map_index_size(&index), expected.len() as u64);
            assert_eq!(
                LIVE.with(Cell::get),
                expected.len().saturating_mul(2).saturating_sub(1)
            );
            let vacant = (0..=128).find(|key| !expected.contains_key(key)).unwrap();
            assert_eq!(native_map_index_vacant(&index), vacant);
            if step % 25 == 0 {
                for (key, value) in &expected {
                    assert!(
                        matches!(native_map_index_get(&index,*key),Some(V::Int(actual)) if actual==value)
                    );
                }
            }
        }
        drop(index);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn point_updates_share_unchanged_paths_and_keep_original_snapshots() {
        let mut index = V::Unit;
        for key in 0..1024 {
            index = native_map_index_set(&index, key, V::Int(key as i64));
        }
        let before = LIVE.with(Cell::get);
        let updated = native_map_index_set(&index, 512, V::Int(-1));
        assert!(
            LIVE.with(Cell::get) - before <= 54,
            "only a bounded radix path may be added"
        );
        assert!(matches!(
            native_map_index_get(&index, 512),
            Some(V::Int(512))
        ));
        assert!(matches!(
            native_map_index_get(&updated, 512),
            Some(V::Int(-1))
        ));
        drop((index, updated));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn deleting_a_key_releases_its_value_and_reuses_the_address_at_the_order_tail() {
        enable_cycle_collection();
        clear_dictionaries();
        let key = V::text("retired");
        let value = V::reference(V::Unit);
        let (V::Text(key_node), V::Reference(value_node)) = (&key, &value) else {
            panic!()
        };
        let key_weak = Rc::downgrade(key_node.0.as_ref().unwrap());
        let value_weak = Rc::downgrade(value_node.0.as_ref().unwrap());
        let d = vec![
            native_operator_dictionary("std/string::Eq"),
            native_hash_dictionary("std/string::Hash"),
        ];
        let mut map = native_map_operation(
            "std/map::singleton",
            d.clone(),
            vec![key.clone(), value.clone()],
        );
        for name in ["stay-a", "stay-b"] {
            map = native_map_operation(
                "std/map::insert",
                d.clone(),
                vec![V::text(name), V::Unit, map],
            );
        }
        let removed =
            native_map_operation("std/map::remove", d.clone(), vec![key.clone(), map.clone()]);
        drop((key, value, map));
        assert!(key_weak.upgrade().is_none());
        assert!(value_weak.upgrade().is_none());
        let inserted = native_map_operation(
            "std/map::insert",
            d.clone(),
            vec![V::text("fresh"), V::Unit, removed],
        );
        assert_eq!(
            native_map_operation("std/map::keys", vec![], vec![inserted.clone()]).render(),
            "[stay-a, stay-b, fresh]"
        );
        drop((d, inserted));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    thread_local! {static EVENTS:RefCell<Vec<String>>=const{RefCell::new(vec![])};}

    fn dictionaries() -> Vec<V> {
        let eq = V::dict(vec![(
            "eq",
            V::fun(vec![], |_, left| {
                V::fun(vec![left], |c, right| {
                    EVENTS.with(|e| {
                        e.borrow_mut()
                            .push(format!("e{}:{}", c[0].render(), right.render()))
                    });
                    V::Bool(c[0].render() == right.render())
                })
            }),
        )]);
        let hash = V::dict(vec![(
            "hash",
            V::fun(vec![], |_, key| {
                let V::Int(key) = key else { panic!() };
                EVENTS.with(|e| e.borrow_mut().push(format!("h{key}")));
                V::Int(key % 2)
            }),
        )]);
        vec![eq, hash]
    }
    fn events() -> Vec<String> {
        EVENTS.with(|e| std::mem::take(&mut *e.borrow_mut()))
    }

    #[test]
    fn fixed_seed_and_mixed_hash_match_the_ts_signed64_contract() {
        for (text, expected) in [
            ("0", 0),
            ("+0001", 1),
            ("-1", u64::MAX),
            ("9223372036854775807", i64::MAX as u64),
            ("-9223372036854775808", i64::MIN as u64),
        ] {
            assert_eq!(native_fixed_hash_seed(text), Ok(expected));
        }
        for text in [
            "",
            "+",
            "--1",
            " 1",
            "1 ",
            "1.0",
            "1e2",
            "1雪",
            "9223372036854775808",
            "-9223372036854775809",
        ] {
            assert!(native_fixed_hash_seed(text).is_err(), "{text}");
        }
        for (hash, seed, expected) in [
            (0, 0, 292980411),
            (42, 1, 91505446),
            (-1, u64::MAX, 292980411),
            (9007199254740991, i64::MAX as u64, 2702587611),
            (-9007199254740991, i64::MIN as u64, 2685810220),
        ] {
            assert_eq!(native_mix_hash(hash, seed), expected);
        }
    }

    #[test]
    fn map_snapshots_preserve_order_and_call_hash_once_then_only_same_bucket_eq() {
        let d = dictionaries();
        events();
        let empty = V::Map(Shared::new(NativeMap::empty(42)));
        let one = native_map_operation(
            "std/map::insert",
            d.clone(),
            vec![V::Int(1), V::text("one"), empty.clone()],
        );
        assert_eq!(events(), ["h1"]);
        let two = native_map_operation(
            "std/map::insert",
            d.clone(),
            vec![V::Int(3), V::text("three"), one.clone()],
        );
        assert_eq!(events(), ["h3", "e1:3"]);
        let three = native_map_operation(
            "std/map::insert",
            d.clone(),
            vec![V::Int(2), V::text("two"), two.clone()],
        );
        assert_eq!(events(), ["h2"]);
        let found = native_map_operation("std/map::get", d.clone(), vec![V::Int(3), three.clone()]);
        assert_eq!(found.payload().render(), "three");
        assert_eq!(events(), ["h3", "e1:3", "e3:3"]);
        let updated = native_map_operation(
            "std/map::insert",
            d.clone(),
            vec![V::Int(1), V::text("new"), three.clone()],
        );
        assert_eq!(events(), ["h1", "e1:1"]);
        assert_eq!(
            native_map_operation("std/map::keys", vec![], vec![updated.clone()]).render(),
            "[1, 3, 2]"
        );
        assert_eq!(
            native_map_operation("std/map::values", vec![], vec![three.clone()]).render(),
            "[one, three, two]"
        );
        let removed = native_map_operation(
            "std/map::remove",
            d.clone(),
            vec![V::Int(3), updated.clone()],
        );
        assert_eq!(events(), ["h3", "e1:3", "e3:3"]);
        assert_eq!(
            native_map_operation("std/map::keys", vec![], vec![removed.clone()]).render(),
            "[1, 2]"
        );
        assert_eq!(
            native_map_operation("std/map::size", vec![], vec![empty.clone()]).render(),
            "0"
        );
        assert!(events().is_empty());
        drop((d, empty, one, two, three, found, updated, removed));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn map_values_are_owned_edges_in_captured_function_ref_cycles() {
        enable_cycle_collection();
        let reference = V::reference(V::Unit);
        let callback = V::fun(vec![reference.clone()], |_, value| value);
        let map = native_map_append(&NativeMap::empty(42), V::text("saved"), callback.clone(), 0);
        reference.write_reference(map.clone());
        let selected = native_map_operation("std/map::values", vec![], vec![map.clone()]).index(0);
        drop((map, callback));
        assert_eq!(selected.apply(V::Int(42)).render(), "42");
        drop((reference, selected));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn contains_and_entries_preserve_pairing_order_and_dictionary_evaluation() {
        let d = dictionaries();
        events();
        let first = native_map_operation(
            "std/map::singleton",
            d.clone(),
            vec![V::Int(1), V::text("one")],
        );
        let values = native_map_operation(
            "std/map::insert",
            d.clone(),
            vec![V::Int(3), V::text("three"), first.clone()],
        );
        events();
        assert_eq!(
            native_map_operation(
                "std/map::containsKey",
                d.clone(),
                vec![V::Int(3), values.clone()]
            )
            .render(),
            "True"
        );
        assert_eq!(events(), ["h3", "e1:3", "e3:3"]);
        assert_eq!(
            native_map_operation(
                "std/map::containsKey",
                d.clone(),
                vec![V::Int(9), values.clone()]
            )
            .render(),
            "False"
        );
        assert_eq!(events(), ["h9", "e1:9", "e3:9"]);
        assert_eq!(
            native_map_operation("std/map::entries", vec![], vec![values.clone()]).render(),
            "[(1, one), (3, three)]"
        );
        assert!(events().is_empty());
        drop((d, first, values));
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
