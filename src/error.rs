#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    //Hound(hound::Error),
    Data(String),
    Com(String),
    Audio(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
    fn from(err: std::sync::mpsc::SendError<T>) -> Error {
        Error::Com(err.to_string())
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(err: std::sync::mpsc::RecvError) -> Error {
        Error::Com(err.to_string())
    }
}

impl From<std::sync::mpsc::TryRecvError> for Error {
    fn from(err: std::sync::mpsc::TryRecvError) -> Error {
        Error::Com(err.to_string())
    }
}
