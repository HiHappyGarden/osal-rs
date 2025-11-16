

pub trait Mutex {

    fn new() -> crate::Result<Self> where Self: Sized;

    fn lock(&mut self);

    fn lock_from_isr(&mut self);

    fn unlock(&mut self);

    fn unlock_from_isr(&mut self);

}