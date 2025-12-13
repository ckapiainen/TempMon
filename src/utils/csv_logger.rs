use anyhow::Result;
use chrono::prelude::*;
use csv::{Error, Writer, WriterBuilder};
use std::fs;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::constants::logging::*;
use crate::types::HardwareLogEntry;
#[derive(Debug)]
pub struct CsvLogger {
    wtr: Writer<File>,
    pub path: PathBuf,
    pub timestamp: DateTime<Local>,
    pub runtime_start: SystemTime,
    write_buffer_size: usize,
    pub write_buffer: Vec<HardwareLogEntry>,
    pub graph_data_buffer: Vec<HardwareLogEntry>,
}

impl CsvLogger {
    // Helper function to get logs directory in AppData
    fn get_logs_dir() -> PathBuf {
        if cfg!(debug_assertions) {
            // dev write to project root /logs
            PathBuf::from("logs")
        } else {
            // prod write to %LOCALAPPDATA%/TempMon/logs
            if let Some(data_dir) = dirs::data_local_dir() {
                data_dir.join("TempMon").join("logs")
            } else {
                // fallback to project root
                PathBuf::from("logs")
            }
        }
    }

    pub fn new(custom_dir_path: Option<&str>) -> Result<Self> {
        let dir = if let Some(custom) = custom_dir_path {
            PathBuf::from(custom)
        } else {
            Self::get_logs_dir()
        };
        fs::create_dir_all(&dir)?;
        let date_str = Local::now().format("%d-%m-%Y").to_string();
        let path = dir.join(format!("{}_cpu_logs.csv", date_str));

        let wtr = Self::open_csv_writer(&path)?;

        Ok(Self {
            wtr,
            path,
            timestamp: Local::now(),
            runtime_start: SystemTime::now(),
            write_buffer_size: if cfg!(debug_assertions) {
                DEV_BUFFER_SIZE
            } else {
                PROD_BUFFER_SIZE
            },
            write_buffer: vec![],
            graph_data_buffer: vec![],
        })
    }

    // pub fn update_path(&mut self, new_path: PathBuf) {
    //     self.path = new_path;
    //     self.wtr = Self::open_csv_writer(&self.path).unwrap();
    // }
    pub fn read(&self) -> Result<Vec<HardwareLogEntry>> {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_path(&self.path)?;
        let mut result = vec![];
        for data in rdr.deserialize() {
            let record: HardwareLogEntry = data?;
            println!("{:?}", record);
            result.push(record);
        }
        Ok(result)
    }
    pub fn write(&mut self, mut entries: Vec<HardwareLogEntry>) -> Result<(), Error> {
        // Check current day if new writer with updated path is needed
        let today = Local::now();
        let date_str = today.format("%d-%m-%Y").to_string();

        if date_str != self.timestamp.format("%d-%m-%Y").to_string() {
            // Flush pending writes before rotating to new file
            self.flush_buffer()?;

            self.timestamp = today;
            let logs_dir = Self::get_logs_dir();
            self.path = logs_dir.join(format!("{}_hardware_logs.csv", date_str));
            self.wtr = Self::open_csv_writer(&self.path)?;
        }

        // Add to graph data (keep last N entries)
        self.graph_data_buffer.extend_from_slice(&entries);
        if self.graph_data_buffer.len() > GRAPH_DATA_BUFFER_MAX {
            self.graph_data_buffer
                .drain(0..self.graph_data_buffer.len() - GRAPH_DATA_BUFFER_MAX);
        }

        // Add to write buffer
        self.write_buffer.append(&mut entries);
        // Flush at max buffer size
        if self.write_buffer.len() >= self.write_buffer_size {
            self.flush_buffer()?;
        }

        Ok(())
    }

    pub fn flush_buffer(&mut self) -> Result<(), Error> {
        // Check if file still exists, recreate if deleted
        if !self.path.exists() {
            eprintln!("CSV file was deleted, recreating: {:?}", self.path);
            // Ensure parent directory exists
            if let Some(parent) = self.path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    Error::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to create directory: {}", e),
                    ))
                })?;
            }

            // Recreate the writer in append mode with headers
            self.wtr = Self::open_csv_writer(&self.path)?;
        }

        for entry in &self.write_buffer {
            self.wtr.serialize(entry)?;
        }
        self.wtr.flush()?;
        self.write_buffer.clear(); // Clear after writing to avoid duplicates
        Ok(())
    }

    // Helper function to open CSV writer in append mode with header check
    fn open_csv_writer(path: &PathBuf) -> Result<Writer<File>, Error> {
        let file_exists = path.exists();

        let file = OpenOptions::new().create(true).append(true).open(path)?;

        let mut wtr = WriterBuilder::new()
            .delimiter(b';')
            .has_headers(false)
            .from_writer(file);

        // Write headers if new file
        if !file_exists {
            wtr.write_record(&[
                "timestamp",
                "component_type",
                "temperature_unit",
                "temperature",
                "usage",
                "power_draw",
            ])?;
            wtr.flush()?;
        }
        Ok(wtr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ComponentType;
    use chrono::Local;
    use tempfile::tempdir;

    #[test]
    #[ignore] // TODO: Fix path issues in CI
    fn test_csv_logger_write_read() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut logger = CsvLogger::new(Some(temp_path)).unwrap();

        let entries = vec![HardwareLogEntry {
            timestamp: Local::now().to_string(),
            component_type: ComponentType::CPU,
            temperature_unit: "Celsius".to_string(),
            temperature: 65.5,
            usage: 45.2,
            power_draw: 35.8,
        }];

        logger.write(entries.clone()).unwrap();
        logger.flush_buffer().unwrap();

        // Read back and verify
        let read_entries = logger.read().unwrap();
        assert_eq!(read_entries.len(), 1);
        assert_eq!(read_entries[0].temperature, 65.5);
        assert_eq!(read_entries[0].usage, 45.2);
        assert_eq!(read_entries[0].power_draw, 35.8);
        println!("{:?}", read_entries);
    }

    #[test]
    #[ignore] // TODO: Fix path issues in CI
    fn test_date_rotation_creates_two_files() {
        // Create temp directory for test
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        // Create logger
        let mut logger = CsvLogger::new(Some(temp_path)).unwrap();

        // Write first entry (creates first file with today's date)
        let entry1 = vec![HardwareLogEntry {
            timestamp: "2025-11-18 10:00:00".to_string(),
            component_type: ComponentType::CPU,
            temperature_unit: "C".to_string(),
            temperature: 65.0,
            usage: 50.0,
            power_draw: 30.0,
        }];
        logger.write(entry1).unwrap();
        logger.flush_buffer().unwrap(); // Force flush to create file

        // Get the first file path
        let first_file = logger.path.clone();
        println!("First file: {:?}", first_file);

        // Simulate date change to yesterday (so "today" will be different)
        let yesterday = Local::now() - chrono::Duration::days(1);
        logger.timestamp = yesterday;

        // Write second entry (should create second file with new date)
        let entry2 = vec![HardwareLogEntry {
            timestamp: "2025-11-18 11:00:00".to_string(),
            component_type: ComponentType::CPU,
            temperature_unit: "C".to_string(),
            temperature: 70.0,
            usage: 60.0,
            power_draw: 35.0,
        }];
        logger.write(entry2).unwrap();
        logger.flush_buffer().unwrap(); // Force flush to create file

        // Get the second file path
        let second_file = logger.path.clone();
        println!("Second file: {:?}", second_file);

        // Verify both files exist
        assert!(first_file.exists(), "First file should exist");
        assert!(second_file.exists(), "Second file should exist");

        // Verify they have different names
        assert_ne!(first_file, second_file, "Files should have different names");

        // Verify both files contain data
        assert!(
            first_file.metadata().unwrap().len() > 0,
            "First file should have data"
        );
        assert!(
            second_file.metadata().unwrap().len() > 0,
            "Second file should have data"
        );
    }

    #[test]
    #[ignore] // TODO: Fix path issues in CI
    fn test_write_buffer_and_graph_data_separate() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        let mut logger = CsvLogger::new(Some(temp_path)).unwrap();

        // Write 5 entries
        for i in 0..5 {
            let entry = vec![HardwareLogEntry {
                timestamp: format!("2025-11-18 10:{:02}:00", i),
                component_type: ComponentType::CPU,
                temperature_unit: "C".to_string(),
                temperature: 65.0 + i as f32,
                usage: 50.0,
                power_draw: 30.0,
            }];
            logger.write(entry).unwrap();
        }

        // Graph data buffer should have all 5 entries
        assert_eq!(logger.graph_data_buffer.len(), 5);

        // Write buffer should be empty because write_buffer_size=1 causes auto-flush after each write
        assert_eq!(logger.write_buffer.len(), 0);

        // Verify the data was written to the file
        let read_entries = logger.read().unwrap();
        assert_eq!(read_entries.len(), 5);
    }
}
