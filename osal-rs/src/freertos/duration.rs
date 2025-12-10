use core::time::Duration;
use crate::traits::{ToTick, FromTick};
use crate::freertos::config::TICK_RATE_HZ;
use crate::freertos::types::TickType;

impl ToTick for Duration {
    fn to_tick(&self) -> TickType {
        ((self.as_millis() as TickType) * TICK_RATE_HZ as TickType) / 1000 as TickType
    }
}

impl FromTick for Duration {
    fn tick(&mut self, tick: TickType) {
        let millis = (tick * 1000) / TICK_RATE_HZ as TickType;
        *self = Duration::from_millis(millis as u64);
    }
}