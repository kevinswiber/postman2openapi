use std::collections::BTreeMap;

use crate::formats::postman;

pub static VAR_REPLACE_CREDITS: usize = 20;

lazy_static! {
    pub static ref VARIABLE_RE: regex::Regex = regex::Regex::new(r"\{\{([^{}]*?)\}\}").unwrap();
    pub static ref URI_TEMPLATE_VARIABLE_RE: regex::Regex =
        regex::Regex::new(r"\{([^{}]*?)\}").unwrap();
}

#[derive(Default)]
pub struct State<'a> {
    pub auth_stack: Vec<&'a postman::Auth<'a>>,
    pub hierarchy: Vec<&'a str>,
    pub variables: Variables<'a>,
}

#[derive(Debug, Clone)]
pub struct CreateOperationParams<'a> {
    pub auth: Option<&'a postman::Auth<'a>>,
    pub item: &'a postman::Items<'a>,
    pub request: &'a postman::RequestClass<'a>,
    pub request_name: &'a str,
    pub path_elements: &'a [postman::PathElement<'a>],
    pub url: &'a postman::UrlClass<'a>,
}

#[derive(Debug, Default, Clone)]
pub struct Variables<'a> {
    pub map: BTreeMap<&'a str, serde_json::value::Value>,
    pub replace_credits: usize,
}

impl<'a> Variables<'a> {
    pub fn resolve(&self, segment: &str) -> String {
        self.resolve_with_credits(segment, self.replace_credits)
    }

    pub fn resolve_with_credits(&self, segment: &str, sub_replace_credits: usize) -> String {
        self.resolve_with_credits_and_replace_fn(segment, sub_replace_credits, |s| s)
    }

    pub fn resolve_with_credits_and_replace_fn(
        &self,
        segment: &str,
        sub_replace_credits: usize,
        replace_fn: fn(String) -> String,
    ) -> String {
        let s = segment.to_string();

        if sub_replace_credits == 0 {
            return s;
        }

        if let Some(cap) = VARIABLE_RE.captures(&s) {
            if cap.len() > 1 {
                for n in 1..cap.len() {
                    let capture = &cap[n];
                    if let Some(v) = self.map.get(capture) {
                        if let Some(v2) = v.as_str() {
                            let re = regex::Regex::new(&regex::escape(&cap[0])).unwrap();
                            return self.resolve_with_credits(
                                &re.replace_all(&s, v2),
                                sub_replace_credits - 1,
                            );
                        }
                    }
                }
            }
        }

        replace_fn(s)
    }
}

pub trait Frontend {
    fn convert<'a, T: Backend>(
        &mut self,
        backend: &mut T,
        state: &mut State<'a>,
        items: &'a [postman::Items],
    );
    fn convert_folder<'a, T: Backend>(
        &mut self,
        backend: &mut T,
        state: &mut State<'a>,
        items: &'a [postman::Items],
        name: &'a str,
        description: Option<&str>,
    );
    fn convert_request<'a, T: Backend>(
        &mut self,
        backend: &mut T,
        state: &mut State<'a>,
        item: &'a postman::Items,
        name: &str,
    );
}

pub trait Backend {
    fn create_server(&mut self, state: &mut State, url: &postman::UrlClass, parts: &[&str]);
    fn create_tag(&mut self, state: &mut State, name: &str, description: Option<&str>);
    fn create_operation(&mut self, state: &mut State, params: CreateOperationParams);
    fn create_security(&mut self, state: &mut State, auth: &postman::Auth);
}