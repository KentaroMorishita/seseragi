use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
static COUNT_SYSTEM: AtomicBool = AtomicBool::new(false);
static SYSTEM_ALLOCS: AtomicUsize = AtomicUsize::new(0);
static SYSTEM_FREES: AtomicUsize = AtomicUsize::new(0);
static SYSTEM_REALLOCS: AtomicUsize = AtomicUsize::new(0);
static SYSTEM_REQUESTED: AtomicUsize = AtomicUsize::new(0);
static SYSTEM_RELEASED: AtomicUsize = AtomicUsize::new(0);
struct CountingSystem;
unsafe impl GlobalAlloc for CountingSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && COUNT_SYSTEM.load(Relaxed) {
            SYSTEM_ALLOCS.fetch_add(1, Relaxed);
            SYSTEM_REQUESTED.fetch_add(layout.size(), Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if COUNT_SYSTEM.load(Relaxed) {
            SYSTEM_FREES.fetch_add(1, Relaxed);
            SYSTEM_RELEASED.fetch_add(layout.size(), Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, old: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(ptr, old, size) };
        if !next.is_null() && COUNT_SYSTEM.load(Relaxed) {
            SYSTEM_REALLOCS.fetch_add(1, Relaxed);
            SYSTEM_REQUESTED.fetch_add(size, Relaxed);
            SYSTEM_RELEASED.fetch_add(old.size(), Relaxed);
        }
        next
    }
}
#[global_allocator]
static MEASURE_SYSTEM: CountingSystem = CountingSystem;
fn main() {
    let kernel = KERNEL(vec![]);
    assert!(matches!(kernel.apply(V::Int(1)), V::Int(0)));
    let before = ALLOCATIONS.with(Cell::get);
    let live = LIVE.with(Cell::get);
    COUNT_SYSTEM.store(true, Relaxed);
    let result = std::hint::black_box(&kernel).apply(V::Int(std::hint::black_box(1114113)));
    COUNT_SYSTEM.store(false, Relaxed);
    assert!(matches!(result, V::Int(620506872831)));
    let nodes = ALLOCATIONS.with(Cell::get) - before;
    assert_eq!(LIVE.with(Cell::get), live);
    assert_eq!(SYSTEM_ALLOCS.load(Relaxed), SYSTEM_FREES.load(Relaxed));
    assert_eq!(
        SYSTEM_REQUESTED.load(Relaxed),
        SYSTEM_RELEASED.load(Relaxed)
    );
    assert!(
        SYSTEM_ALLOCS.load(Relaxed) <= ALLOCATION_BOUND,
        "scalar ADT allocation bound: {} > {}",
        SYSTEM_ALLOCS.load(Relaxed),
        ALLOCATION_BOUND
    );
    drop(kernel);
    clear_dictionaries();
    clear_globals();
    assert_eq!(LIVE.with(Cell::get), 0);
    println!("{{\"allocations\":{},\"deallocations\":{},\"reallocations\":{},\"cumulativeRequestedBytes\":{},\"cumulativeReleasedBytes\":{},\"managedAllocations\":{},\"managedLive\":0,\"checksum\":620506872831}}", SYSTEM_ALLOCS.load(Relaxed), SYSTEM_FREES.load(Relaxed), SYSTEM_REALLOCS.load(Relaxed), SYSTEM_REQUESTED.load(Relaxed), SYSTEM_RELEASED.load(Relaxed), nodes);
}
