use crate::os::types::OsalRsBool;
use crate::utils::Result;
use super::ToTick;

pub trait Mutex
where
    Self: Sized,
{
    fn new() -> Result<Self>
    where 
        Self: Sized;

    fn lock(&self) -> OsalRsBool;

    fn lock_from_isr(&self) -> OsalRsBool;

    fn unlock(&self) -> OsalRsBool;

    fn unlock_from_isr(&self) -> OsalRsBool;

    fn delete(&mut self);
}
