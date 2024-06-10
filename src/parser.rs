// TODO: line and position information for certain errors?
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("input is not a ACMI file")]
    InvalidFileType,
    #[error("invalid version, expected ACMI v2.x")]
    InvalidVersion,
    #[error("error reading input")]
    Io(#[from] std::io::Error),
    #[error("unexpected end of line")]
    Eol,
    #[error("object id is not a u64")]
    InvalidId(#[from] std::num::ParseIntError),
    #[error("expected numeric")]
    InvalidNumeric(#[from] std::num::ParseFloatError),
    #[error("could not find expected delimiter `{0}`")]
    MissingDelimiter(char),
    #[error("failed to parse event")]
    InvalidEvent,
    #[error("encountered invalid coordinate format")]
    InvalidCoordinateFormat,
    // #[error("error reading zip compressed input")]
    // Zip(#[from] zip::result::ZipError),
}
