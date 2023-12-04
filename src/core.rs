use std::{borrow::Cow, collections::BTreeMap};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

use crate::formats::postman;

#[cfg(not(target_arch = "wasm32"))]
pub type WrappedJson = SerdeJsonValue;

#[cfg(target_arch = "wasm32")]
pub type WrappedJson = WasmJsValue;

pub static VAR_REPLACE_CREDITS: usize = 20;

pub trait JsonValue {
    fn is_array(&self) -> bool;
    fn is_boolean(&self) -> bool;
    fn is_null(&self) -> bool;
    fn is_number(&self) -> bool;
    fn is_object(&self) -> bool;
    fn is_string(&self) -> bool;
    fn as_str(&self) -> Option<&str>;
    fn from_str(s: &str) -> Self
    where
        Self: Sized;
}

#[cfg(not(target_arch = "wasm32"))]
type SerdeJsonValue = serde_json::Value;

#[cfg(not(target_arch = "wasm32"))]
impl JsonValue for SerdeJsonValue {
    fn is_array(&self) -> bool {
        self.is_array()
    }

    fn is_boolean(&self) -> bool {
        self.is_boolean()
    }

    fn is_null(&self) -> bool {
        self.is_null()
    }

    fn is_number(&self) -> bool {
        self.is_number()
    }

    fn is_object(&self) -> bool {
        self.is_object()
    }

    fn is_string(&self) -> bool {
        self.is_string()
    }

    fn as_str(&self) -> Option<&str> {
        self.as_str()
    }

    fn from_str(s: &str) -> Self {
        SerdeJsonValue::String(s.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
pub type WasmJsValue = JsValue;

#[cfg(target_arch = "wasm32")]
impl JsonValue for WasmJsValue {
    fn is_array(&self) -> bool {
        self.is_array()
    }

    fn is_boolean(&self) -> bool {
        match self.as_bool() {
            Some(_) => true,
            None => false,
        }
    }

    fn is_null(&self) -> bool {
        self.is_null()
    }

    fn is_number(&self) -> bool {
        match self.as_f64() {
            Some(_) => true,
            None => false,
        }
    }

    fn is_object(&self) -> bool {
        self.is_object()
    }

    fn is_string(&self) -> bool {
        self.is_string()
    }

    fn as_str(&self) -> Option<&str> {
        self.as_str()
    }

    fn from_str(s: &str) -> Self {
        WasmJsValue::from_str(s)
    }
}

enum CaptureState {
    Start,
    VariableOpen,
    Variable,
    VariableClose,
}

pub struct Capture<'a> {
    pub start: usize,
    pub end: usize,
    pub value: Cow<'a, str>,
}

pub fn capture_openapi_path_variables(s: &str) -> Option<Vec<Capture<'_>>> {
    let mut captures = Vec::new();
    let mut state = CaptureState::Start;
    let mut state_start = 0;
    s.chars().enumerate().for_each(|(i, c)| {
        state = match state {
            CaptureState::Start => match c {
                '{' => {
                    state_start = i + 1;
                    CaptureState::Variable
                }
                _ => CaptureState::Start,
            },
            CaptureState::Variable => match c {
                '}' => {
                    captures.push(Capture {
                        start: state_start,
                        end: i - 1,
                        value: Cow::Borrowed(&s[state_start..i]),
                    });
                    CaptureState::Start
                }
                '{' => CaptureState::VariableOpen,
                _ => CaptureState::Variable,
            },
            _ => CaptureState::Start,
        }
    });

    if !captures.is_empty() {
        Some(captures)
    } else {
        None
    }
}

pub fn capture_collection_variables(s: &str) -> Option<Vec<Capture<'_>>> {
    let mut captures = Vec::new();
    let mut state = CaptureState::Start;
    let mut state_start = 0;
    s.chars().enumerate().for_each(|(i, c)| {
        state = match state {
            CaptureState::Start => match c {
                '{' => CaptureState::VariableOpen,
                _ => CaptureState::Start,
            },
            CaptureState::VariableOpen => match c {
                '{' => {
                    state_start = i + 1;
                    CaptureState::Variable
                }
                _ => CaptureState::Start,
            },
            CaptureState::Variable => match c {
                '}' => CaptureState::VariableClose,
                '{' => CaptureState::VariableOpen,
                _ => CaptureState::Variable,
            },
            CaptureState::VariableClose => match c {
                '}' => {
                    captures.push(Capture {
                        start: state_start,
                        end: i - 2,
                        value: Cow::Borrowed(&s[state_start..i - 1]),
                    });
                    CaptureState::Start
                }
                '{' => CaptureState::VariableOpen,
                _ => CaptureState::Start,
            },
        }
    });

    if !captures.is_empty() {
        Some(captures)
    } else {
        None
    }
}

#[derive(Default)]
pub struct State<'a, T: JsonValue> {
    pub auth_stack: Vec<&'a postman::Auth<'a, T>>,
    pub hierarchy: Vec<Cow<'a, str>>,
    pub variables: Variables<'a, T>,
}

#[derive(Debug, Clone)]
pub struct CreateOperationParams<'a, T: JsonValue> {
    pub auth: Option<&'a postman::Auth<'a, T>>,
    pub item: &'a postman::Items<'a, T>,
    pub request: &'a postman::RequestClass<'a, T>,
    pub request_name: Cow<'a, str>,
    pub path_elements: Option<&'a Vec<postman::PathElement<'a>>>,
    pub url: &'a postman::UrlClass<'a, T>,
}

#[derive(Debug, Default, Clone)]
pub struct Variables<'a, T: JsonValue> {
    pub map: BTreeMap<Cow<'a, str>, T>,
    pub replace_credits: usize,
}

impl<'a, T: JsonValue> Variables<'a, T> {
    pub fn resolve(&self, segment: Cow<'a, str>) -> String {
        self.resolve_with_credits(segment, self.replace_credits)
    }

    pub fn resolve_with_credits(
        &self,
        segment: Cow<'a, str>,
        sub_replace_credits: usize,
    ) -> String {
        self.resolve_with_credits_and_replace_fn(segment, sub_replace_credits, |s| s)
    }

    pub fn resolve_with_credits_and_replace_fn(
        &self,
        segment: Cow<'a, str>,
        sub_replace_credits: usize,
        replace_fn: fn(String) -> String,
    ) -> String {
        let s = segment.to_string();

        if sub_replace_credits == 0 {
            return s;
        }

        if let Some(cap) = capture_collection_variables(&s) {
            for capture in cap {
                if let Some(v) = self.map.get(capture.value.as_ref()) {
                    if let Some(v2) = v.as_str() {
                        return self.resolve_with_credits(
                            Cow::Owned(s.replace(
                                format!("{{{{{value}}}}}", value = capture.value).as_str(),
                                v2,
                            )),
                            sub_replace_credits - 1,
                        );
                    }
                }
            }
        }

        replace_fn(s)
    }
}

pub trait Frontend {
    fn convert<'a, TJson: JsonValue, T: Backend<'a, TJson>>(
        &mut self,
        backend: &mut T,
        state: &mut State<'a, TJson>,
        items: &'a [postman::Items<'a, TJson>],
    );
    fn convert_folder<'a, TJson: JsonValue, T: Backend<'a, TJson>>(
        &mut self,
        backend: &mut T,
        state: &mut State<'a, TJson>,
        items: &'a [postman::Items<'a, TJson>],
        name: Cow<'a, str>,
        description: Option<Cow<'a, str>>,
    );
    fn convert_request<'a, TJson: JsonValue, T: Backend<'a, TJson>>(
        &mut self,
        backend: &mut T,
        state: &mut State<'a, TJson>,
        item: &'a postman::Items<'a, TJson>,
        name: Cow<'a, str>,
    );
}

pub trait Backend<'a, T: JsonValue> {
    fn create_server(
        &mut self,
        state: &mut State<T>,
        url: &postman::UrlClass<T>,
        parts: &[Cow<str>],
    );
    fn create_tag(&mut self, state: &mut State<T>, name: Cow<str>, description: Option<Cow<str>>);
    fn create_operation<'cp: 'a>(
        &mut self,
        state: &mut State<T>,
        params: CreateOperationParams<'cp, T>,
    );
    fn create_security(&mut self, state: &mut State<T>, auth: &postman::Auth<T>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_captures_one_collection_variable() {
        let captures = capture_collection_variables("{{foo}}").unwrap();
        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0].start, 2);
        assert_eq!(captures[0].end, 4);
        assert_eq!(captures[0].value, "foo");
    }

    #[test]
    fn it_captures_many_collection_variables() {
        let captures = capture_collection_variables("{{foo}}{{bar}}").unwrap();
        assert_eq!(captures.len(), 2);

        assert_eq!(captures[0].start, 2);
        assert_eq!(captures[0].end, 4);
        assert_eq!(captures[0].value, "foo");

        assert_eq!(captures[1].start, 9);
        assert_eq!(captures[1].end, 11);
        assert_eq!(captures[1].value, "bar");
    }

    #[test]
    fn it_only_captures_nested_collection_variables() {
        let captures = capture_collection_variables("{{foo{{bar}}}}{{{{bar}}foo}}").unwrap();
        assert_eq!(captures.len(), 2);

        assert_eq!(captures[0].start, 7);
        assert_eq!(captures[0].end, 9);
        assert_eq!(captures[0].value, "bar");

        assert_eq!(captures[1].start, 18);
        assert_eq!(captures[1].end, 20);
        assert_eq!(captures[1].value, "bar");
    }

    #[test]
    fn it_captures_one_openapi_path_variable() {
        let captures = capture_openapi_path_variables("{foo}").unwrap();
        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0].start, 1);
        assert_eq!(captures[0].end, 3);
        assert_eq!(captures[0].value, "foo");
    }

    #[test]
    fn it_captures_multiple_openapi_path_variables() {
        let captures = capture_openapi_path_variables("{foo}/{bar}").unwrap();
        assert_eq!(captures.len(), 2);
        assert_eq!(captures[0].start, 1);
        assert_eq!(captures[0].end, 3);
        assert_eq!(captures[0].value, "foo");

        assert_eq!(captures[1].start, 7);
        assert_eq!(captures[1].end, 9);
        assert_eq!(captures[1].value, "bar");
    }
}
