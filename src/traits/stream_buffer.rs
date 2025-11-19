



pub trait StreamBuffer {
    fn new(size: usize, trigger_size: usize) -> Self where Self: Sized;

    fn send(&mut self, data: &[u8], time: u64) -> crate::Result<usize>;

    fn send_from_isr(&mut self, data: &[u8], time: u64) -> crate::Result<usize>;

    fn receive(&mut self, data: &mut [u8], time: u64) -> crate::Result<usize>;

    fn receive_from_isr(&mut self, data: &mut [u8], time: u64) -> crate::Result<usize>;

    fn available_data(&self) -> usize;

    fn available_space(&self) -> usize;
    
    fn reset(&mut self);
}