mod lower;
mod model;

use crate::SurfaceModule;

pub use model::{ModuleHeader, ModuleHeaderName};

pub(super) fn module_header_from_surface(module: &str, surface: &SurfaceModule) -> ModuleHeader {
    lower::module_header_from_surface(module, surface)
}

pub(super) fn empty_module_header(module: String, source: String) -> ModuleHeader {
    lower::empty_module_header(module, source)
}
