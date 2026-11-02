use super::*;

pub(super) fn array_interface() -> ModuleInterface {
    collection_interface("std/array", "Array", "toList", "List")
}
