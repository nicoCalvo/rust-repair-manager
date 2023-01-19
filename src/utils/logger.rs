#![allow(dead_code, unused_imports)]
use dotenv::dotenv;
use std::env;

use log::{debug, error, info, trace, warn};

pub fn setup_logger() -> Result<(), fern::InitError> {
    dotenv().ok(); 
    let logdate_format = env::var("LOGDATE_FORMAT").unwrap_or("[%Y-%m-%d][%H:%M:%S]".to_string());
    let log_file = env::var("LOG_FILE").unwrap_or("output.log".to_string());
    fern::Dispatch::new()
        .format(move|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format(&logdate_format),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file)?)
        .apply()?;
    Ok(())
}
