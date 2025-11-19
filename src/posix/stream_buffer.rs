
#[allow(
    dead_code,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes
)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/posix_bindings.rs"));

    impl Default for pthread_mutex_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_cond_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_condattr_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for pthread_mutexattr_t {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }

    impl Default for timespec {
        fn default() -> Self {
            unsafe { core::mem::zeroed() }
        }
    }
}

use std::ffi::c_int;
use crate::osal::stream_buffer::ffi::{clock_gettime, pthread_cond_destroy, pthread_cond_init, pthread_cond_signal, pthread_cond_t, pthread_cond_timedwait, pthread_cond_wait, pthread_condattr_init, pthread_condattr_setclock, pthread_condattr_t, pthread_mutex_destroy, pthread_mutex_init, pthread_mutex_lock, pthread_mutex_t, pthread_mutex_unlock, pthread_mutexattr_init, pthread_mutexattr_setprotocol, pthread_mutexattr_t, timespec, CLOCK_MONOTONIC, PTHREAD_PRIO_INHERIT};
use crate::traits::StreamBuffer as StreamBufferTrait;
use crate::{ErrorType, ErrorType::*, Error::Type, WAIT_FOREVER, Result, Error};
use crate::types::NSECS_PER_SEC;

macro_rules! timeout {
    ($self:expr, $rc:expr, $ret:expr, $txt:expr) => {{
        pthread_mutex_unlock (&mut $self.mutex);
        pthread_cond_signal (&mut $self.cond);

        if $rc == OsEno {
            return Ok($ret as usize);
        } else {
            return Err(Type($rc, $txt));
        }
    }};
}

pub struct StreamBuffer {
    cond: pthread_cond_t,
    mutex: pthread_mutex_t,
    trigger_size: usize,
    r: usize,
    w: usize,
    end: usize,
    count: usize,
    size: usize,
    buffer: Vec<u8>,
}

impl StreamBufferTrait for StreamBuffer {
    fn new(size: usize, trigger_size: usize) -> Self
    where
        Self: Sized
    {
        let mut mattr: pthread_mutexattr_t = Default::default();
        let mut cattr: pthread_condattr_t = Default::default();

        let buffer = vec![0u8; size];

        let mut cond: pthread_cond_t = Default::default();
        let mut mutex: pthread_mutex_t = Default::default();

        unsafe {
            pthread_condattr_init(&mut cattr);
            pthread_condattr_setclock(&mut cattr, CLOCK_MONOTONIC as c_int);
            pthread_cond_init(&mut cond, &cattr);

            pthread_mutexattr_init(&mut mattr);
            pthread_mutexattr_setprotocol(&mut mattr, PTHREAD_PRIO_INHERIT as c_int);
            pthread_mutex_init(&mut mutex, &mattr);
        }

        Self {
            cond,
            mutex,
            trigger_size,
            r: 0,
            w: 0,
            end: 0,
            count: 0,
            size,
            buffer,
        }
    }

    fn send(&mut self, data: &[u8], time: u64) -> Result<usize> {
        let mut ts: timespec = Default::default();
        let mut nsec = time * 1_000_000;
        let mut error = 0u8;
        let size = data.len();

        if data.is_empty() {
            return Err(Error::Std(-1, "Data is empty"));
        }

        if self.count == self.size {
            return Err(Type(OsEinval, "Stream buffer is full"));
        }

        if time != WAIT_FOREVER {
            unsafe {
                clock_gettime(CLOCK_MONOTONIC as i32, &mut ts);
            }
            nsec += ts.tv_nsec as u64;

            ts.tv_sec += (nsec / NSECS_PER_SEC) as i64;
            ts.tv_nsec = (nsec % NSECS_PER_SEC) as i64;
        }

        #[allow(unused_assignments)]
        let mut ret = 0;

        unsafe {
            pthread_mutex_lock(&mut self.mutex);

            ret = self.count;

            while self.count == self.size
            {
                if time != WAIT_FOREVER
                {
                    match ErrorType::new(pthread_cond_timedwait (&mut self.cond, &mut self.mutex, &ts)) {
                        OsEno => {},
                        OsEtimedout => timeout!(self, OsEtimedout, 0, "The time specified by abstime to pthread_cond_timedwait() has passed."),
                        OsEinval => timeout!(self, OsEinval, 0, "The value specified by abstime is invalid."),
                        OsEperm => timeout!(self, OsEperm, 0, "The mutex was not owned by the current thread at the time of the call."),
                        err => timeout!(self, err, 0, "Unhandled error."),
                    }
                } else {
                    match ErrorType::new(pthread_cond_wait (&mut self.cond, &mut self.mutex)) {
                        OsEno => {},
                        OsEtimedout => timeout!(self, OsEtimedout, 0, "The time specified by abstime to pthread_cond_wait() has passed."),
                        OsEinval => timeout!(self, OsEinval, 0, "The value specified by abstime is invalid."),
                        err => timeout!(self, err, 0, "Unhandled error."),
                    }
                }
            }
        }

        if (self.w + size) >= self.size && self.r > 0
        {
            let data_to_write  = self.size - self.w;
            let data_override  = size - data_to_write;

            self.buffer[self.w..self.w + data_to_write].copy_from_slice(&data[..data_to_write]);
            self.count += data_to_write;

            self.end = self.w;
            self.w += data_to_write;

            //i write on already read data
            if data_override <= self.r
            {
                //I can write all remaining data
                self.buffer[0..data_override].copy_from_slice(&data[data_to_write..data_to_write + data_override]);
                self.w = data_override;
                self.count += data_override;

            }
            else
            {
                //Partial writing, I trunk some data
                self.buffer[0..self.r].copy_from_slice(&data[data_to_write..data_to_write + self.r]);
                self.w = self.r;
                self.count += self.r;
                error = 1;
            }
        } else if self.r == 0 {
            if (self.w + size) <= self.size {
                self.buffer[self.w..self.w + size].copy_from_slice(&data[..size]);
                self.count += size;
                self.w += size;
            } else {
                let bytes_to_write = self.size - self.w;
                self.buffer[self.w..self.size].copy_from_slice(&data[..bytes_to_write]);
                self.count += bytes_to_write;
                self.w += bytes_to_write;
            }
        } else if self.r > 0 {
            let size_available = if self.r > self.w {
                self.r - self.w
            } else {
                self.w - self.r
            };

            if size <= size_available {
                self.buffer[self.w..self.w + size].copy_from_slice(&data[..size]);
                self.count += size;
                self.w += size;
            } else {
                // Scrittura parziale con troncamento
                let bytes_to_write = size.min(size_available);
                self.buffer[self.r..self.r + bytes_to_write].copy_from_slice(&data[..bytes_to_write]);
                self.count += bytes_to_write;
                self.r += bytes_to_write;
                error = 1;
            }
        }

        unsafe {
            pthread_mutex_unlock(&mut self.mutex);
            pthread_cond_signal(&mut self.cond);
        }

        if error != 0 {
            Err(Type(OsEinval, "Partial write occurred, some data was truncated"))
        } else {
            Ok(self.count - ret)
        }
    }

    fn send_from_isr(&mut self, data: &[u8], time: u64) -> Result<usize> {
        self.send(data, time)
    }

    fn receive(&mut self, data: &mut [u8], time: u64) -> Result<usize> {
        let mut ts: timespec = Default::default();
        let mut nsec = time * 1_000_000;
        let mut already_received= 0;
        let mut size = data.len();

        if data.is_empty() {
            return Err(Error::Std(-1, "Data is empty"));
        }

        if time != WAIT_FOREVER
        {
            unsafe {
                clock_gettime(CLOCK_MONOTONIC as i32, &mut ts);
            }
            nsec += ts.tv_nsec as u64;

            ts.tv_sec += (nsec / NSECS_PER_SEC) as i64;
            ts.tv_nsec = (nsec % NSECS_PER_SEC) as i64;
        }
        
        unsafe {
            pthread_mutex_lock (&mut self.mutex);

            while self.count < self.trigger_size {
                if time != WAIT_FOREVER
                {
                    match ErrorType::new(pthread_cond_timedwait (&mut self.cond, &mut self.mutex, &ts)) {
                        OsEno => {},
                        OsEtimedout => timeout!(self, OsEtimedout, 0, "The time specified by abstime to pthread_cond_timedwait() has passed."),
                        OsEinval => timeout!(self, OsEinval, 0, "The value specified by abstime is invalid."),
                        OsEperm => timeout!(self, OsEperm, 0, "The mutex was not owned by the current thread at the time of the call."),
                        err => timeout!(self, err, 0, "Unhandled error."),
                    }
                } else {
                    match ErrorType::new(pthread_cond_wait (&mut self.cond, &mut self.mutex)) {
                        OsEno => {},
                        OsEtimedout => timeout!(self, OsEtimedout, 0, "The time specified by abstime to pthread_cond_wait() has passed."),
                        OsEinval => timeout!(self, OsEinval, 0, "The value specified by abstime is invalid."),
                        err => timeout!(self, err, 0, "Unhandled error."),
                    }
                }

                if self.count == 0
                {
                    return Err(Error::Std(-2, "Stream buffer is empty"));
                }

                if self.r < self.w && self.end == 0 {
                    //space available
                    let mut  data_available = self.w - self.r;
                    if data_available == 0 && self.count > 0
                    {
                        data_available = self.count;
                    }

                    if size <= data_available
                    {
                        data[..size].copy_from_slice(&self.buffer[self.r..self.r + size]);

                        self.r += size;

                        self.count -= size;

                        already_received  = size;
                    }
                    else
                    {
                        data[..data_available].copy_from_slice(&self.buffer[self.r..self.r + data_available]);

                        self.r += data_available;

                        self.count -= data_available;

                        size -= data_available;

                        already_received  = data_available;
                    }
                }
                else if self.r >= self.w && self.end > 0
                {
                    //rotation but not all data are read before end
                    let mut data_available_between_r_and_size = self.size - self.r;


                    if data_available_between_r_and_size > 0 && size <= data_available_between_r_and_size
                    {
                        data[already_received..already_received + size].copy_from_slice(&self.buffer[self.r..self.r + size]);

                        self.r += size;

                        self.count -= size;

                        already_received = size;

                        #[allow(unused_assignments)]
                        {
                            data_available_between_r_and_size = size;
                        }

                        size = 0;
                    }
                    else if data_available_between_r_and_size > 0 && size > data_available_between_r_and_size
                    {
                        data[already_received..already_received + data_available_between_r_and_size].copy_from_slice(&self.buffer[self.r..self.r + data_available_between_r_and_size]);

                        self.r += data_available_between_r_and_size;

                        self.count -= data_available_between_r_and_size;

                        already_received = data_available_between_r_and_size;

                        size -= data_available_between_r_and_size;

                        #[allow(unused_assignments)]
                        {
                            data_available_between_r_and_size = 0;
                        }
                    }

                    if size > 0 && size <= self.w
                    {
                        data[already_received..already_received + size].copy_from_slice(&self.buffer[..size]);

                        self.r = size;

                        self.count -= size;

                        already_received += size;

                        size = 0;

                    }
                    else if size > 0 && size > self.w
                    {
                        data[already_received..already_received + self.w].copy_from_slice(&self.buffer[..self.w]);

                        self.r = self.w;

                        self.count -= self.w;

                        already_received = self.w;

                        size -= self.w;

                    }
                }
            }
        }

        if self.count == 0 {
            self.r = 0;
            self.w = 0;
            self.end = 0;
        }

        unsafe {
            pthread_mutex_unlock (&mut self.mutex);
            pthread_cond_signal (&mut self.cond);
        }

        Ok(already_received)
    }

    fn receive_from_isr(&mut self, data: &mut [u8], time: u64) -> Result<usize> {
        self.receive(data, time)
    }

    fn available_data(&self) -> usize {
        self.size
    }

    fn available_space(&self) -> usize {
        self.size - self.count
    }

    fn reset(&mut self) {
        self.r = 0;
        self.w = 0;
        self.end = 0;
        self.buffer.clear();
    }
}

impl Drop for StreamBuffer {
    fn drop(&mut self) {
        unsafe {
            pthread_cond_destroy (&mut self.cond);
            pthread_mutex_destroy (&mut self.mutex);
            self.reset();
        }
    }
}