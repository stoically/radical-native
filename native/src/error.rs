// Copyright 2020 stoically@protonmail.com
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
