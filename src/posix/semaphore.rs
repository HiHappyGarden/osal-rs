
pub struct Semaphore {

}

impl Semaphore {
    pub fn new() -> Self {
        Semaphore {}
    }
}

impl Default for Semaphore {
    fn default() -> Self {
        Self::new()
    }
}
