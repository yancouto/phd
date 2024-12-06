use std::sync::{LazyLock, Mutex};

use dynamic_2core::lists::AggregatedData;
use flexi_logger::{Logger, LoggerHandle};

pub mod slow_lct;
pub mod slow_lists;

#[derive(Debug, Clone, Default)]
pub struct AggSum(pub i32);

impl AggregatedData for AggSum {
    type Data = i32;

    fn from(data: &Self::Data) -> Self {
        Self(*data)
    }

    fn merge(self, right: Self) -> Self {
        Self(self.0 + right.0)
    }

    fn reverse(self) -> Self {
        self
    }
}

impl PartialEq<i32> for AggSum {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

#[derive(Default, Debug, Clone)]
pub struct AggDigit {
    number: i32,
    size: u8,
}

impl AggregatedData for AggDigit {
    type Data = i32;

    fn from(&data: &Self::Data) -> Self {
        assert!(data >= 0 && data < 10);
        Self {
            number: data,
            size: 1,
        }
    }

    fn merge(self, right: Self) -> Self {
        Self {
            number: self.number * 10_i32.pow(right.size.into()) + right.number,
            size: self.size + right.size,
        }
    }

    fn reverse(mut self) -> Self {
        let mut new_number = 0;
        for _ in 0..self.size {
            new_number = new_number * 10 + (self.number % 10);
            self.number /= 10;
        }
        self.number = new_number;
        self
    }
}

impl PartialEq<i32> for AggDigit {
    fn eq(&self, &other: &i32) -> bool {
        self.number == other
    }
}

#[allow(dead_code)]
pub static LOGGER: LazyLock<Mutex<LoggerHandle>> = LazyLock::new(|| {
    Mutex::new(
        Logger::try_with_env_or_str("info")
            .unwrap()
            .write_mode(flexi_logger::WriteMode::SupportCapture)
            .log_to_stdout()
            .set_palette("196;208;3;7;8".to_owned())
            .format(|w, now, record| {
                let style = flexi_logger::style(record.level());
                write!(
                    w,
                    "{} {pref}[{}] {}{suf}",
                    now.format("%H:%M:%S"),
                    &record.level().as_str()[0..1],
                    record.args(),
                    pref = style.prefix(),
                    suf = style.suffix(),
                )
            })
            .start()
            .unwrap(),
    )
});

#[allow(dead_code)]
pub fn init_logger() {
    let _ = &*LOGGER;
}
