#[cfg(test)]
mod tests {
    extern crate alloc;
    
    use core::time::Duration;
    use osal_rs::os::*;
    use osal_rs::traits::{ToTick, FromTick};

    #[test]
    fn test_duration_to_ticks() {
        let duration = Duration::from_millis(100);
        let ticks = duration.to_ticks();
        assert!(ticks > 0);
    }

    #[test]
    fn test_duration_from_ticks() {
        let ticks: TickType = 1000;
        let mut duration = Duration::from_millis(0);
        duration.ticks(ticks);
        assert!(duration.as_millis() > 0);
    }

    #[test]
    fn test_duration_conversion_roundtrip() {
        let original = Duration::from_millis(500);
        let ticks = original.to_ticks();
        
        let mut converted = Duration::from_millis(0);
        converted.ticks(ticks);
        
        // Allow small rounding error
        let diff = if original > converted {
            original - converted
        } else {
            converted - original
        };
        assert!(diff.as_millis() < 10);
    }

    #[test]
    fn test_duration_zero() {
        let duration = Duration::from_millis(0);
        let ticks = duration.to_ticks();
        assert_eq!(ticks, 0);
    }

    #[test]
    fn test_duration_one_second() {
        let duration = Duration::from_secs(1);
        let ticks = duration.to_ticks();
        assert!(ticks >= 1000); // At least 1000 ticks for 1 second (1kHz tick rate)
    }

    #[test]
    fn test_duration_microseconds() {
        let duration = Duration::from_micros(1000); // 1 millisecond
        let ticks = duration.to_ticks();
        assert!(ticks >= 0);
    }

    #[test]
    fn test_duration_large_value() {
        let duration = Duration::from_secs(60); // 1 minute
        let ticks = duration.to_ticks();
        assert!(ticks > 0);
    }
}
