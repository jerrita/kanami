use log::Log;
use std::{fmt, str::FromStr};

struct Logger;

macro_rules! with_color {
    ($args: ident, $color_code: ident) => {{ format!("\u{1B}[{}m{}\u{1B}[0m", $color_code as u8, $args) }};
}

pub fn init() {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(if let Ok(level) = std::env::var("LOG") {
        log::LevelFilter::from_str(&level).unwrap()
    } else {
        log::LevelFilter::Info
    });
}

fn print_in_color(args: fmt::Arguments, color_code: u8) {
    println!("{}", with_color!(args, color_code));
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }
    fn log(&self, record: &log::Record) {
        if record.target().starts_with("tungstenite") {
            return;
        }
        let color_code = match record.level() {
            log::Level::Error => 31, // Red
            log::Level::Warn => 33,  // Yellow
            log::Level::Info => 32,  // Green
            log::Level::Debug => 90, // Gray
            log::Level::Trace => 90, // Gray
        };
        print_in_color(
            format_args!(
                "[{}][{:>5}][{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.file().unwrap_or_default(),
                record.line().unwrap_or(0),
                record.args()
            ),
            color_code,
        );
    }
    fn flush(&self) {}
}
