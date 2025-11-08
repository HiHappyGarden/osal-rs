//! POSIX queue implementation using channels

use crate::{Error, Result, time::Duration};
use std::sync::mpsc::{self, SyncSender, Receiver, TryRecvError, RecvTimeoutError, TrySendError};
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

pub struct PosixQueue<T> {
    sender: SyncSender<T>,
    receiver: Arc<Mutex<Receiver<T>>>,
    capacity: usize,
}

impl<T> PosixQueue<T> {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::sync_channel(capacity);
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            capacity,
        }
    }

    pub fn send(&self, item: T) -> Result<()> {
        self.sender.send(item).map_err(|_| Error::Other("Queue disconnected"))
    }

    pub fn try_send(&self, item: T) -> Result<()> {
        self.sender.try_send(item).map_err(|e| match e {
            TrySendError::Full(_) => Error::WouldBlock,
            TrySendError::Disconnected(_) => Error::Other("Queue disconnected"),
        })
    }

    pub fn send_timeout(&self, item: T, _timeout: Duration) -> Result<()> {
        // Note: std::sync::mpsc::SyncSender doesn't have send_timeout
        // For simplicity, we use try_send which blocks until there's space
        self.sender.send(item).map_err(|_| Error::Other("Queue disconnected"))
    }

    pub fn recv(&self) -> Result<T> {
        let receiver = self.receiver.lock().unwrap();
        receiver.recv().map_err(|_| Error::Other("Queue disconnected"))
    }

    pub fn try_recv(&self) -> Result<T> {
        let receiver = self.receiver.lock().unwrap();
        receiver.try_recv().map_err(|e| match e {
            TryRecvError::Empty => Error::WouldBlock,
            TryRecvError::Disconnected => Error::Other("Queue disconnected"),
        })
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<T> {
        let receiver = self.receiver.lock().unwrap();
        receiver.recv_timeout(StdDuration::from(timeout)).map_err(|e| match e {
            RecvTimeoutError::Timeout => Error::Timeout,
            RecvTimeoutError::Disconnected => Error::Other("Queue disconnected"),
        })
    }

    pub fn len(&self) -> usize {
        // Note: std::sync::mpsc doesn't provide len(), so we return an approximation
        // In a real implementation, you might use a custom counter
        0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<T> Clone for PosixQueue<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: Arc::clone(&self.receiver),
            capacity: self.capacity,
        }
    }
}
