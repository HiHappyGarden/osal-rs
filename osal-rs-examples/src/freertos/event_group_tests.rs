#[cfg(test)]
mod tests {
    extern crate alloc;
    
    use osal_rs::os::*;
    use osal_rs::utils::Result;
    use core::time::Duration;

    const BIT_0: EventBits = 1 << 0;
    const BIT_1: EventBits = 1 << 1;
    const BIT_2: EventBits = 1 << 2;
    const BIT_3: EventBits = 1 << 3;

    #[test]
    fn test_event_group_creation() {
        let event_group = EventGroup::new();
        assert!(event_group.is_ok());
    }

    #[test]
    fn test_event_group_set_get() {
        let event_group = EventGroup::new().unwrap();
        
        let result = event_group.set(BIT_0);
        assert_ne!(result, 0);
        
        let bits = event_group.get();
        assert_eq!(bits & BIT_0, BIT_0);
    }

    #[test]
    fn test_event_group_multiple_bits() {
        let event_group = EventGroup::new().unwrap();
        
        event_group.set(BIT_0 | BIT_1 | BIT_2);
        
        let bits = event_group.get();
        assert_eq!(bits & BIT_0, BIT_0);
        assert_eq!(bits & BIT_1, BIT_1);
        assert_eq!(bits & BIT_2, BIT_2);
    }

    #[test]
    fn test_event_group_clear() {
        let event_group = EventGroup::new().unwrap();
        
        event_group.set(BIT_0 | BIT_1 | BIT_2);
        
        event_group.clear(BIT_1);
        
        let bits = event_group.get();
        assert_eq!(bits & BIT_0, BIT_0);
        assert_eq!(bits & BIT_1, 0);
        assert_eq!(bits & BIT_2, BIT_2);
    }

    #[test]
    fn test_event_group_clear_all() {
        let event_group = EventGroup::new().unwrap();
        
        event_group.set(BIT_0 | BIT_1 | BIT_2 | BIT_3);
        
        event_group.clear(0xFFFFFFFF);
        
        let bits = event_group.get();
        assert_eq!(bits, 0);
    }

    #[test]
    fn test_event_group_wait() {
        let event_group = EventGroup::new().unwrap();
        
        event_group.set(BIT_0 | BIT_1);
        
        let result = event_group.wait(BIT_0 | BIT_1, Duration::from_millis(100).to_ticks());
        assert_eq!(result & BIT_0, BIT_0);
        assert_eq!(result & BIT_1, BIT_1);
    }

    #[test]
    fn test_event_group_wait_timeout() {
        let event_group = EventGroup::new().unwrap();
        
        let result = event_group.wait(BIT_0, Duration::from_millis(10).to_ticks());
        assert_eq!(result, 0);
    }

    #[test]
    fn test_event_group_wait_partial() {
        let event_group = EventGroup::new().unwrap();
        
        event_group.set(BIT_0);
        
        let result = event_group.wait(BIT_0 | BIT_1, Duration::from_millis(10).to_ticks());
        assert_eq!(result & BIT_0, BIT_0);
    }

    #[test]
    fn test_event_group_sequential_operations() {
        let event_group = EventGroup::new().unwrap();
        
        event_group.set(BIT_0);
        assert_eq!(event_group.get() & BIT_0, BIT_0);
        
        event_group.set(BIT_1);
        assert_eq!(event_group.get() & (BIT_0 | BIT_1), BIT_0 | BIT_1);
        
        event_group.clear(BIT_0);
        assert_eq!(event_group.get() & BIT_0, 0);
        assert_eq!(event_group.get() & BIT_1, BIT_1);
        
        event_group.set(BIT_2);
        assert_eq!(event_group.get() & (BIT_1 | BIT_2), BIT_1 | BIT_2);
    }

    #[test]
    fn test_event_group_all_bits() {
        let event_group = EventGroup::new().unwrap();
        
        let all_bits = 0x00FFFFFF;
        event_group.set(all_bits);
        
        let bits = event_group.get();
        assert_eq!(bits & all_bits, all_bits);
    }

    #[test]
    fn test_event_group_drop() {
        let event_group = EventGroup::new().unwrap();
        event_group.set(BIT_0 | BIT_1);
        drop(event_group);
    }
}
