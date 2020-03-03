#[macro_use]
extern crate log;

use al_engine::application::Application;
use al_engine::renderer::VulkanApplication;
use simplelog::{Config, LevelFilter, SimpleLogger, TermLogger, TerminalMode};

fn setup_logger() {
    let log_level = LevelFilter::Trace;
    let config = Config::default();

    if let Err(_) = TermLogger::init(log_level, config.clone(), TerminalMode::Mixed) {
        SimpleLogger::init(log_level, config).unwrap();
    }
}

fn main() {
    setup_logger();

    let mut app = Application::new();
    app.main_loop();
}
