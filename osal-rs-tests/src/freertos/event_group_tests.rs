extern crate alloc;

use osal_rs::os::*;
use osal_rs::os::types::EventBits;
use osal_rs::utils::Result;
use core::time::Duration;

const BIT_0: EventBits = 1 << 0;
const BIT_1: EventBits = 1 << 1;
const BIT_2: EventBits = 1 << 2;
const BIT_3: EventBits = 1 << 3;

pub fn test_event_group_creation() -> Result<()> {
    let event_group = EventGroup::new();
    assert!(event_group.is_ok());
    Ok(())
}

pub fn test_event_group_set_get() -> Result<()> {
    let event_group = EventGroup::new()?;
    
    let result = event_group.set(BIT_0);
    assert_ne!(result, 0);
    
    let bits = event_group.get();
    assert_eq!(bits & BIT_0, BIT_0);
    Ok(())
}

pub fn test_event_group_multiple_bits() -> Result<()> {
    let event_group = EventGroup::new()?;
    
    event_group.set(BIT_0 | BIT_1 | BIT_2);
    
    let bits = event_group.get();
    assert_eq!(bits & BIT_0, BIT_0);
    assert_eq!(bits & BIT_1, BIT_1);
    assert_eq!(bits & BIT_2, BIT_2);
    Ok(())
}

pub fn test_event_group_clear() -> Result<()> {
    let event_group = EventGroup::new()?;
    
    event_group.set(BIT_0 | BIT_1 | BIT_2);
    
    event_group.clear(BIT_1);
    
    let bits = event_group.get();
    assert_eq!(bits & BIT_0, BIT_0);
    assert_eq!(bits & BIT_1, 0);
    assert_eq!(bits & BIT_2, BIT_2);
    Ok(())
}

pub fn test_event_group_clear_all() -> Result<()> {
    let event_group = EventGroup::new()?;
    
    event_group.set(BIT_0 | BIT_1 | BIT_2 | BIT_3);
    
    event_group.clear(0xFFFFFFFF);
    
    let bits = event_group.get();
    assert_eq!(bits, 0);
    Ok(())
}

pub fn test_event_group_wait() -> Result<()> {
    let event_group = EventGroup::new()?;
    
    event_group.set(BIT_0 | BIT_1);
    
    let result = event_group.wait(BIT_0 | BIT_1, Duration::from_millis(100).to_ticks());
    assert_eq!(result & BIT_0, BIT_0);
    assert_eq!(result & BIT_1, BIT_1);
    Ok(())
}

pub fn test_event_group_wait_timeout() -> Result<()> {
    let event_group = EventGroup::new()?;
    
    let result = event_group.wait(BIT_0, Duration::from_millis(10).to_ticks());
    assert_eq!(result, 0);
    Ok(())
}

pub fn test_event_group_wait_partial() -> Result<()> {
    let event_group = EventGroup::new()?;
    
    event_group.set(BIT_0);
    
    let result = event_group.wait(BIT_0 | BIT_1, Duration::from_millis(10).to_ticks());
    assert_eq!(result & BIT_0, BIT_0);
    Ok(())
}

pub fn test_event_group_sequential_operations() -> Result<()> {
    let event_group = EventGroup::new()?;
    
    event_group.set(BIT_0);
    assert_eq!(event_group.get() & BIT_0, BIT_0);
    
    event_group.set(BIT_1);
    assert_eq!(event_group.get() & (BIT_0 | BIT_1), BIT_0 | BIT_1);
    
    event_group.clear(BIT_0);
    assert_eq!(event_group.get() & BIT_0, 0);
    assert_eq!(event_group.get() & BIT_1, BIT_1);
    
    event_group.set(BIT_2);
    assert_eq!(event_group.get() & (BIT_1 | BIT_2), BIT_1 | BIT_2);
    Ok(())
}

pub fn test_event_group_all_bits() -> Result<()> {
    let event_group = EventGroup::new()?;
    
    let all_bits = 0x00FFFFFF;
    event_group.set(all_bits);
    
    let bits = event_group.get();
    assert_eq!(bits & all_bits, all_bits);
    Ok(())
}

pub fn test_event_group_drop() -> Result<()> {
    let event_group = EventGroup::new()?;
    event_group.set(BIT_0 | BIT_1);
    drop(event_group);
    Ok(())
}

pub fn run_all_tests() -> Result<()> {
    test_event_group_creation()?;
    test_event_group_set_get()?;
    test_event_group_multiple_bits()?;
    test_event_group_clear()?;
    test_event_group_clear_all()?;
    test_event_group_wait()?;
    test_event_group_wait_timeout()?;
    test_event_group_wait_partial()?;
    test_event_group_sequential_operations()?;
    test_event_group_all_bits()?;
    test_event_group_drop()?;
    Ok(())
}
