use std::fs;
// use std::path::Path;
use serde::{Serialize, Deserialize};
use dirs;
use reqwest::blocking::Client;
use clap::{Parser, Subcommand};
use tokio::runtime::Runtime;


#[derive(Debug, Deserialize)]
struct StatusResponse {
    temps: Temps,
    brightness: Brightness,
}

#[derive(Debug, Deserialize)]
struct Temps {
    reservoir: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Brightness {
    red: u32,
    green: u32,
    blue: u32,
    warmwhite: u32,
}

#[derive(Debug, Deserialize)]
enum DimMode {
    Log10,
    Log2,
    Linear,
}

#[derive(Debug, Deserialize)]
struct Config {
    default_dim_mode: DimMode,
    default_dim_steps: u8,
    api_url: String,
    led_max_dim_value: u16,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// increase brightness
    Up { amount: Option<u32> },
    /// decrease brightness
    Down { amount: Option<u32> },
    /// set brightness
    Set { brightness: u32 },
}

enum BrightnessChangeTypes {
    Increase,
    Decrease,
    Set,
}

fn send_brightness(brightness: &Brightness) -> Result<(), Box<dyn std::error::Error>> {
    let url = "http://192.168.1.81/api/v1/light/brightness";
    let client = reqwest::blocking::Client::new();
    println!("{:#?}", brightness);
    let response = client.post(url)
        .json(brightness)
        .send()?;

    Ok(())
}

fn fetch_status() -> Result<StatusResponse, Box<dyn std::error::Error>> {
    let url = "http://192.168.1.81/api/v1/status";
    let response = reqwest::blocking::get(url)?;
    let status: StatusResponse = response.json()?;
    Ok(status)
}

fn brightness_change(change_type: BrightnessChangeTypes, amount: u32, current_status: Option<StatusResponse>) -> StatusResponse{
    let mut status = match current_status {
        Some(current_status) => current_status,
        None => fetch_status().unwrap(),
    };
    match change_type {
        BrightnessChangeTypes::Increase => status.brightness.warmwhite += amount,
        BrightnessChangeTypes::Decrease => status.brightness.warmwhite -= amount,
        BrightnessChangeTypes::Set => status.brightness.warmwhite = amount,
    }
    send_brightness(&status.brightness).unwrap();
    status
}


fn main() {
    // let args = Args::parse();
    let cli = Cli::parse();

    let config_dir = dirs::config_dir().expect("Failed to get config directory");
    let blights_dir = config_dir.join("blights");
    let config_path = blights_dir.join("config.toml");
    let default_config = r#"
        default_dim_mode = "Log10"
        default_dim_steps = 1
        api_url = "http://192.168.1.81/api/v1"
        led_max_dim_value = 1023
    "#;

    if !blights_dir.exists() {
        fs::create_dir_all(&blights_dir).expect("Failed to create blights directory");
    }

    if !config_path.exists() {
        fs::write(&config_path, default_config).expect("Failed to write default config");
    }

    let config_str = fs::read_to_string(&config_path).expect("Failed to read config file");
    let config: Config = toml::from_str(&config_str).expect("Failed to parse config file");

    match &cli.command {
        Commands::Up { amount } => {
            match amount {
                Some(amount) => brightness_change(BrightnessChangeTypes::Increase, *amount, None),
                None => todo!("preset dimming"),
            };
        }
        Commands::Down { amount } => {
            match amount {
                Some(amount) => brightness_change(BrightnessChangeTypes::Decrease, *amount, None),
                None => todo!("preset dimming"),
            };
        }
        Commands::Set { brightness } => {
            brightness_change(BrightnessChangeTypes::Set, *brightness, None);
        }
    }

    

    // // Override config values with command-line arguments
    // if let Some(api_url) = args.api_url {
    //     config.api_url = api_url;
    // }
    // if let Some(dim_steps) = args.dim_steps {
    //     config.default_dim_steps = dim_steps;
    // }
    // if let Some(dim_mode) = args.dim_mode {
    //     config.default_dim_mode = match dim_mode.as_str() {
    //         "Log10" => DimMode::Log10,
    //         "Log2" => DimMode::Log2,
    //         "Linear" => DimMode::Linear,
    //         _ => panic!("Invalid dim mode value"),
    //     };
    // }

    // println!("Config: {:?}", config);
}