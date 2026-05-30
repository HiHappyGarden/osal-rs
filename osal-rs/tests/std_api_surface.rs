#![cfg(feature = "std")]

use core::mem::size_of;
use core::ptr::null_mut;
use std::sync::Arc;
use std::time::Duration;

use osal_rs::os::*;

#[cfg(not(feature = "serde"))]
use osal_rs::os::{Deserialize, Serialize};

#[cfg(not(feature = "serde"))]
use osal_rs::utils::{Error, Result};

#[cfg(not(feature = "serde"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct QueueMessage {
    bytes: [u8; 4],
}

#[cfg(not(feature = "serde"))]
impl BytesHasLen for QueueMessage {
    fn len(&self) -> usize {
        self.bytes.len()
    }
}

#[cfg(not(feature = "serde"))]
impl Serialize for QueueMessage {
    fn to_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(not(feature = "serde"))]
impl Deserialize for QueueMessage {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 4 {
            return Err(Error::ReadError("invalid queue message length"));
        }

        Ok(Self {
            bytes: [bytes[0], bytes[1], bytes[2], bytes[3]],
        })
    }
}

#[cfg(feature = "serde")]
type QueueMessage = [u8; 4];

fn sample_message() -> QueueMessage {
    #[cfg(not(feature = "serde"))]
    {
        QueueMessage { bytes: [1, 2, 3, 4] }
    }

    #[cfg(feature = "serde")]
    {
        [1, 2, 3, 4]
    }
}

fn dummy_thread_handle() -> types::ThreadHandle {
    1usize as types::ThreadHandle
}

#[derive(Clone, Copy)]
struct HostPriority(types::UBaseType);

impl ToPriority for HostPriority {
    fn to_priority(&self) -> types::UBaseType {
        self.0
    }
}

#[test]
fn std_system_surface_signatures_compile() {
    let _start: fn() = <System as SystemFn>::start;
    let _get_state: fn() -> ThreadState = <System as SystemFn>::get_state;
    let _suspend_all: fn() = <System as SystemFn>::suspend_all;
    let _resume_all: fn() -> types::BaseType = <System as SystemFn>::resume_all;
    let _stop: fn() = <System as SystemFn>::stop;
    let _get_tick_count: fn() -> types::TickType = <System as SystemFn>::get_tick_count;
    let _get_current_time_us: fn() -> Duration = <System as SystemFn>::get_current_time_us;
    let _get_us_from_tick: fn(&Duration) -> types::TickType = <System as SystemFn>::get_us_from_tick;
    let _count_threads: fn() -> usize = <System as SystemFn>::count_threads;
    let _get_all_thread: fn() -> SystemState = <System as SystemFn>::get_all_thread;
    let _delay: fn(types::TickType) = <System as SystemFn>::delay;
    let _delay_until: fn(&mut types::TickType, types::TickType) = <System as SystemFn>::delay_until;
    let _critical_section_enter: fn() = <System as SystemFn>::critical_section_enter;
    let _critical_section_exit: fn() = <System as SystemFn>::critical_section_exit;
    let _check_timer: fn(&Duration, &Duration) -> osal_rs::utils::OsalRsBool = <System as SystemFn>::check_timer;
    let _yield_from_isr: fn(types::BaseType) = <System as SystemFn>::yield_from_isr;
    let _end_switching_isr: fn(types::BaseType) = <System as SystemFn>::end_switching_isr;
    let _enter_critical: fn() = <System as SystemFn>::enter_critical;
    let _exit_critical: fn() = <System as SystemFn>::exit_critical;
    let _enter_critical_from_isr: fn() -> types::UBaseType = <System as SystemFn>::enter_critical_from_isr;
    let _exit_critical_from_isr: fn(types::UBaseType) = <System as SystemFn>::exit_critical_from_isr;
    let _get_free_heap_size: fn() -> usize = <System as SystemFn>::get_free_heap_size;

    System::delay_with_to_tick(Duration::ZERO);

    let mut wake = 0;
    System::delay_until_with_to_tick(&mut wake, Duration::ZERO);

    let state = System::get_all_thread();
    let tasks: &[ThreadMetadata] = &state;
    let _ = tasks.first();
}

#[test]
fn std_backend_exports_core_api_surface() {
    let _max_mask: types::EventBits = EventGroup::MAX_MASK;
    let mut event_group = EventGroup::new().unwrap();
    let event_group_handle: &types::EventGroupHandle = &event_group;
    let _ = event_group_handle;
    let _ = event_group.set(0b0001);
    let _ = event_group.set_from_isr(0b0001);
    let _ = event_group.get();
    let _ = event_group.get_from_isr();
    let _ = event_group.clear(0b0001);
    let _ = event_group.clear_from_isr(0b0001);
    let _ = event_group.wait(0b0001, 0);
    let _ = event_group.wait_with_to_tick(0b0001, Duration::ZERO);
    let _ = format!("{event_group:?}");
    let _ = format!("{event_group}");
    event_group.delete();

    let mut raw_mutex = RawMutex::new().unwrap();
    let raw_mutex_handle: &types::MutexHandle = &raw_mutex;
    let _ = raw_mutex_handle;
    let _ = raw_mutex.lock();
    let _ = raw_mutex.unlock();
    let _ = raw_mutex.lock_from_isr();
    let _ = raw_mutex.unlock_from_isr();
    let _ = format!("{raw_mutex:?}");
    let _ = format!("{raw_mutex}");
    raw_mutex.delete();

    let mut mutex = Mutex::new(0u32);
    let mut guard = mutex.lock().unwrap();
    guard.update(&41);
    *guard += 1;
    drop(guard);
    let guard_from_isr = mutex.lock_from_isr().unwrap();
    drop(guard_from_isr);
    let explicit_guard = mutex.lock_from_isr_explicit().unwrap();
    drop(explicit_guard);
    *mutex.get_mut() = 7;
    let _ = format!("{mutex:?}");
    let _ = format!("{mutex}");
    let _shared_mutex = Mutex::new_arc(9u32);
    let inner_mutex = Mutex::new(11u32);
    let _ = inner_mutex.into_inner().unwrap();

    let mut queue = Queue::new(2, 4).unwrap();
    let queue_handle: &types::QueueHandle = &queue;
    let _ = queue_handle;
    let payload = [1u8, 2, 3, 4];
    let mut payload_buffer = [0u8; 4];
    queue.post(&payload, 0).unwrap();
    let _ = queue.fetch(&mut payload_buffer, 0);
    queue.post_from_isr(&payload).unwrap();
    let _ = queue.fetch_from_isr(&mut payload_buffer);
    queue.post_with_to_tick(&payload, Duration::ZERO).unwrap();
    let _ = queue.fetch_with_to_tick(&mut payload_buffer, Duration::ZERO);
    let _ = format!("{queue:?}");
    let _ = format!("{queue}");
    queue.delete();

    let message = sample_message();
    let mut streamed_queue = QueueStreamed::<QueueMessage>::new(2, size_of::<QueueMessage>() as types::UBaseType).unwrap();
    let streamed_handle: &types::QueueHandle = &streamed_queue;
    let _ = streamed_handle;
    streamed_queue.post(&message, 0).unwrap();
    let mut message_buffer = sample_message();
    let _ = streamed_queue.fetch(&mut message_buffer, 0);
    streamed_queue.post_from_isr(&message).unwrap();
    let _ = streamed_queue.fetch_from_isr(&mut message_buffer);
    let _ = format!("{streamed_queue:?}");
    let _ = format!("{streamed_queue}");
    streamed_queue.delete();

    let mut semaphore = Semaphore::new(2, 1).unwrap();
    let semaphore_handle: &types::SemaphoreHandle = &semaphore;
    let _ = semaphore_handle;
    let _ = Semaphore::new_with_count(0).unwrap();
    let _ = semaphore.wait(Duration::ZERO);
    let _ = semaphore.wait_from_isr();
    let _ = semaphore.signal();
    let _ = semaphore.signal_from_isr();
    let _ = format!("{semaphore:?}");
    let _ = format!("{semaphore}");
    semaphore.delete();

    let handle = dummy_thread_handle();
    let _ = Thread::new_with_handle(handle, "worker", 128, 1).unwrap();
    let _ = Thread::new_with_to_priority("worker", 128, HostPriority(1));
    let _ = Thread::new_with_handle_and_to_priority(handle, "worker", 128, HostPriority(1)).unwrap();
    let _ = Thread::get_metadata_from_handle(handle);

    let mut thread = Thread::new("worker", 128, 1);
    let thread_param: ThreadParam = Arc::new(1u32);
    let spawned = thread
        .spawn(Some(thread_param.clone()), |_thread, param| {
            Ok(param.unwrap_or_else(|| Arc::new(0u32) as ThreadParam))
        })
        .unwrap();
    let thread_handle: &types::ThreadHandle = &spawned;
    let _ = thread_handle;
    let _ = format!("{spawned:?}");
    let _ = format!("{spawned}");
    let _ = spawned.get_metadata();
    let _ = Thread::get_metadata(&spawned);
    spawned.notify(ThreadNotification::Increment).unwrap();
    let mut higher_priority_task_woken = 0;
    spawned
        .notify_from_isr(ThreadNotification::SetValueWithOverwrite(7), &mut higher_priority_task_woken)
        .unwrap();
    let _ = spawned.wait_notification(0, 0, 0);
    let _ = spawned.wait_notification_with_to_tick(0, 0, Duration::ZERO);
    spawned.suspend();
    spawned.resume();
    let _ = spawned.join(null_mut()).unwrap();
    spawned.delete();

    let mut simple_thread = Thread::new("simple", 128, 1);
    let _ = simple_thread.spawn_simple(|| {}).unwrap();
    let current = Thread::get_current();
    let _ = format!("{current:?}");

    let timer_param: TimerParam = Arc::new(5u32);
    let mut timer = Timer::new("timer", 1, false, Some(timer_param.clone()), |_timer, param| {
        Ok(param.unwrap_or_else(|| Arc::new(0u32) as TimerParam))
    })
    .unwrap();
    let _ = Timer::new_with_to_tick("timer", Duration::from_millis(1), true, None, |_timer, param| {
        Ok(param.unwrap_or_else(|| Arc::new(0u32) as TimerParam))
    })
    .unwrap();
    let timer_handle: &types::TimerHandle = &timer;
    let _ = timer_handle;
    let _ = format!("{timer:?}");
    let _ = format!("{timer}");
    let _ = timer.start(0);
    let _ = timer.stop(0);
    let _ = timer.reset(0);
    let _ = timer.change_period(1, 0);
    let _ = timer.start_with_to_tick(Duration::ZERO);
    let _ = timer.stop_with_to_tick(Duration::ZERO);
    let _ = timer.reset_with_to_tick(Duration::ZERO);
    let _ = timer.change_period_with_to_tick(Duration::from_millis(1), Duration::ZERO);
    let _ = timer.delete_with_to_tick(Duration::ZERO);
}