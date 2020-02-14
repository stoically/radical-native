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

macro_rules! as_str {
    ($json:expr, $field:expr) => {
        match $json[$field].as_str() {
            Some(ret) => ret.to_string(),
            None => {
                return Err(Error::MissingField {
                    error: format!("Missing field '{}' in {}", $field, $json),
                })
            }
        }
    };
}

macro_rules! as_i64 {
    ($json:expr, $field:expr) => {
        match $json[$field].as_i64() {
            Some(ret) => ret,
            None => {
                return Err(Error::MissingField {
                    error: format!("Missing field '{}' in {}", $field, $json),
                })
            }
        }
    };
}

macro_rules! get {
    ($json:expr, $field:expr) => {
        match $json.get($field) {
            Some(ret) => ret,
            None => {
                return Err(Error::MissingField {
                    error: format!("Missing field '{}' in {}", $field, $json),
                })
            }
        }
    };
}
