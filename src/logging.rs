//! Logging initialisation with log4rs.
//!
//! Sets up a rolling-file appender (10 MB per file, delete-oldest policy)
//! and a console appender. The `picoforge` logger defaults to `Trace` in
//! debug builds and `Info` in release builds; verbose third-party
//! loggers (`gpui`, `gpui_component`, `blade_graphics`) are capped at
//! `Error` to reduce noise.

use log::LevelFilter;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        rolling_file::{
            RollingFileAppender,
            policy::compound::{
                CompoundPolicy, roll::delete::DeleteRoller, trigger::size::SizeTrigger,
            },
        },
    },
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
};
use std::fs;

pub(crate) fn local_timestamp() -> String {
    crate::preferences::format_time(chrono::Utc::now(), "%H:%M:%S %:z")
}

/// Initializes log4rs with custom configuration for stdout and file logging.
pub fn logger_init() {
    // TODO: Add session based log files or rolling log files with archiving of old files, to prevent a single log file from growing too large.
    let size_trigger = SizeTrigger::new(10 * 1024 * 1024); // 10 MB limit
    let roller = DeleteRoller::new();
    let policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(roller));

    // File Appender
    let logfile = crate::storage::paths().and_then(|paths| {
        fs::create_dir_all(&paths.logs).map_err(|error| error.to_string())?;
        RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "[{d(%Y-%m-%d %H:%M:%S %Z)} {l} {t}] {m}{n}",
            )))
            .build(paths.logs.join("picoforge.log"), Box::new(policy))
            .map_err(|error| error.to_string())
    });

    // Console Appender
    let stdout = ConsoleAppender::builder()
        .target(Target::Stdout)
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S %Z)} {h({l})} {t}] {m}{n}",
        )))
        .build();

    let app_level = if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };

    let mut builder =
        log4rs::Config::builder().appender(Appender::builder().build("stdout", Box::new(stdout)));
    let mut appenders = vec!["stdout"];
    match logfile {
        Ok(logfile) => {
            builder = builder.appender(Appender::builder().build("logfile", Box::new(logfile)));
            appenders.push("logfile");
        }
        Err(error) => eprintln!("Could not open the application log: {error}"),
    }
    let config = builder
        .logger(
            Logger::builder()
                .appenders(appenders.clone())
                .additive(false)
                .build("picoforge", app_level),
        )
        .logger(Logger::builder().build("gpui", LevelFilter::Error))
        .logger(Logger::builder().build("gpui_component", LevelFilter::Error))
        .logger(Logger::builder().build("blade_graphics", LevelFilter::Error))
        .build(
            Root::builder()
                .appenders(appenders)
                .build(LevelFilter::Error),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
}
