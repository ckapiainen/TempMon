use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LogFileMetadata {
    pub path: PathBuf,
    pub filename: String,
    pub date: String,
    pub has_process_data: bool,
    pub processes: HashSet<String>,
    pub entry_count: usize,
    pub file_size: u64,
}

impl LogFileMetadata {
    pub fn from_path(path: PathBuf) -> Option<Self> {
        let filename = path.file_name()?.to_str()?.to_string();
        // Parse date from filename: "DD-MM-YYYY_hardware_logs.csv"
        let date = filename.strip_suffix("_hardware_logs.csv")?.to_string();
        // File size
        let file_size = fs::metadata(&path).ok()?.len();
        // scan for process data
        let (processes, entry_count) = Self::check_has_process_data(&path).unwrap_or_default(); // Returns (HashSet::new(), 0) on error
        let has_process_data = !processes.is_empty();

        Some(LogFileMetadata {
            path,
            filename,
            date,
            has_process_data,
            processes,
            entry_count,
            file_size,
        })
    }

    /// Scans entire file and extracts all unique process names
    fn check_has_process_data(path: &PathBuf) -> anyhow::Result<(HashSet<String>, usize)> {
        let mut processes = HashSet::new();
        let mut entry_count = 0;
        let mut rdr = csv::ReaderBuilder::new().delimiter(b';').from_path(path)?;

        for result in rdr.records() {
            let record = result?;
            entry_count += 1;
            if let Some(process_field) = record.get(1) {
                if !process_field.is_empty() {
                    // Split by comma to get individual processes
                    for process_entry in process_field.split(',') {
                        // Split by '=' and take only the name part (before stats)
                        if let Some(name) = process_entry.split('=').next() {
                            let trimmed = name.trim();
                            if !trimmed.is_empty() {
                                processes.insert(trimmed.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok((processes, entry_count))
    }

    /// Format file size (bytes → KB/MB)
    pub fn format_size(&self) -> String {
        if self.file_size < 1024 {
            format!("{} B", self.file_size)
        } else if self.file_size < 1024 * 1024 {
            format!("{} KB", self.file_size / 1024)
        } else {
            format!("{:.1} MB", self.file_size as f64 / (1024.0 * 1024.0))
        }
    }
}
