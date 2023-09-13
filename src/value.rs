use std::collections::BTreeMap;
#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use js_sys::{JsString, Object};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use serde::ser::Serialize;
#[cfg(not(target_arch = "wasm32"))]
use serde::de::Deserialize;
#[cfg(not(target_arch = "wasm32"))]
use serde::ser::Serialize;

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct Error {
    inner: serde_wasm_bindgen::Error
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct Error {
    inner: serde_json::Error
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Default)]
pub struct Value {
    inner: JsValue,
}

#[cfg(target_arch = "wasm32")]
impl Value {
    pub fn from_str(s: &str) -> Value {
        Value {
            inner: JsValue::from_str(s)
        }
    }

    pub fn value_from_str(value: &str) -> Result<Value, Error> {
        match js_sys::JSON::parse(value) {
            Ok(v) => {
                Ok(Value { inner: v.into() })
            }
            Err(e) => Err(Error { inner: e.into() })
        }
    }

    pub fn to_value<T: Serialize + ?Sized>(value: &T) -> Result<Value, Error> {
        match serde_wasm_bindgen::to_value(value) {
            Ok(v) => {
                match v {
                    Ok(v) => Ok(Value { inner: v }),
                    Err(e) => Err(Error { inner: e.into() })
                }
            }
            Err(e) => Err(Error { inner: e })
        }
    }

    pub fn is_object(&self) -> bool {
        self.inner.is_object()
    }

    pub fn is_array(&self) -> bool {
        self.inner.is_array()
    }

    pub fn is_string(&self) -> bool {
        self.inner.is_string()
    }

    pub fn is_number(&self) -> bool {
        match self.inner.as_f64() {
            Some(_) => true,
            None => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self.inner.as_bool() {
            Some(_) => true,
            None => false,
        }
    }

    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    pub fn as_string(&self) -> Option<String> {
        self.inner.as_string()
    }
}

#[cfg(target_arch = "wasm32")]
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(*self)
    }
}

#[cfg(target_arch = "wasm32")]
impl From<BTreeMap<String, Value>> for Value {
    fn from(f: BTreeMap<String, Value>) -> Value {
        let map: HashMap<String, Value> = f.into_iter().collect();
        Value { inner: serde_wasm_bindgen::to_value(map) }
    }
}


#[cfg(target_arch = "wasm32")]
impl Into<BTreeMap<String, Value>> for Value {
    fn into(self) -> BTreeMap<String, Value> {
        let mut r = BTreeMap::<String, Value>::new();
        let v: JsString = self.inner.into();
        if v.is_object() {
            let map: core::result::Result<HashMap<String, JsValue>, serde_wasm_bindgen::Error> = serde_wasm_bindgen::from_value(self.inner);
            if let Ok(map) = map {
                for (key, value) in map {
                    r.insert(key, Value::to_value(&value).unwrap_or(Value::from_str("")));
                }
            }
        }
        r
    }
}

#[cfg(target_arch = "wasm32")]
impl Into<Vec<Value>> for Value {
    fn into(self) -> Vec<Value> {
        let mut r = Vec::<Value>::new();
        if self.inner.is_array() {
            let arr: core::result::Result<Vec<JsValue>, serde_wasm_bindgen::Error> = serde_wasm_bindgen::from_value(self.inner);
            if let Ok(arr) = arr {
                for value in arr {
                    r.push(Value::to_value(&value).unwrap_or(Value::from_str("")));
                }
            }
        }
        r
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Value {
    inner: serde_json::value::Value,
}

#[cfg(not(target_arch = "wasm32"))]
impl Value {
    pub fn value_from_str(value: &str) -> Result<Value, Error> {
        match serde_json::from_str::<serde_json::value::Value>(value) {
            Ok(v) => Ok(Value { inner: v }),
            Err(e) => Err(Error { inner: e.into() })
        }
    }

    pub fn from_str(s: &str) -> Value {
        Value {
            inner: serde_json::value::Value::String(s.to_string()),
        }
    }

    pub fn deserialize_from_str<'a, T: Deserialize<'a>>(s: &'a str) -> Result<T, Error> {
        let res: serde_json::Result<T> = serde_json::from_str(s);
            match res {
            Ok(v) => Ok(v),
            Err(e) => Err(Error { inner: e })
            }
    }

    pub fn to_value<T: Serialize>(value: T) -> Result<Value, Error> {
        match serde_json::to_value(value) {
            Ok(v) => Ok(Value { inner: v }),
            Err(e) => Err(Error { inner: e })
        }
    }

    pub fn from_value<T: Serialize>(value: T) -> Result<Value, Error> {
        match serde_json::to_value(value) {
            Ok(v) => Ok(Value { inner: v }),
            Err(e) => Err(Error { inner: e })
        }
    }

    pub fn is_object(&self) -> bool {
        self.inner.is_object()
    }

    pub fn is_array(&self) -> bool {
        self.inner.is_array()
    }

    pub fn is_string(&self) -> bool {
        self.inner.is_string()
    }

    pub fn is_number(&self) -> bool {
        self.inner.is_number()
    }

    pub fn is_bool(&self) -> bool {
        self.inner.is_boolean()
    }

    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    pub fn as_string(&self) -> Option<String> {
        match &self.inner {
            serde_json::value::Value::String(s) => Some(s.clone()),
            _ => None
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<BTreeMap<String, Value>> for Value {
    fn from(f: BTreeMap<String, Value>) -> Value {
        let mut map = serde_json::Map::<String, serde_json::value::Value>::new(); 
        for (key, val) in f {
            map.insert(key, val.inner);
        }
        Value { inner: serde_json::value::Value::Object(map) }
    }
}


#[cfg(not(target_arch = "wasm32"))]
impl Into<BTreeMap<String, Value>> for Value {
    fn into(self) -> BTreeMap<String, Value> {
        let mut r = BTreeMap::<String, Value>::new();
        match self.inner {
            serde_json::value::Value::Object(map) => {
                for (key, value) in map {
                    r.insert(key, Value::to_value(&value).unwrap_or(Value::from_str("")));
                }
                r
            },
            _ => r
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Into<Vec<Value>> for Value {
    fn into(self) -> Vec<Value> {
        let mut r = Vec::<Value>::new();
        match self.inner {
            serde_json::value::Value::Array(arr) => {
                for value in arr {
                    r.push(Value::to_value(&value).unwrap_or(Value::from_str("")));
                }
                r
            },
            _ => r
        }
    }
}
