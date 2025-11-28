#[path = "../src/utils/csv_logger.rs"]
mod csv_logger;
#[cfg(test)]
mod tests {
    use crate::csv_logger::{CsvCpuLogEntry, CsvLogger};
    use chrono::Local;
    use tempfile::tempdir;
    #[test]
    fn test_csv_logger_write_read() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut logger = CsvLogger::new(Some(temp_path)).unwrap();

        let entries = vec![CsvCpuLogEntry {
            timestamp: Local::now().to_string(),
            temperature_unit: "Celsius".to_string(),
            temperature: 65.5,
            cpu_usage: 45.2,
            power_draw: 35.8,
        }];

        logger.write(entries.clone()).unwrap();
        logger.flush_buffer().unwrap();

        // Read back and verify
        let read_entries = logger.read().unwrap();
        assert_eq!(read_entries.len(), 1);
        assert_eq!(read_entries[0].temperature, 65.5);
        assert_eq!(read_entries[0].cpu_usage, 45.2);
        assert_eq!(read_entries[0].power_draw, 35.8);
        println!("{:?}", read_entries);
    }

    #[test]
    fn test_date_rotation_creates_two_files() {
        // Create temp directory for test
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        // Create logger
        let mut logger = CsvLogger::new(Some(temp_path)).unwrap();

        // Write first entry (creates first file with today's date)
        let entry1 = vec![CsvCpuLogEntry {
            timestamp: "2025-11-18 10:00:00".to_string(),
            temperature_unit: "C".to_string(),
            temperature: 65.0,
            cpu_usage: 50.0,
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
        let entry2 = vec![CsvCpuLogEntry {
            timestamp: "2025-11-18 11:00:00".to_string(),
            temperature_unit: "C".to_string(),
            temperature: 70.0,
            cpu_usage: 60.0,
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
    fn test_write_buffer_and_graph_data_separate() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        let mut logger = CsvLogger::new(Some(temp_path)).unwrap();

        // Write 5 entries
        for i in 0..5 {
            let entry = vec![CsvCpuLogEntry {
                timestamp: format!("2025-11-18 10:{:02}:00", i),
                temperature_unit: "C".to_string(),
                temperature: 65.0 + i as f32,
                cpu_usage: 50.0,
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
