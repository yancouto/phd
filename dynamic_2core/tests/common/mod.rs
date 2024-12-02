use std::sync::{Arc, LazyLock, Mutex, RwLock};

use dynamic_2core::implicit_bst::AggregatedData;
use flexi_logger::{Logger, LoggerHandle};
use slow_bst::{Group, SlowBstData};

pub mod slow_bst;

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
}

static GROUPS: RwLock<Vec<Arc<RwLock<Group<AggSum>>>>> = RwLock::new(vec![]);

impl SlowBstData for AggSum {
    fn map() -> &'static RwLock<Vec<Arc<RwLock<Group<AggSum>>>>> {
        &GROUPS
    }
}

impl PartialEq<i32> for AggSum {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
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
