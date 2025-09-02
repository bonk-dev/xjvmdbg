#[derive(Debug)]
pub enum JdwpErrorCode {}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    JdwpError(JdwpErrorCode),
    ParsingError { message: String },
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IoError(value)
    }
}
