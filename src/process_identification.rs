use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows_sys::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows_sys::Win32::Foundation::{CloseHandle};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::formatting::update_process_header;

static LAST_PROCESS_NAME: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static mut FIRST_CALL_FLAG: bool = true;

fn get_focused_process_name() -> Result<Option<String>, Box<dyn std::error::Error>> {
    unsafe {
        // Get the foreground window
        let hwnd = GetForegroundWindow();
        if hwnd == std::ptr::null_mut() {
            return Err("No foreground window".into());
        }

        // Get process ID from a window
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut process_id);

        if process_id == 0 {
            return Err("Failed to get process ID".into());
        }

        // Open process handle
        let process_handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false as i32, // bInheritHandle = FALSE
            process_id
        );

        if process_handle == std::ptr::null_mut() {
            return Err("Failed to open process".into());
        }

        // Get module name
        let mut buffer: [u16; 260] = [0; 260];
        let chars_copied = GetModuleBaseNameW(
            process_handle,
            std::ptr::null_mut(), // hModule = NULL for main executable
            buffer.as_mut_ptr(),
            buffer.len() as u32
        );

        // Cleanup handle
        CloseHandle(process_handle);

        if chars_copied == 0 {
            return Err("Failed to get module name".into());
        }

        // Convert UTF-16 to String
        let name = String::from_utf16_lossy(&buffer[..chars_copied as usize]);

        // Update last process for first call
        if FIRST_CALL_FLAG {
            *LAST_PROCESS_NAME.lock().unwrap() = name.clone();
            FIRST_CALL_FLAG = false;
            return Ok(Some(name));
        }

        if name == *LAST_PROCESS_NAME.lock().unwrap() {
            return Ok(None);
        } else {
            *LAST_PROCESS_NAME.lock().unwrap() = name.clone();
        }

        Ok(Some(name))
    }
}

pub fn display_focused_process_name() {
        match get_focused_process_name() {
            Ok(None) => (),
            Ok(name) => {
                update_process_header(&name.unwrap()).unwrap();
            }
            Err(e) => eprintln!("Error retrieving focused process name: {}", e),
        }
}


pub fn reset_first_call_flag() {
    unsafe {
        FIRST_CALL_FLAG = true;
    }
}
