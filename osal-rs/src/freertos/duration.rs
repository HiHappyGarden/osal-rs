use core::time::Duration;
use crate::traits::{ToTick, FromTick};
use crate::freertos::config::TICK_RATE_HZ;
use crate::freertos::types::TickType;

impl ToTick for Duration {
    fn get_tick(&self) -> TickType {
        (self.as_millis() * TICK_RATE_HZ) / 1000
    }
}

impl FromTick for Duration {
    fn set_tick(&mut self, tick: TickType) {
        let millis = (tick * 1000) / TICK_RATE_HZ;
        *self = Duration::from_millis(millis);
    }
}