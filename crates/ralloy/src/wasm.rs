use crate::Engine;
use std::cell::RefCell;

struct Instance {
    engine: Engine,
    input: Vec<u8>,
    output: Vec<u8>,
}
thread_local! {static INSTANCE:RefCell<Instance>=RefCell::new(Instance { engine: Engine, input: Vec::new(), output: Vec::new() });}
#[unsafe(no_mangle)]
pub extern "C" fn ralloy_abi_version() -> u32 {
    1
}
#[unsafe(no_mangle)]
pub extern "C" fn ralloy_input_resize(length: u32) -> *mut u8 {
    INSTANCE.with_borrow_mut(|instance| {
        instance.input.resize(length as usize, 0);
        instance.input.as_mut_ptr()
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn ralloy_dispatch() {
    INSTANCE.with_borrow_mut(|instance| {
        instance.output = serde_json::to_vec(&instance.engine.dispatch(&instance.input))
            .expect("serializable analysis");
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn ralloy_output_ptr() -> *const u8 {
    INSTANCE.with_borrow(|instance| instance.output.as_ptr())
}
#[unsafe(no_mangle)]
pub extern "C" fn ralloy_output_len() -> u32 {
    INSTANCE.with_borrow(|instance| instance.output.len() as u32)
}
