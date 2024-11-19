use crate::interrupts::InterruptIndex;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::instructions::port::Port;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
    );
    static ref SHELL: Mutex<crate::shell::Shell> = Mutex::new(crate::shell::Shell::new());
}

pub fn handle_keyboard_interrupt() {
    let mut keyboard = KEYBOARD.lock();
    let mut shell = SHELL.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(decoded_key) = keyboard.process_keyevent(key_event) {
            shell.handle_key(decoded_key);
        }
    }

    unsafe {
        crate::interrupts::PICS
            .lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
