
use alloc::string::ToString;
use core::fmt::Arguments;
use crate::console::{print, print_with_color};
use crate::sbi;
use spin::RwLock;
pub struct Logger{
    heap_enabled: RwLock<bool>
}
impl Logger{
    pub const fn new() -> Self{
        Logger{ heap_enabled: RwLock::new(false) }
    }

    pub fn enable_heap(&self) {
        *self.heap_enabled.write() = true;
    }
}
impl log::Log for Logger{
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {

        let args = format_args!("[{}]{}:{} {}\n",
                                level_to_str(record.level()),
                                record.file().unwrap_or("-"),
                                record.line().unwrap_or(0),
                                record.args());
        let color_code = level_to_color_code(record.level());
        let args = with_color!(args, color_code);
        let heap_enabled = *self.heap_enabled.read();
        if heap_enabled {
            sbi::putstr(&args.to_string());
        }else{
            print(args);
        }
    }

    fn flush(&self) {
        
    }
}
static STD_LOGGER: Logger = Logger::new();

pub fn init_logger() {
    log::set_logger(&STD_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
}
pub fn enable_heap_logging() {
    STD_LOGGER.enable_heap();
}
fn level_to_color_code(lv: log::Level) -> u8{
    match lv{
        log::Level::Error => 91,
        log::Level::Warn => 93,
        log::Level::Info => 94,
        log::Level::Debug => 95,
        log::Level::Trace => 96,
    }
}

fn level_to_str(lv: log::Level) -> &'static str{
    match lv {
        log::Level::Error => "E",
        log::Level::Warn => "W",
        log::Level::Info => "I",
        log::Level::Debug => "D",
        log::Level::Trace => "T",
    }
}