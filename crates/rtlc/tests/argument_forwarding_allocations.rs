use rtlc::{SafetyBounds, SafetyProgram};
use serde_json::{Value, json};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

const ARGUMENTS: usize = 4_096;
const LEVELS: usize = 3;
const PEAK_BUDGET_BYTES: usize = 256 * 1024 * 1024;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

struct PeakAllocator;

fn grow(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Ordering::Relaxed) + bytes;
    PEAK.fetch_max(live, Ordering::Relaxed);
}

unsafe impl GlobalAlloc for PeakAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            grow(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let resized = unsafe { System.realloc(pointer, layout, new_size) };
        if !resized.is_null() {
            if new_size >= layout.size() {
                grow(new_size - layout.size());
            } else {
                LIVE.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
            }
        }
        resized
    }
}

#[global_allocator]
static ALLOCATOR: PeakAllocator = PeakAllocator;

fn parameter(slot: usize) -> Value {
    json!({"op":"parameter","slot":slot})
}

fn forwarded(definition: usize) -> Value {
    json!({
        "op":"call",
        "definition":definition,
        "arguments":(0..ARGUMENTS).map(parameter).collect::<Vec<_>>(),
        "captures":[]
    })
}

fn definition(id: usize, parameter_count: usize, body: Value) -> Value {
    json!({
        "id":id,
        "parameterCount":parameter_count,
        "captureCount":0,
        "bodyKind":"expression",
        "localScope":0,
        "body":body
    })
}

fn forwarding_program() -> SafetyProgram {
    let zero = json!({"op":"literal","value":{"type":"int","value":"0"}});
    let mut definitions = vec![definition(
        0,
        ARGUMENTS,
        json!({"op":"literal","value":{"type":"bool","value":true}}),
    )];
    for level in 1..LEVELS {
        definitions.push(definition(level, ARGUMENTS, forwarded(level - 1)));
    }
    definitions.push(definition(
        LEVELS,
        0,
        json!({
            "op":"call",
            "definition":LEVELS - 1,
            "arguments":vec![zero; ARGUMENTS],
            "captures":[]
        }),
    ));
    serde_json::from_value(json!({
        "format":"modelt.safety-program",
        "version":2,
        "name":"Forwarding",
        "variables":["x"],
        "initial":[{"name":"Init","steps":[{"kind":"assign","slot":0,"value":{"op":"literal","value":{"type":"int","value":"0"}}}]}],
        "next":[{"name":"Next","steps":[{"kind":"assign","slot":0,"value":{"op":"read","slot":0,"next":false}}]}],
        "invariants":[{"name":"Safe","expression":{"op":"call","definition":LEVELS,"arguments":[],"captures":[]}}],
        "assumptions":[],
        "checkDeadlock":false,
        "definitions":definitions
    }))
    .unwrap()
}

#[test]
fn forwarding_4096_arguments_through_3_levels_completes_within_256_mib() {
    let program = forwarding_program();
    let before = LIVE.load(Ordering::Relaxed);
    PEAK.store(before, Ordering::Relaxed);
    let result = program
        .check(SafetyBounds {
            max_depth: 8,
            max_states: 100,
            max_work: 100000,
            max_temporal_nodes: None,
        })
        .unwrap();
    let peak = PEAK.load(Ordering::Relaxed).saturating_sub(before);
    assert_eq!(
        serde_json::to_value(&result).unwrap(),
        json!({
            "status":"complete",
            "reason":null,
            "states":1,
            "generated":2,
            "depth":0,
            "work":12,
            "violation":null
        })
    );
    assert!(
        peak <= PEAK_BUDGET_BYTES,
        "peak live heap during the check was {} MiB",
        peak / (1024 * 1024)
    );
}
