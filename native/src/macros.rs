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
