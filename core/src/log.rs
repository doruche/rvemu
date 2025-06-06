use std::{fmt, sync::OnceLock};

const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const BLACK: &str = "\x1b[30m";
const RESET: &str = "\x1b[0m";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Off = 0,
    Trace = 1,
    Debug = 2,
    Warn = 3,
    Error = 4,
}

impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Off => "OFF",
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }

    fn color_code(&self) -> &'static str {
        match self {
            Level::Off => "",
            Level::Trace => BLACK,
            Level::Debug => CYAN,
            Level::Warn => YELLOW,
            Level::Error => RED,
        }
    }
}

static THRESHOLD: OnceLock<Level> = OnceLock::new();

fn get_threshold() -> Level {
    *THRESHOLD.get().expect("msg: Log level not initialized")
}

pub fn log_init(level: Level) {
    THRESHOLD.set(level).expect("Log level already set");
}

pub fn log(
    level: Level,
    args: fmt::Arguments<'_>,
    file: &'static str,
    line: u32,
) {
    if level <= get_threshold() {
        return;
    }
    println!(
        "{}{}\t[{}:{}]\t{}{}",
        level.color_code(),
        level.as_str(),
        file,
        line,
        args,
        RESET,
    );
}

// Or use unstable features to allow $$.
// #![feature(macro_metavar_expr)]

macro_rules! generate_log_macros {
    ($dollar:tt $($name:ident, $level:ident)*) => {
        $(
            #[macro_export]
            macro_rules! $name {
                ($dollar($args:tt)*) => {
                    $crate::log::log(
                        $crate::log::Level::$level,
                        format_args!($dollar($args)*),
                        file!(),
                        line!(),
                    );
                };
            }
        )*
    };
}

generate_log_macros!(
    $
    trace, Trace
    debug, Debug
    warn, Warn
    error, Error
);