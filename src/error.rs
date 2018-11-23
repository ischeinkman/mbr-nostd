

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MbrError {
    cause: ErrorCause,
}

impl MbrError {
    pub fn from_cause(cause: ErrorCause) -> MbrError {
        MbrError { cause }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ErrorCause {
    UnsupportedPartitionError{tag : u8},
    InvalidMBRSuffix{actual : [u8 ; 2]},
    BufferWrongSizeError{expected : usize ,actual : usize},
}