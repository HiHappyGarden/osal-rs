use core::ops::Deref;

use crate::os::ToTick;
use crate::os::config::TICK_RATE_HZ;
use crate::traits::DurationFn;
use crate::freertos::types::{TickType};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration {
    tick: TickType,
}

impl DurationFn for Duration {
    fn new_sec(sec: impl Into<TickType>) -> Self {
        Self { tick: ((sec.into() * TICK_RATE_HZ as TickType) / 1_000) * 1000 }
    }

    fn new_millis(millis: impl Into<TickType>) -> Self {
        Self { tick: (millis.into() * TICK_RATE_HZ as TickType) / 1_000 }
    }

    fn new_micros(micros: impl Into<TickType>) -> Self {
        Self { tick: (micros.into() * TICK_RATE_HZ as TickType) / 1_000_000 }
    }
}

impl ToTick for Duration {
    type Target = TickType;

    fn get(&self) -> Self::Target {
        self.tick
    }
}