use anyhow::Result;
use chrono::prelude::*;
use csv::{Error, Writer, WriterBuilder};
use std::fs;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::constants::logging::*;
use crate::types::{ HardwareLogEntry};
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
