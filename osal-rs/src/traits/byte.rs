use crate::utils::Result;

trait BytesHasLen {
    fn len(&self) -> usize;
}

pub trait ToBytes {
    fn to_bytes(&self) -> &[u8];
}

impl<T, const N: usize> BytesHasLen for [T; N] 
where 
    T: ToBytes + Sized {
    fn len(&self) -> usize {
        N
    }
}

pub trait FromBytes: Sized
where
    Self: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}




