use std::alloc::System;
use std::ops::Add;
use std::time::{Duration, SystemTime};

use chrono::NaiveDateTime;
use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::{err_inv, Error};

macro_rules! impl_segment {
    () => (
        pub fn new(json: Value) -> Self {
            Self {
                json
            }
        }

        pub fn get_str(&self, key: &str) -> Option<&str> {
            self.json.get(key)?.as_str()
        }

        pub fn get_i64(&self, key: &str) -> Option<i64> {
            self.json.get(key)?.as_i64()
        }

        pub fn get_u64(&self, key: &str) -> Option<u64> {
            self.json.get(key)?.as_u64()
        }

        pub fn get_f64(&self, key: &str) -> Option<f64> {
            self.json.get(key)?.as_f64()
        }

        pub fn get_bool(&self, key: &str) -> Option<bool> {
            self.json.get(key)?.as_bool()
        }

        pub fn get_object(&self, key: &str) -> Option<&Map<String, Value>> {
            self.json.get(key)?.as_object()
        }

        pub fn get_array(&self, key: &str) -> Option<&Vec<Value>> {
            self.json.get(key)?.as_array()
        }

        pub fn get_null(&self, key: &str) -> Option<()> {
            self.json.get(key)?.as_null()
        }

        pub fn load_into<T: DeserializeOwned>(&self) -> Result<T, Error> {
            Ok(serde_json::from_value::<T>(self.json.clone()).or(Err(err_inv("Failed to deserialize segment")))?)
        }
    )
}

pub type HeaderBodyRaw = Vec<u8>;
pub type Signature = Vec<u8>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    pub(crate) json: Value,
}

impl Header {
    impl_segment!();

    pub fn alg(&self) -> Option<&str> {
        self.get_str("alg")
    }

    pub fn enc(&self) -> Option<&str> {
        self.get_str("enc")
    }

    pub fn zip(&self) -> Option<&str> {
        self.get_str("zip")
    }

    pub fn jku(&self) -> Option<&str> {
        self.get_str("jku")
    }

    pub fn jkw(&self) -> Option<&str> {
        self.get_str("jkw")
    }

    pub fn kid(&self) -> Option<&str> {
        self.get_str("kid")
    }

    pub fn x5u(&self) -> Option<&str> {
        self.get_str("x5u")
    }

    pub fn x5c(&self) -> Option<&str> {
        self.get_str("x5c")
    }

    pub fn x5t(&self) -> Option<&str> {
        self.get_str("x5t")
    }

    pub fn typ(&self) -> Option<&str> {
        self.get_str("typ")
    }

    pub fn cty(&self) -> Option<&str> {
        self.get_str("cty")
    }

    pub fn ctit(&self) -> Option<&str> {
        self.get_str("crit")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub(crate) json: Value,
}

impl Payload {
    impl_segment!();

    pub fn iss(&self) -> Option<&str> {
        self.get_str("iss")
    }

    pub fn sub(&self) -> Option<&str> {
        self.get_str("sub")
    }

    pub fn aud(&self) -> Option<&str> {
        self.get_str("aud")
    }

    pub fn exp(&self) -> Option<u64> {
        self.get_u64("exp")
    }

    pub fn nbf(&self) -> Option<u64> {
        self.get_u64("nbf")
    }

    pub fn iat(&self) -> Option<u64> {
        self.get_u64("iat")
    }

    pub fn jti(&self) -> Option<&str> {
        self.get_str("jti")
    }

    pub fn expiry(&self) -> Option<SystemTime> {
        if let Some(time) = self.exp() {
            Some(SystemTime::UNIX_EPOCH.add(Duration::new(time, 0)))
        } else {
            None
        }
    }

    pub fn issued_at(&self) -> Option<SystemTime> {
        if let Some(time) = self.iat() {
            Some(SystemTime::UNIX_EPOCH.add(Duration::new(time, 0)))
        } else {
            None
        }
    }

    pub fn not_before(&self) -> Option<SystemTime> {
        if let Some(time) = self.nbf() {
            Some(SystemTime::UNIX_EPOCH.add(Duration::new(time, 0)))
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwt {
    header: Header,
    payload: Payload,
}

impl Jwt {
    pub fn new(header: Header, payload: Payload) -> Self {
        Jwt {
            header,
            payload,
        }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn expired(&self) -> Option<bool> {
        self.expired_date(SystemTime::now())
    }

    pub fn expired_date(&self, dt: SystemTime) -> Option<bool> {
        match self.payload.expiry() {
            Some(time) => { Some(dt > time) }
            None => { None }
        }
    }

    pub fn early(&self) -> Option<bool> {
        self.early_date(SystemTime::now())
    }

    pub fn early_date(&self, dt: SystemTime) -> Option<bool> {
        match self.payload.not_before() {
            Some(time) => { Some(dt < time) }
            None => { None }
        }
    }

    pub fn issued_by(&self, issuer: &str) -> Option<bool> {
        match self.payload.iss() {
            Some(t) => { Some(t == issuer) }
            None => { None }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::jwt::Payload;

    fn test_payload() {
        let json = json!({
            "iss": "test_iss",
            "exp": 123456f64,  // f64--not u64 since JSON uses f64
            "iat": 123f64,  // f64--not u64 since JSON uses f64
            "sub": "test_sub",
            "aud": "test_aud",
            "nbf": 456f64,  // f64--not u64 since JSON uses f64
            "jti": "test_jti",  // f64--not u64 since JSON uses f64
        });

        let payload = Payload {
            json,
        };

        assert_eq!("test_iss", payload.iss().unwrap());
        assert_eq!(123456u64, payload.exp().unwrap());
        assert_eq!(123u64, payload.iat().unwrap());
        assert_eq!("test_sub", payload.sub().unwrap());
        assert_eq!("test_aud", payload.aud().unwrap());
        assert_eq!(456u64, payload.nbf().unwrap());
        assert_eq!("test_jti", payload.jti().unwrap());
    }
}
