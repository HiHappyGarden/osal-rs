// Include auto-generated config from build.rs
include!(concat!(env!("OUT_DIR"), "/config_generated.rs"));

#[macro_export]
macro_rules! tick_period_ms {
    () => {
        // CHECK (1000 / $crate::freertos::config::TICK_RATE_HZ)
        ($crate::freertos::config::TICK_RATE_HZ)
    };
}