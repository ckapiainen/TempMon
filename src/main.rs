#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide terminal on Windows
mod app;
mod assets;
mod collectors;
mod constants;
mod types;
mod utils;

use app::tempmon::TempMon;
use colored::Colorize;
use lhm_client::service::is_service_installed;
use lhm_client::{ComputerOptions, LHMClient};

/// Attempts to connect to the LHM service.
async fn connect_to_lhm_service() -> Option<lhm_client::LHMClientHandle> {
    match LHMClient::connect().await {
        Ok(client) => {
            println!("Connected to hardware monitoring service");
            client
                .set_options(ComputerOptions {
                    controller_enabled: false,
                    cpu_enabled: true,
                    gpu_enabled: true,
                    motherboard_enabled: false,
                    battery_enabled: false,
                    memory_enabled: false,
                    network_enabled: false,
                    psu_enabled: true,
                    storage_enabled: false,
                })
                .await
                .unwrap();
            client.update_all().await.unwrap();
            println!("{}", "Service options set".green().bold());
            Some(client)
        }
        Err(e) => {
            eprintln!("{} {}", "Failed to connect to service: {}".red(), e);
            eprintln!("{}", "The service may not be running. Try:".red());
            eprintln!("{}", "1. Run 'install-service.bat' as administrator".red());
            eprintln!(
                "{}",
                "2. Or manually start the service from Services (services.msc)".red()
            );
            None
        }
    }
}

/// Entry point for the app. Checks if LHM service is installed and runs the app.
fn main() -> iced::Result {
    match is_service_installed() {
        Ok(true) => {
            println!("{}", "✓ Service is ready".green());
        }
        Ok(false) => {
            eprintln!(
                "{}",
                "Hardware monitoring service not installed".red().bold()
            );
            eprintln!(
                "{}",
                "Please run 'install-service.bat' as administrator"
                    .red()
                    .bold()
            );
            // TODO: Show user a dialog or instructions
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error checking service: {}", e);
            std::process::exit(1);
        }
    }
    iced::daemon(TempMon::new, TempMon::update, TempMon::view)
        .subscription(TempMon::subscription)
        .title("TempMon")
        .antialiasing(true)
        .theme(TempMon::theme)
        .run()
}
