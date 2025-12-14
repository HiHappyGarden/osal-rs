use crate::utils::Result;



pub trait ToBytes {
    fn to_bytes(&self) -> &[u8];
}

pub trait FromBytes: Sized
where
    Self: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}

