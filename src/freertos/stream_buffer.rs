use crate::types::{Result, Error::Std};
use crate::traits::StreamBufferTrait;
use crate::freertos::ffi::{StreamBufferHandle_t};

pub struct StreamBuffer {

}

impl StreamBufferTrait for StreamBuffer {
    fn new(size: usize, trigger_size: usize) -> Self where Self: Sized {
        todo!()
    }

    fn send(&mut self, data: &[u8], time: u64) -> crate::Result<usize> {
        todo!()
    }

    fn send_from_isr(&mut self, data: &[u8], time: u64) -> crate::Result<usize> {
        todo!()
    }

    fn receive(&mut self, data: &mut [u8], time: u64) -> crate::Result<usize> {
        todo!()
    }

    fn receive_from_isr(&mut self, data: &mut [u8], time: u64) -> crate::Result<usize> {
        todo!()
    }

    fn available_data(&self) -> usize {
        todo!()
    }

    fn available_space(&self) -> usize {
        todo!()
    }

    fn reset(&mut self) {
        todo!()
    }
}


unsafe impl Send for StreamBuffer {}
unsafe impl Sync for StreamBuffer {}