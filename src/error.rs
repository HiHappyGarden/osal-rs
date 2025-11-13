

pub enum Error {
    Std(i32, &'static str)
}

pub type Result<T, E = Error> = core::result::Result<T, E>;