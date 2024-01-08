mod backends;
pub mod core;
pub mod formats;
#[cfg(not(target_arch = "wasm32"))]
pub use anyhow::Result;

use crate::backends::openapi3_0::OpenApi30Backend;
use crate::core::VAR_REPLACE_CREDITS;
use crate::core::{Backend, Converter, CreateOperationParams, State, Variables};
use crate::formats::openapi;
use crate::formats::postman;
#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Default)]
pub struct TranspileOptions {
    pub format: TargetFormat,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn from_path(filename: &str, options: TranspileOptions) -> Result<String> {
    let collection = std::fs::read_to_string(filename)?;
    from_str(&collection, options)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn from_str(collection: &str, options: TranspileOptions) -> Result<String> {
    let postman_spec: postman::Spec = serde_json::from_str(collection)?;
    let oas_spec = Transpiler::transpile(postman_spec);
    let oas_definition = match options.format {
        TargetFormat::Json => openapi::to_json(&oas_spec),
        TargetFormat::Yaml => openapi::to_yaml(&oas_spec),
    }?;
    Ok(oas_definition)
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
#[cfg(target_arch = "wasm32")]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn transpile(collection: JsValue) -> std::result::Result<JsValue, JsValue> {
    let postman_spec: std::result::Result<postman::Spec, _> =
        postman::Spec::deserialize(serde_wasm_bindgen::Deserializer::from(collection));
    match postman_spec {
        Ok(s) => {
            let oas_spec = Transpiler::transpile(s);
            let serializer = serde_wasm_bindgen::Serializer::json_compatible();
            let s = oas_spec.serialize(&serializer);
            //let s = serde_wasm_bindgen::to_value(&oas_spec);
            match s {
                Ok(s) => Ok(s),
                Err(err) => Err(JsValue::from_str(&err.to_string())),
            }
        }
        Err(err) => Err(JsValue::from_str(&err.to_string())),
    }
}

#[derive(PartialEq, Eq, Debug, Default)]
pub enum TargetFormat {
    Json,
    #[default]
    Yaml,
}

impl std::str::FromStr for TargetFormat {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(TargetFormat::Json),
            "yaml" => Ok(TargetFormat::Yaml),
            _ => Err("invalid format"),
        }
    }
}

#[derive(Default)]
pub struct Transpiler;

impl Transpiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn transpile(spec: postman::Spec) -> openapi::OpenApi {
        let description: Option<Cow<str>> = spec.info.description.as_ref().map(|d| d.into());

        let mut variable_map = BTreeMap::<Cow<str>, serde_json::value::Value>::new();
        if let Some(var) = spec.variable {
            for v in var {
                if let Some(v_name) = v.key {
                    if let Some(v_val) = v.value {
                        if v_val != serde_json::Value::String("".to_string()) {
                            variable_map.insert(v_name, v_val);
                        }
                    }
                }
            }
        };

        let variables = Variables {
            map: variable_map,
            replace_credits: VAR_REPLACE_CREDITS,
        };

        let hierarchy = Vec::<Cow<str>>::new();
        let auth_stack = Vec::<&postman::Auth>::new();

        let state = &mut State {
            auth_stack,
            hierarchy,
            variables,
        };

        let mut oas = OpenApi30Backend::generate(spec.info.name, description);
        let mut backend = OpenApi30Backend {
            oas: &mut oas,
            operation_ids: BTreeMap::<String, usize>::new(),
        };
        let mut transpiler = Transpiler {};

        if let Some(auth) = spec.auth.as_ref() {
            state.auth_stack.push(auth);
            backend.create_security(state, auth);
        }

        transpiler.convert_collection(&mut backend, state, &spec.item);
        openapi::OpenApi::V3_0(Box::new(oas))
    }
}

impl Converter for Transpiler {
    fn convert_collection<'a, T: Backend<'a>>(
        &mut self,
        backend: &mut T,
        state: &mut State<'a>,
        items: &'a [postman::Items],
    ) {
        for item in items {
            if let Some(sub_items) = &item.item {
                let name = match &item.name {
                    Some(n) => n,
                    None => &Cow::Borrowed("<folder>"),
                };
                let description: Option<Cow<'a, str>> = item.description.as_ref().map(|d| d.into());

                if let Some(auth) = item.auth.as_ref() {
                    state.auth_stack.push(auth);
                }

                self.convert_folder(backend, state, sub_items, name.clone(), description);

                if item.auth.is_some() {
                    state.auth_stack.pop();
                }
            } else {
                let name = match &item.name {
                    Some(n) => n.clone(),
                    None => Cow::Borrowed("<request>"),
                };
                self.convert_request(backend, state, item, name);
            }
        }
    }

    fn convert_folder<'a, T: Backend<'a>>(
        &mut self,
        backend: &mut T,
        state: &mut State<'a>,
        items: &'a [postman::Items],
        name: Cow<'a, str>,
        description: Option<Cow<'a, str>>,
    ) {
        backend.create_tag(state, name.clone(), description);
        state.hierarchy.push(name);

        self.convert_collection(backend, state, items);

        state.hierarchy.pop();
    }

    fn convert_request<'a, T: Backend<'a>>(
        &mut self,
        backend: &mut T,
        state: &mut State<'a>,
        item: &'a postman::Items,
        name: Cow<'a, str>,
    ) {
        if let Some(postman::RequestUnion::RequestClass(request)) = &item.request {
            if let Some(postman::Url::UrlClass(u)) = &request.url {
                if let Some(postman::Host::StringArray(parts)) = &u.host {
                    backend.create_server(state, u, parts);
                }

                let path_elements = match &u.path {
                    Some(postman::UrlPath::UnionArray(p)) => Some(p),
                    _ => None,
                };

                let auth = if let Some(auth) = &request.auth {
                    Some(auth)
                } else if !state.auth_stack.is_empty() {
                    state.auth_stack.last().copied()
                } else {
                    None
                };

                let params = CreateOperationParams {
                    item,
                    request,
                    request_name: name.clone(),
                    url: u,
                    path_elements,
                    auth,
                };
                backend.create_operation(state, params)
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use openapi::v3_0::{self as openapi3, MediaTypeExample, ObjectOrReference, Parameter, Schema};
    use openapi::OpenApi;
    use postman::Spec;

    #[test]
    fn it_preserves_order_on_paths() {
        let fixture = get_fixture("echo.postman.json");
        let spec: Spec = serde_json::from_str(&fixture).unwrap();
        let oas = Transpiler::transpile(spec);
        let ordered_paths = [
            "/get",
            "/post",
            "/put",
            "/patch",
            "/delete",
            "/headers",
            "/response-headers",
            "/basic-auth",
            "/digest-auth",
            "/auth/hawk",
            "/oauth1",
            "/cookies/set",
            "/cookies",
            "/cookies/delete",
            "/status/200",
            "/stream/5",
            "/delay/2",
            "/encoding/utf8",
            "/gzip",
            "/deflate",
            "/ip",
            "/time/now",
            "/time/valid",
            "/time/format",
            "/time/unit",
            "/time/add",
            "/time/subtract",
            "/time/start",
            "/time/object",
            "/time/before",
            "/time/after",
            "/time/between",
            "/time/leap",
            "/transform/collection",
            "/{method}/hello",
        ];
        let OpenApi::V3_0(s) = oas;
        let keys = s.paths.keys().enumerate();
        for (i, k) in keys {
            assert_eq!(k, ordered_paths[i])
        }
    }

    #[test]
    fn it_uses_the_correct_content_type_for_form_urlencoded_data() {
        let fixture = get_fixture("echo.postman.json");
        let spec: Spec = serde_json::from_str(&fixture).unwrap();
        let oas = Transpiler::transpile(spec);
        match oas {
            OpenApi::V3_0(oas) => {
                let b = oas
                    .paths
                    .get("/post")
                    .unwrap()
                    .post
                    .as_ref()
                    .unwrap()
                    .request_body
                    .as_ref()
                    .unwrap();
                if let ObjectOrReference::Object(b) = b {
                    assert!(b.content.contains_key("application/x-www-form-urlencoded"));
                }
            }
        }
    }

    #[test]
    fn it_generates_headers_from_the_request() {
        let fixture = get_fixture("echo.postman.json");
        let spec: Spec = serde_json::from_str(&fixture).unwrap();
        let oas = Transpiler::transpile(spec);
        match oas {
            OpenApi::V3_0(oas) => {
                let params = oas
                    .paths
                    .get("/headers")
                    .unwrap()
                    .get
                    .as_ref()
                    .unwrap()
                    .parameters
                    .as_ref()
                    .unwrap();
                let header = params
                    .iter()
                    .find(|p| {
                        if let ObjectOrReference::Object(p) = p {
                            p.location == "header"
                        } else {
                            false
                        }
                    })
                    .unwrap();
                let expected = ObjectOrReference::Object(Parameter {
                    name: "my-sample-header".to_owned(),
                    location: "header".to_owned(),
                    description: Some("My Sample Header".to_owned()),
                    schema: Some(Schema {
                        schema_type: Some("string".to_owned()),
                        example: Some(serde_json::Value::String(
                            "Lorem ipsum dolor sit amet".to_owned(),
                        )),
                        ..Schema::default()
                    }),
                    ..Parameter::default()
                });
                assert_eq!(header, &expected);
            }
        }
    }

    #[test]
    fn it_generates_root_path_when_no_path_exists_in_collection() {
        let fixture = get_fixture("only-root-path.postman.json");
        let spec: Spec = serde_json::from_str(&fixture).unwrap();
        let oas = Transpiler::transpile(spec);
        match oas {
            OpenApi::V3_0(oas) => {
                assert!(oas.paths.contains_key("/"));
            }
        }
    }

    #[test]
    fn it_parses_graphql_request_bodies() {
        let fixture = get_fixture("graphql.postman.json");
        let spec: Spec = serde_json::from_str(&fixture).unwrap();
        let oas = Transpiler::transpile(spec);
        match oas {
            OpenApi::V3_0(oas) => {
                let body = oas
                    .paths
                    .get("/")
                    .unwrap()
                    .post
                    .as_ref()
                    .unwrap()
                    .request_body
                    .as_ref()
                    .unwrap();

                if let ObjectOrReference::Object(body) = body {
                    assert!(body.content.contains_key("application/json"));
                    let content = body.content.get("application/json").unwrap();
                    let schema = content.schema.as_ref().unwrap();
                    if let ObjectOrReference::Object(schema) = schema {
                        let props = schema.properties.as_ref().unwrap();
                        assert!(props.contains_key("query"));
                        assert!(props.contains_key("variables"));
                    }
                    let examples = content.examples.as_ref().unwrap();
                    if let MediaTypeExample::Example { example } = examples {
                        let example: serde_json::Map<String, serde_json::Value> =
                            serde_json::from_value(example.clone()).unwrap();
                        assert!(example.contains_key("query"));
                        assert!(example.contains_key("variables"));
                    }
                }
            }
        }
    }

    #[test]
    fn it_collapses_duplicate_query_params() {
        let fixture = get_fixture("duplicate-query-params.postman.json");
        let spec: Spec = serde_json::from_str(&fixture).unwrap();
        let oas = Transpiler::transpile(spec);
        match oas {
            OpenApi::V3_0(oas) => {
                let query_param_names = oas
                    .paths
                    .get("/v2/json-rpc/{site id}")
                    .unwrap()
                    .post
                    .as_ref()
                    .unwrap()
                    .parameters
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter_map(|p| match p {
                        ObjectOrReference::Object(p) => {
                            if p.location == "query" {
                                Some(p.name.clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect::<Vec<String>>();

                assert!(!query_param_names.is_empty());

                let duplicates = (1..query_param_names.len())
                    .filter_map(|i| {
                        if query_param_names[i..].contains(&query_param_names[i - 1]) {
                            Some(query_param_names[i - 1].clone())
                        } else {
                            None
                        }
                    })
                    .collect::<std::collections::HashSet<String>>();

                assert!(duplicates.is_empty(), "duplicates: {duplicates:?}");
            }
        }
    }

    #[test]
    fn it_uses_the_security_requirement_on_operations() {
        let fixture = get_fixture("echo.postman.json");
        let spec: Spec = serde_json::from_str(&fixture).unwrap();
        let oas = Transpiler::transpile(spec);
        match oas {
            OpenApi::V3_0(oas) => {
                let sr1 = oas
                    .paths
                    .get("/basic-auth")
                    .unwrap()
                    .get
                    .as_ref()
                    .unwrap()
                    .security
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    sr1.get(0)
                        .unwrap()
                        .requirement
                        .as_ref()
                        .unwrap()
                        .get("basicAuth"),
                    Some(&vec![])
                );
                let sr1 = oas
                    .paths
                    .get("/digest-auth")
                    .unwrap()
                    .get
                    .as_ref()
                    .unwrap()
                    .security
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    sr1.get(0)
                        .unwrap()
                        .requirement
                        .as_ref()
                        .unwrap()
                        .get("digestAuth"),
                    Some(&vec![])
                );

                let schemes = oas.components.unwrap().security_schemes.unwrap();
                let basic = schemes.get("basicAuth").unwrap();
                if let ObjectOrReference::Object(basic) = basic {
                    match basic {
                        openapi3::SecurityScheme::Http { scheme, .. } => {
                            assert_eq!(scheme, "basic");
                        }
                        _ => panic!("Expected Http Security Scheme"),
                    }
                }
                let digest = schemes.get("digestAuth").unwrap();
                if let ObjectOrReference::Object(digest) = digest {
                    match digest {
                        openapi3::SecurityScheme::Http { scheme, .. } => {
                            assert_eq!(scheme, "digest");
                        }
                        _ => panic!("Expected Http Security Scheme"),
                    }
                }
            }
        }
    }

    fn get_fixture(filename: &str) -> String {
        use std::fs;

        let filename: std::path::PathBuf =
            [env!("CARGO_MANIFEST_DIR"), "./tests/fixtures/", filename]
                .iter()
                .collect();
        let file = filename.into_os_string().into_string().unwrap();
        fs::read_to_string(file).unwrap()
    }
}
