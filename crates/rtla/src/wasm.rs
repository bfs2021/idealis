use std::cell::RefCell;

#[derive(Default)]
struct Buffer {
    input: Vec<u8>,
    output: Vec<u8>,
}
thread_local! { static BUFFER: RefCell<Buffer> = RefCell::new(Buffer::default()); }
#[unsafe(no_mangle)]
pub extern "C" fn rtla_abi_version() -> u32 {
    1
}
#[unsafe(no_mangle)]
pub extern "C" fn rtla_input_resize(length: u32) -> *mut u8 {
    BUFFER.with_borrow_mut(|buffer| {
        buffer.input.resize(length as usize, 0);
        buffer.input.as_mut_ptr()
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn rtla_dispatch() {
    BUFFER.with_borrow_mut(|buffer| {
        buffer.output = crate::dispatch(&buffer.input);
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn rtla_output_ptr() -> *const u8 {
    BUFFER.with_borrow(|buffer| buffer.output.as_ptr())
}
#[unsafe(no_mangle)]
pub extern "C" fn rtla_output_len() -> u32 {
    BUFFER.with_borrow(|buffer| buffer.output.len() as u32)
}
