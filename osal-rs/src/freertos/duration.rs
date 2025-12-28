use core::time::Duration;

use crate::traits::{ToTick, FromTick};
use super::config::TICK_RATE_HZ;
use super::types::TickType;

impl ToTick for Duration {
    fn to_ticks(&self) -> TickType {
        let millis = self.as_millis() as TickType;
        
        // Check for potential overflow and saturate at max value
        millis.saturating_mul(TICK_RATE_HZ as TickType) / 1000
    }
}

impl FromTick for Duration {
    fn ticks(&mut self, tick: TickType) {
        let millis = tick.saturating_mul(1000) / TICK_RATE_HZ as TickType;
        *self = Duration::from_millis(millis as u64);
    }
}