use std::{borrow::Cow, collections::BTreeMap};

use crate::formats::postman;

#[cfg(not(target_arch = "wasm32"))]
pub type Map<V> = indexmap::IndexMap<String, V>;
#[cfg(target_arch = "wasm32")]
pub type Map = js_sys::Map;
#[cfg(not(target_arch = "wasm32"))]
pub type Set<T> = indexmap::IndexSet<T>;
#[cfg(target_arch = "wasm32")]
pub type Set = js_sys::Set;

pub static VAR_REPLACE_CREDITS: usize = 20;

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
pub struct State<'a> {
    pub auth_stack: Vec<&'a postman::Auth<'a>>,
    pub hierarchy: Vec<Cow<'a, str>>,
    pub variables: Variables<'a>,
}

#[derive(Debug, Clone)]
pub struct CreateOperationParams<'a> {
    pub auth: Option<&'a postman::Auth<'a>>,
    pub item: &'a postman::Items<'a>,
    pub request: &'a postman::RequestClass<'a>,
    pub request_name: Cow<'a, str>,
    pub path_elements: Option<&'a Vec<postman::PathElement<'a>>>,
    pub url: &'a postman::UrlClass<'a>,
}

#[derive(Debug, Default, Clone)]
pub struct Variables<'a> {
    pub map: BTreeMap<Cow<'a, str>, serde_json::value::Value>,
    pub replace_credits: usize,
}

impl<'a> Variables<'a> {
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

pub trait Converter {
    fn convert_collection<'a, T: Backend<'a>>(
        &mut self,
        backend: &mut T,
        state: &mut State<'a>,
        items: &'a [postman::Items],
    );
    fn convert_folder<'a, T: Backend<'a>>(
        &mut self,
        backend: &mut T,
        state: &mut State<'a>,
        items: &'a [postman::Items],
        name: Cow<'a, str>,
        description: Option<Cow<'a, str>>,
    );
    fn convert_request<'a, T: Backend<'a>>(
        &mut self,
        backend: &mut T,
        state: &mut State<'a>,
        item: &'a postman::Items,
        name: Cow<'a, str>,
    );
}

pub trait Backend<'a> {
    fn create_server(&mut self, state: &mut State, url: &postman::UrlClass, parts: &[Cow<str>]);
    fn create_tag(&mut self, state: &mut State, name: Cow<str>, description: Option<Cow<str>>);
    fn create_operation<'cp: 'a>(&mut self, state: &mut State, params: CreateOperationParams<'cp>);
    fn create_security(&mut self, state: &mut State, auth: &postman::Auth);
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
