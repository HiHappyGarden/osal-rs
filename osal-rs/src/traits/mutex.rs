use crate::utils::Result;
use super::ToTick;

pub trait MutexGuard
where
    Self: Sized,
{
    fn create() -> Result<Self>;
    fn take(&self, max_wait: impl ToTick) -> Result<()>;
    fn give(&self);
}
