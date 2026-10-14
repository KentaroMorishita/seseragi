//! Deterministic resource-state model, not an async Effect interpreter.
use super::heap::Handle;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum Exit {
    Success(String),
    Failure(String),
    Cancelled,
    Defect(Vec<String>),
}

struct ResourceState {
    name: String,
    closed: Cell<bool>,
    capture: RefCell<Option<Handle>>,
    defect: Option<String>,
}
#[derive(Clone)]
pub struct Resource(Rc<ResourceState>);
impl Resource {
    pub fn new(name: &str, capture: Option<Handle>, defect: Option<&str>) -> Self {
        Self(Rc::new(ResourceState {
            name: name.into(),
            closed: Cell::new(false),
            capture: RefCell::new(capture),
            defect: defect.map(str::to_owned),
        }))
    }
    pub fn close(&self, events: &mut Vec<String>) -> Result<(), String> {
        if self.0.closed.replace(true) {
            return Ok(());
        }
        events.push(format!("release:{}", self.0.name));
        drop(self.0.capture.borrow_mut().take());
        self.0.defect.clone().map_or(Ok(()), Err)
    }
    pub fn closed(&self) -> bool {
        self.0.closed.get()
    }
}

#[derive(Default)]
pub struct Scope {
    resources: Vec<Resource>,
    closed: bool,
    defects: Vec<String>,
}
impl Scope {
    // Represents successful acquisition + registration as one uninterruptible
    // transition. No cancellation checkpoint exists between the two steps.
    pub fn acquired(&mut self, resource: Resource, events: &mut Vec<String>) {
        events.push(format!("acquire:{}", resource.0.name));
        if self.closed {
            // A non-cancellable acquisition may finish after scope close.
            // Release its result before discarding it; never publish the handle.
            if let Err(defect) = resource.close(events) {
                self.defects.push(defect);
            }
        } else {
            self.resources.push(resource);
        }
    }
    pub fn close(&mut self, original: Exit, events: &mut Vec<String>) -> Exit {
        if !self.closed {
            self.closed = true;
            while let Some(resource) = self.resources.pop() {
                if let Err(defect) = resource.close(events) {
                    self.defects.push(defect);
                }
            }
        }
        if self.defects.is_empty() {
            original
        } else {
            Exit::Defect(self.defects.clone())
        }
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct Trace {
    pub events: Vec<String>,
    pub exit: Exit,
}

pub fn scenarios() -> std::collections::BTreeMap<String, Trace> {
    let mut cases = std::collections::BTreeMap::new();
    for (name, exit) in [
        ("success", Exit::Success("done".into())),
        ("failure", Exit::Failure("expected".into())),
        ("defect", Exit::Defect(vec!["use-defect".into()])),
        ("cancel", Exit::Cancelled),
    ] {
        let mut scope = Scope::default();
        let mut events = Vec::new();
        scope.acquired(Resource::new("a", None, None), &mut events);
        scope.acquired(Resource::new("b", None, None), &mut events);
        let exit = scope.close(exit, &mut events);
        cases.insert(name.into(), Trace { events, exit });
    }
    let mut scope = Scope::default();
    let mut events = Vec::new();
    scope.acquired(Resource::new("a", None, Some("a-defect")), &mut events);
    scope.acquired(Resource::new("b", None, Some("b-defect")), &mut events);
    let exit = scope.close(Exit::Failure("expected".into()), &mut events);
    cases.insert("finalizer-defects".into(), Trace { events, exit });

    let mut scope = Scope::default();
    let mut events = Vec::new();
    let resource = Resource::new("explicit", None, None);
    scope.acquired(resource.clone(), &mut events);
    resource.close(&mut events).unwrap();
    let exit = scope.close(Exit::Success("done".into()), &mut events);
    cases.insert("explicit-close".into(), Trace { events, exit });

    let mut scope = Scope::default();
    let mut events = vec!["acquire:start".into(), "cancel".into()];
    scope.close(Exit::Cancelled, &mut events);
    scope.acquired(Resource::new("late", None, None), &mut events);
    let exit = scope.close(Exit::Cancelled, &mut events);
    cases.insert("late-acquire".into(), Trace { events, exit });
    cases
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heap::Heap;

    #[test]
    fn resource_cleanup_is_independent_of_gc_and_repeated_close() {
        let heap = Heap::default();
        let cell = heap.reference(heap.int(0));
        let callback = heap.function(vec![cell.clone(), heap.int(40)], |c, n| c[1].integer() + n);
        cell.write(callback.clone());
        let resource = Resource::new("subscription", Some(callback.clone()), None);
        drop(cell);
        let mut scope = Scope::default();
        let mut events = Vec::new();
        scope.acquired(resource.clone(), &mut events);
        drop(callback);
        assert_eq!(
            heap.collect().unwrap().reclaimed,
            0,
            "scope finalizer is a root"
        );
        assert!(!resource.closed());
        scope.close(Exit::Cancelled, &mut events);
        assert!(
            resource.closed(),
            "cleanup already ran without collecting memory"
        );
        assert_eq!(
            heap.live(),
            3,
            "value cycle survives deterministic resource close"
        );
        resource.close(&mut events).unwrap();
        scope.close(Exit::Cancelled, &mut events);
        assert_eq!(events, ["acquire:subscription", "release:subscription"]);
        assert_eq!(heap.collect().unwrap().reclaimed, 3);
    }

    #[test]
    fn escaped_callback_remains_alive_after_its_subscription_closes() {
        let heap = Heap::default();
        let callback = heap.function(vec![heap.int(40)], |c, n| c[0].integer() + n);
        let mut scope = Scope::default();
        let mut events = Vec::new();
        scope.acquired(
            Resource::new("subscription", Some(callback.clone()), None),
            &mut events,
        );
        scope.close(Exit::Success("done".into()), &mut events);
        heap.collect().unwrap();
        assert_eq!(callback.apply(2), 42);
        drop(callback);
        assert_eq!(heap.live(), 0);
    }

    #[test]
    fn cancel_at_each_acquire_handoff_boundary_never_loses_resource() {
        for cancel_before_register in [false, true] {
            let mut scope = Scope::default();
            let mut events = Vec::new();
            let resource = Resource::new("resource", None, None);
            if cancel_before_register {
                scope.close(Exit::Cancelled, &mut events);
            }
            scope.acquired(resource.clone(), &mut events);
            scope.close(Exit::Cancelled, &mut events);
            assert!(resource.closed());
            assert_eq!(events, ["acquire:resource", "release:resource"]);
        }
    }

    #[test]
    fn nested_scopes_and_finalizer_defects_preserve_local_lifo() {
        let mut outer = Scope::default();
        let mut inner = Scope::default();
        let mut events = Vec::new();
        outer.acquired(Resource::new("outer", None, None), &mut events);
        inner.acquired(
            Resource::new("inner-a", None, Some("a-defect")),
            &mut events,
        );
        inner.acquired(
            Resource::new("inner-b", None, Some("b-defect")),
            &mut events,
        );
        let exit = inner.close(Exit::Cancelled, &mut events);
        assert_eq!(
            exit,
            Exit::Defect(vec!["b-defect".into(), "a-defect".into()])
        );
        assert_eq!(outer.close(exit.clone(), &mut events), exit);
        assert_eq!(
            events,
            [
                "acquire:outer",
                "acquire:inner-a",
                "acquire:inner-b",
                "release:inner-b",
                "release:inner-a",
                "release:outer"
            ]
        );
    }
}
