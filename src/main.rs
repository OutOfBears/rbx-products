use std::collections::HashMap;

use clap::{Parser, Subcommand};
use log::info;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

mod api;
pub mod sync;
pub mod ui;
pub mod utils;

use crate::sync::download::Downloader;
use crate::sync::upload::Uploader;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Automatically answer "yes" to all prompts
    #[arg(short = 'y', long, default_value_t = false)]
    yes: bool,
    #[arg(short = 'o', long, default_value_t = false)]
    overwrite: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initializes the products file
    Init,
    /// Downloads all the products from the universe
    Download,
    /// Syncs products between file and universe
    Sync,
}

fn init_logging() {
    if std::env::var("RUST_LOG").is_err() {
        if cfg!(debug_assertions) {
            unsafe { std::env::set_var("RUST_LOG", "off,rbx_product=debug") }
        } else {
            unsafe { std::env::set_var("RUST_LOG", "rbx_product=info") }
        }
    }

    env_logger::init();
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    init_logging();
    let _ = color_eyre::install();

    if let Some(token) = std::env::var("RBX_API_KEY").ok() {
        api::set_api_token(token).await;
    }

    let args = Args::parse();
    let command = match args.command {
        Some(cmd) => cmd,
        None => {
            eprintln!("No command provided. Use --help for more information.");
            return;
        }
    };

    // flags::FLAGS.auto_yes = args.yes;

    let result = match command {
        Commands::Init => {
            info!("Initializing products file...");

            if std::fs::exists("products.toml").unwrap() {
                log::error!("products.toml already exists. Aborting initialization.");
                return;
            }

            let products = sync::products::VCSProducts {
                metadata: sync::products::Metadata {
                    universe_id: 1234,
                    discount_prefix: Some("💲{}% OFF💲 ".to_string()),
                    luau_file: Some("products.luau".to_string()),
                    name_filters: None,
                },
                gamepasses: HashMap::new(),
                products: HashMap::new(),
            };

            match products.save_products().await {
                Ok(_) => {
                    info!("products.toml initialized successfully.");
                    Ok(())
                }
                Err(e) => Err(format!("Failed to initialize products.toml: {}", e).into()),
            }
        }
        Commands::Download => Downloader::download(args.overwrite).await,
        Commands::Sync => Uploader::upload(args.overwrite).await,
    };

    if let Err(e) = result {
        log::error!("Error: {}", e);
    }
}
