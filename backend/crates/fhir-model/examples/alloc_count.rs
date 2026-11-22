//! Counts heap allocations made while deserializing the bench fixtures.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

use haste_fhir_model::r4::generated::resources::{Bundle, Patient, Resource};

struct Counting;
static ALLOCS: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn measure<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let before = ALLOCS.load(Ordering::Relaxed);
    let out = f();
    println!(
        "{label}: {} allocations",
        ALLOCS.load(Ordering::Relaxed) - before
    );
    out
}

fn main() {
    let bundle_string = include_str!("../benches/fixtures/synthea_transaction_bundle.json");
    // Warm the lazily built regexes so they are not counted.
    let warm = serde_json::from_str::<Bundle>(bundle_string).unwrap();
    let patient = warm
        .entry
        .iter()
        .flatten()
        .find_map(|entry| match entry.resource.as_deref() {
            Some(Resource::Patient(patient)) => Some(patient.clone()),
            _ => None,
        })
        .unwrap();
    let patient_string = serde_json::to_string(&patient).unwrap();

    measure("deserialize Bundle as Resource", || {
        serde_json::from_str::<Resource>(bundle_string).unwrap()
    });
    measure("deserialize Patient as Patient", || {
        serde_json::from_str::<Patient>(&patient_string).unwrap()
    });
    measure("deserialize Patient as Resource", || {
        serde_json::from_str::<Resource>(&patient_string).unwrap()
    });
}
