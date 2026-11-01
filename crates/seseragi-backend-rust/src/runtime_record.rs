// Only records connected to an HTML style input carry insertion order.
#[derive(Clone, Default)]
struct NativeStyleRecord {
    fields: BTreeMap<String, V>,
    order: Vec<String>,
}
impl NativeStyleRecord {
    fn new() -> Self {
        Self::default()
    }
    fn insert(&mut self, name: String, value: V) -> Option<V> {
        if !self.fields.contains_key(&name) {
            self.order.push(name.clone());
        }
        self.fields.insert(name, value)
    }
    fn extend(&mut self, fields: impl IntoIterator<Item = (String, V)>) {
        for (name, value) in fields {
            self.insert(name, value);
        }
    }
    fn ordered(&self) -> impl Iterator<Item = (&String, &V)> {
        self.order.iter().map(|name| (name, &self.fields[name]))
    }
}
impl std::ops::Deref for NativeStyleRecord {
    type Target = BTreeMap<String, V>;
    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}
impl TraceValue for NativeStyleRecord {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        for value in self.fields.values() {
            visit(value);
        }
        Ok(())
    }
}

#[cfg(test)]
mod record_order_tests {
    use super::*;

    #[test]
    fn updates_and_spreads_keep_the_first_insertion_order_and_sorted_lookup() {
        let mut base = NativeStyleRecord::new();
        base.insert("zIndex".into(), V::text("2"));
        base.insert("color".into(), V::text("red"));
        let mut record = NativeStyleRecord::new();
        record.extend(
            base.ordered()
                .map(|(name, value)| (name.clone(), value.clone())),
        );
        record.insert("color".into(), V::text("blue"));
        record.insert("backgroundColor".into(), V::text("white"));
        assert_eq!(
            record
                .ordered()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            ["zIndex", "color", "backgroundColor"]
        );
        assert_eq!(
            record.keys().map(String::as_str).collect::<Vec<_>>(),
            ["backgroundColor", "color", "zIndex"]
        );
        assert_eq!(native_text(record.get("color").unwrap()), "blue");
        drop(record);
        drop(base);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}

impl V {
    fn record_fields(&self) -> &BTreeMap<String, V> {
        match self {
            Self::Record(record) => record.get(),
            Self::HtmlStyleRecord(record) => &record.get().fields,
            _ => panic!("Rust ABI: expected record"),
        }
    }
    fn style_fields(&self) -> Vec<(&String, &V)> {
        match self {
            Self::HtmlStyleRecord(record) => record.get().ordered().collect(),
            Self::Record(record) => record.get().iter().collect(),
            _ => panic!("Rust ABI: expected record"),
        }
    }
}
