#[derive(Debug)]
pub(crate) enum Error {
    ParseEvent { error: String },
    ParseCheckpointDirection { error: String },
    ParseLoadDirection { error: String },
    ParseSearchObject { error: String },
    SeshatError { error: String },
    SerdeError { error: String },
    MpscRecError { error: String },
    MissingField { error: String },
    UnknownMethod { error: String },
    IoError { error: String },
    IndexNotInitialized,
    CloseIndexBeforeDelete,
    UserDataDirNotFound,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::IoError {
            error: format!("{}", error),
        }
    }
}

impl From<seshat::Error> for Error {
    fn from(error: seshat::Error) -> Error {
        Error::SeshatError {
            error: format!("{}", error),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::SerdeError {
            error: format!("{}", error),
        }
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(error: std::sync::mpsc::RecvError) -> Error {
        Error::MpscRecError {
            error: format!("{}", error),
        }
    }
}
