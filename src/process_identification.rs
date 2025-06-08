use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows_sys::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows_sys::Win32::Foundation::{CloseHandle};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::formatting::{update_process_header, update_status_header};

static LAST_PROCESS_NAME: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

type ProcessNameResult<T> = Result<T, Box<dyn std::error::Error>>;

fn get_focused_process_name() -> ProcessNameResult<Option<String>> {
    unsafe {
        // Get the foreground window
        let hwnd = GetForegroundWindow();
        if hwnd == std::ptr::null_mut() {
            return Ok(None);
        }

        // Get process ID from a window
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut process_id);

        if process_id == 0 {
            return Ok(None);
        }

        // Open process handle
        let process_handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false as i32, // bInheritHandle = FALSE
            process_id
        );

        if process_handle == std::ptr::null_mut() {
            return Err(format!("Failed to open process for PID {}: {}", process_id, std::io::Error::last_os_error()).into());
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
            return Err(format!("Failed to get module name for PID {}: {}", process_id, std::io::Error::last_os_error()).into());
        }

        // Convert UTF-16 to String
        let name = String::from_utf16_lossy(&buffer[..chars_copied as usize]);

        let last_name = LAST_PROCESS_NAME.lock()
            .map_err(|_| "Failed to acquire lock on last process name")?;

        if last_name.as_ref() == Some(&name) {
            return Ok(None); // No change detected
        }

        Ok(Some(name))
    }
}

pub fn display_focused_process_name() {
    match get_focused_process_name() {
        // Successfully retrieved process name
        Ok(Some(name)) => {
            match LAST_PROCESS_NAME.lock() {
                // Successfully acquired lock and can update the process name
                Ok(mut last_name) => {
                    if let Err(e) = update_process_header(&name) {
                        eprintln!("Error updating process header: {}", e);
                        return;
                    }
                    *last_name = Some(name);
                }
                // Failed to acquire lock
                Err(_) => eprintln!("Failed to acquire lock for updating process name"),
            }
        }
        // No process name change detected
        Ok(None) => {
        }
        // Error retrieving process name
        Err(e) => {
            eprintln!("Error retrieving focused process name: {}", e);
            if let Err(e) = update_status_header("❌ Process detection error") {
                eprintln!("Additional error updating status: {}", e);
            }
        }
    }
}