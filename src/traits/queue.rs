use crate::Result;

pub trait Queue {

    fn new(size: usize, message_size: usize) -> Self where Self: Sized;

    fn fetch<T>(&mut self, msg: &mut T, time: u64 ) -> Result<()>
    where
        T: Sized;

    fn fetch_from_isr<T>(&mut self, msg: &mut T, time: u64 ) -> Result<()>
    where
        T: Sized;

    fn post<T>(&mut self, msg: &T, time: u64) -> Result<()>
    where
        T: Sized;

    fn post_from_isr<T>(&mut self, msg: &T, time: u64) -> Result<()>
    where
        T: Sized;

    fn size(&self) -> usize;
}