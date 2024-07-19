#[derive(Debug, thiserror::Error)]
#[error("Not all fields are prepared")]
pub struct NotPreparedError;

#[derive(Debug)]
pub enum ComputeClassHashError {
    InvalidBuiltinName,
    BytecodeSegmentLengthMismatch(BytecodeSegmentLengthMismatchError),
    InvalidBytecodeSegment(InvalidBytecodeSegmentError),
    PcOutOfRange(PcOutOfRangeError),
    Json(JsonError),
}

#[derive(Debug)]
pub enum CompressProgramError {
    Json(JsonError),
    Io(std::io::Error),
}

#[derive(Debug)]
pub struct JsonError {
    pub(crate) message: String,
}

#[derive(Debug)]
pub struct BytecodeSegmentLengthMismatchError {
    pub segment_length: usize,
    pub bytecode_length: usize,
}

#[derive(Debug)]
pub struct InvalidBytecodeSegmentError {
    pub visited_pc: u64,
    pub segment_start: u64,
}

#[derive(Debug)]
pub struct PcOutOfRangeError {
    pub pc: u64,
}
