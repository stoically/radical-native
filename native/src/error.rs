#[derive(Debug)]
pub(crate) enum Error {
    ParseEvent { error: String },
    ParseCheckpointDirection { error: String },
    ParseLoadDirection { error: String },
    ParseSearchObject { error: String },
    Seshat { error: String },
    Serde { error: String },
    MpscRec { error: String },
    MissingField { error: String },
    UnknownMethod { error: String },
    Io { error: String },
    IndexNotInitialized,
    UserDataDirNotFound,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::Io {
            error: format!("{}", error),
        }
    }
}

impl From<seshat::Error> for Error {
    fn from(error: seshat::Error) -> Error {
        Error::Seshat {
            error: format!("{}", error),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::Serde {
            error: format!("{}", error),
        }
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(error: std::sync::mpsc::RecvError) -> Error {
        Error::MpscRec {
            error: format!("{}", error),
        }
    }
}
