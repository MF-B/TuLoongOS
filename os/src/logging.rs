use log::{Level, LevelFilter, Metadata, Record, Log};
pub struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let (color, symbol) = match record.level() {
            Level::Error => ("\x1b[1;31m", "✗"),  // 粗体红色，错误符号
            Level::Warn => ("\x1b[1;33m", "⚠"),   // 粗体黄色，警告符号
            Level::Info => ("\x1b[1;32m", "ℹ"),   // 粗体绿色，信息符号
            Level::Debug => ("\x1b[1;36m", "⚙"),  // 粗体青色，调试符号
            Level::Trace => ("\x1b[1;35m", "»"),  // 粗体紫色，跟踪符号
        };
        // 模块名样式处理
        let module = "kernel";

        println!(
            "{}{} [{:<5}][{}] - {}\x1b[0m",
            color,
            symbol,
            record.level(),
            module,
            record.args(),
        );
    }

    fn flush(&self) {}
}

// 初始化日志系统的函数
pub fn init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN") => LevelFilter::Warn,
        Some("INFO") => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _ => LevelFilter::Info,
    });
}