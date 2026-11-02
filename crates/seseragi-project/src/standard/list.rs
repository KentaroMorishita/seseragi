use super::*;

pub(super) fn list_interface() -> ModuleInterface {
    collection_interface("std/list", "List", "toArray", "Array")
}
