#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

pub mod openapi;
pub mod postman;

pub use anyhow::Result;
use convert_case::{Case, Casing};
#[cfg(target_arch = "wasm32")]
use gloo_utils::format::JsValueSerdeExt;
use indexmap::{IndexMap, IndexSet};
use openapi::v3_0::{self as openapi3, ObjectOrReference, Parameter, SecurityRequirement};
use postman::AuthType;
use std::collections::BTreeMap;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

static VAR_REPLACE_CREDITS: usize = 20;

lazy_static! {
    static ref VARIABLE_RE: regex::Regex = regex::Regex::new(r"\{\{([^{}]*?)\}\}").unwrap();
    static ref URI_TEMPLATE_VARIABLE_RE: regex::Regex =
        regex::Regex::new(r"\{([^{}]*?)\}").unwrap();
}

#[derive(Default)]
pub struct TranspileOptions {
    pub format: TargetFormat,
}

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

#[cfg(target_arch = "wasm32")]
pub fn from_str(collection: &str, options: TranspileOptions) -> Result<String> {
    let postman_spec: postman::Spec = serde_json::from_str(collection)?;
    let oas_spec = Transpiler::transpile(postman_spec);
    match options.format {
        TargetFormat::Json => openapi::to_json(&oas_spec).map_err(|err| err.into()),
        TargetFormat::Yaml => Err(anyhow::anyhow!(
            "YAML is not supported for WebAssembly. Please convert from YAML to JSON."
        )),
    }
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
    let postman_spec: std::result::Result<postman::Spec, serde_json::Error> =
        collection.into_serde();
    match postman_spec {
        Ok(s) => {
            let oas_spec = Transpiler::transpile(s);
            let oas_definition = JsValue::from_serde(&oas_spec);
            match oas_definition {
                Ok(val) => Ok(val),
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

struct OpenApi30Backend<'a> {
    oas: &'a mut openapi3::Spec,
    operation_ids: BTreeMap<String, usize>,
}

#[derive(Default)]
struct State {
    auth_stack: Vec<postman::Auth>,
    hierarchy: Vec<String>,
    variables: Variables,
}

impl<'a> OpenApi30Backend<'a> {
    fn generate(name: String, description: Option<String>) -> openapi3::Spec {
        openapi3::Spec {
            openapi: String::from("3.0.3"),
            info: openapi3::Info {
                license: None,
                contact: Some(openapi3::Contact::default()),
                description,
                terms_of_service: None,
                version: String::from("1.0.0"),
                title: name,
            },
            components: None,
            external_docs: None,
            paths: IndexMap::new(),
            security: None,
            servers: Some(Vec::<openapi3::Server>::new()),
            tags: Some(IndexSet::<openapi3::Tag>::new()),
        }
    }

    fn create_path_parameters(
        state: &mut State,
        resolved_segments: &[String],
        postman_variables: &Option<Vec<postman::Variable>>,
    ) -> Option<Vec<openapi3::ObjectOrReference<openapi3::Parameter>>> {
        let params: Vec<openapi3::ObjectOrReference<openapi3::Parameter>> = resolved_segments
            .iter()
            .flat_map(|segment| {
                URI_TEMPLATE_VARIABLE_RE
                    .captures_iter(segment.as_str())
                    .map(|capture| {
                        let var = capture.get(1).unwrap().as_str();
                        let mut param = Parameter {
                            name: var.to_owned(),
                            location: "path".to_owned(),
                            required: Some(true),
                            ..Parameter::default()
                        };

                        let mut schema = openapi3::Schema {
                            schema_type: Some("string".to_string()),
                            ..Default::default()
                        };
                        if let Some(path_val) = &postman_variables {
                            if let Some(p) = path_val.iter().find(|p| match &p.key {
                                Some(k) => k == var,
                                _ => false,
                            }) {
                                param.description = p.description.as_ref().map(|d| d.into());
                                if let Some(pval) = &p.value {
                                    if let Some(pval_val) = pval.as_str() {
                                        schema.example = Some(serde_json::Value::String(
                                            state.variables.resolve(pval_val),
                                        ));
                                    }
                                }
                            }
                        }
                        param.schema = Some(schema);
                        openapi3::ObjectOrReference::Object(param)
                    })
            })
            .collect();

        if !params.is_empty() {
            Some(params)
        } else {
            None
        }
    }

    fn create_query_parameters(
        state: &mut State,
        query_params: &[postman::QueryParam],
    ) -> Option<Vec<openapi3::ObjectOrReference<openapi3::Parameter>>> {
        let mut keys = vec![];
        let params = query_params
            .iter()
            .filter_map(|qp| match qp.key {
                Some(ref key) => {
                    if keys.contains(&key.as_str()) {
                        return None;
                    }

                    keys.push(key);
                    let param = Parameter {
                        name: key.to_owned(),
                        description: qp.description.as_ref().map(|d| d.into()),
                        location: "query".to_owned(),
                        schema: Some(openapi3::Schema {
                            schema_type: Some("string".to_string()),
                            example: qp.value.as_ref().map(|pval| {
                                serde_json::Value::String(state.variables.resolve(pval))
                            }),
                            ..openapi3::Schema::default()
                        }),
                        ..Parameter::default()
                    };

                    Some(openapi3::ObjectOrReference::Object(param))
                }
                None => None,
            })
            .collect::<Vec<openapi3::ObjectOrReference<openapi3::Parameter>>>();

        if !params.is_empty() {
            Some(params)
        } else {
            None
        }
    }

    fn create_request_body(
        state: &mut State,
        body: &postman::Body,
        op: &mut openapi3::Operation,
        name: &str,
        ct: Option<String>,
    ) {
        let mut content_type = ct;
        let mut request_body = if let Some(ObjectOrReference::Object(rb)) = op.request_body.as_mut()
        {
            rb.clone()
        } else {
            openapi3::RequestBody::default()
        };

        let default_media_type = openapi3::MediaType::default();

        if let Some(mode) = &body.mode {
            match mode {
                postman::Mode::Raw => {
                    content_type = Some("application/octet-stream".to_string());
                    if let Some(raw) = &body.raw {
                        let resolved_body = state.variables.resolve(raw);
                        let example_val;

                        //set content type based on options or inference.
                        match serde_json::from_str(&resolved_body) {
                            Ok(v) => match v {
                                serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                                    content_type = Some("application/json".to_string());
                                    let content = {
                                        let ct = content_type.as_ref().unwrap();
                                        if !request_body.content.contains_key(ct) {
                                            request_body
                                                .content
                                                .insert(ct.clone(), default_media_type.clone());
                                        }

                                        request_body.content.get_mut(ct).unwrap()
                                    };

                                    if let Some(schema) = Self::create_schema(&v) {
                                        content.schema =
                                            Some(openapi3::ObjectOrReference::Object(schema));
                                    }
                                    example_val = v;
                                }
                                _ => {
                                    example_val = serde_json::Value::String(resolved_body);
                                }
                            },
                            _ => {
                                content_type = Some("text/plain".to_string());
                                if let Some(options) = body.options.clone() {
                                    if let Some(raw_options) = options.raw {
                                        if raw_options.language.is_some() {
                                            content_type =
                                                match raw_options.language.unwrap().as_str() {
                                                    "xml" => Some("application/xml".to_string()),
                                                    "json" => Some("application/json".to_string()),
                                                    "html" => Some("text/html".to_string()),
                                                    _ => Some("text/plain".to_string()),
                                                }
                                        }
                                    }
                                }
                                example_val = serde_json::Value::String(resolved_body);
                            }
                        }

                        let content = {
                            let ct = content_type.as_ref().unwrap();
                            if !request_body.content.contains_key(ct) {
                                request_body
                                    .content
                                    .insert(ct.clone(), default_media_type.clone());
                            }

                            request_body.content.get_mut(ct).unwrap()
                        };

                        let examples = content.examples.clone().unwrap_or(
                            openapi3::MediaTypeExample::Examples {
                                examples: BTreeMap::new(),
                            },
                        );

                        let example = openapi3::Example {
                            summary: None,
                            description: None,
                            value: Some(example_val),
                        };

                        if let openapi3::MediaTypeExample::Examples { examples: mut ex } = examples
                        {
                            ex.insert(name.to_string(), ObjectOrReference::Object(example));
                            content.examples =
                                Some(openapi3::MediaTypeExample::Examples { examples: ex });
                        }
                        *content = content.clone();
                    }
                }
                postman::Mode::Urlencoded => {
                    content_type = Some("application/x-www-form-urlencoded".to_string());
                    let content = {
                        let ct = content_type.as_ref().unwrap();
                        if !request_body.content.contains_key(ct) {
                            request_body
                                .content
                                .insert(ct.clone(), default_media_type.clone());
                        }

                        request_body.content.get_mut(ct).unwrap()
                    };
                    if let Some(urlencoded) = &body.urlencoded {
                        let mut oas_data = serde_json::Map::new();
                        for i in urlencoded {
                            if let Some(v) = &i.value {
                                let value = serde_json::Value::String(v.to_string());
                                oas_data.insert(i.key.clone(), value);
                            }
                        }
                        let oas_obj = serde_json::Value::Object(oas_data);
                        if let Some(schema) = Self::create_schema(&oas_obj) {
                            content.schema = Some(openapi3::ObjectOrReference::Object(schema));
                        }

                        let examples = content.examples.clone().unwrap_or(
                            openapi3::MediaTypeExample::Examples {
                                examples: BTreeMap::new(),
                            },
                        );

                        let example = openapi3::Example {
                            summary: None,
                            description: None,
                            value: Some(oas_obj),
                        };

                        if let openapi3::MediaTypeExample::Examples { examples: mut ex } = examples
                        {
                            ex.insert(name.to_string(), ObjectOrReference::Object(example));
                            content.examples =
                                Some(openapi3::MediaTypeExample::Examples { examples: ex });
                        }
                    }
                }
                postman::Mode::Formdata => {
                    content_type = Some("multipart/form-data".to_string());
                    let content = {
                        let ct = content_type.as_ref().unwrap();
                        if !request_body.content.contains_key(ct) {
                            request_body
                                .content
                                .insert(ct.clone(), default_media_type.clone());
                        }

                        request_body.content.get_mut(ct).unwrap()
                    };

                    let mut schema = openapi3::Schema {
                        schema_type: Some("object".to_string()),
                        ..Default::default()
                    };
                    let mut properties = BTreeMap::<String, openapi3::Schema>::new();

                    if let Some(formdata) = &body.formdata {
                        for i in formdata {
                            if let Some(t) = &i.form_parameter_type {
                                let is_binary = t.as_str() == "file";
                                if let Some(v) = &i.value {
                                    let value = serde_json::Value::String(v.to_string());
                                    let prop_schema = Self::create_schema(&value);
                                    if let Some(mut prop_schema) = prop_schema {
                                        if is_binary {
                                            prop_schema.format = Some("binary".to_string());
                                        }
                                        prop_schema.description =
                                            i.description.as_ref().map(|d| d.into());
                                        properties.insert(i.key.clone(), prop_schema);
                                    }
                                } else {
                                    let mut prop_schema = openapi3::Schema {
                                        schema_type: Some("string".to_string()),
                                        description: i.description.as_ref().map(|d| d.into()),
                                        ..Default::default()
                                    };
                                    if is_binary {
                                        prop_schema.format = Some("binary".to_string());
                                    }
                                    properties.insert(i.key.clone(), prop_schema);
                                }
                            }
                            // NOTE: Postman doesn't store the content type of multipart files. :(
                        }
                        schema.properties = Some(properties);
                        content.schema = Some(openapi3::ObjectOrReference::Object(schema));
                    }
                }

                postman::Mode::GraphQl => {
                    content_type = Some("application/json".to_string());
                    let content = {
                        let ct = content_type.as_ref().unwrap();
                        if !request_body.content.contains_key(ct) {
                            request_body
                                .content
                                .insert(ct.clone(), default_media_type.clone());
                        }

                        request_body.content.get_mut(ct).unwrap()
                    };

                    // The schema is the same for every GraphQL request.
                    content.schema = Some(ObjectOrReference::Object(openapi3::Schema {
                        schema_type: Some("object".to_owned()),
                        properties: Some(BTreeMap::from([
                            (
                                "query".to_owned(),
                                openapi3::Schema {
                                    schema_type: Some("string".to_owned()),
                                    ..openapi3::Schema::default()
                                },
                            ),
                            (
                                "variables".to_owned(),
                                openapi3::Schema {
                                    schema_type: Some("object".to_owned()),
                                    ..openapi3::Schema::default()
                                },
                            ),
                        ])),
                        ..openapi3::Schema::default()
                    }));

                    if let Some(postman::GraphQlBody::GraphQlBodyClass(graphql)) = &body.graphql {
                        if let Some(query) = &graphql.query {
                            let mut example_map = serde_json::Map::new();
                            example_map.insert("query".to_owned(), query.to_owned().into());
                            if let Some(vars) = &graphql.variables {
                                if let Ok(vars) = serde_json::from_str::<serde_json::Value>(vars) {
                                    example_map.insert("variables".to_owned(), vars);
                                }
                            }

                            let example = openapi3::MediaTypeExample::Example {
                                example: serde_json::Value::Object(example_map),
                            };
                            content.examples = Some(example);
                        }
                    }
                }
                _ => content_type = Some("application/octet-stream".to_string()),
            }
        }

        if content_type.is_none() {
            content_type = Some("application/octet-stream".to_string());
            request_body
                .content
                .insert(content_type.unwrap(), default_media_type);
        }

        op.request_body = Some(openapi3::ObjectOrReference::Object(request_body));
    }

    fn create_schema(value: &serde_json::Value) -> Option<openapi3::Schema> {
        match value {
            serde_json::Value::Object(m) => {
                let mut schema = openapi3::Schema {
                    schema_type: Some("object".to_string()),
                    ..Default::default()
                };

                let mut properties = BTreeMap::<String, openapi3::Schema>::new();

                for (key, val) in m.iter() {
                    if let Some(v) = Self::create_schema(val) {
                        properties.insert(key.to_string(), v);
                    }
                }

                schema.properties = Some(properties);
                Some(schema)
            }
            serde_json::Value::Array(a) => {
                let mut schema = openapi3::Schema {
                    schema_type: Some("array".to_string()),
                    ..Default::default()
                };

                let mut item_schema = openapi3::Schema::default();

                for n in 0..a.len() {
                    if let Some(i) = a.get(n) {
                        if let Some(i) = Self::create_schema(i) {
                            if n == 0 {
                                item_schema = i;
                            } else {
                                item_schema = Self::merge_schemas(item_schema, &i);
                            }
                        }
                    }
                }

                schema.items = Some(Box::new(item_schema));
                schema.example = Some(value.clone());

                Some(schema)
            }
            serde_json::Value::String(_) => {
                let schema = openapi3::Schema {
                    schema_type: Some("string".to_string()),
                    example: Some(value.clone()),
                    ..Default::default()
                };
                Some(schema)
            }
            serde_json::Value::Number(_) => {
                let schema = openapi3::Schema {
                    schema_type: Some("number".to_string()),
                    example: Some(value.clone()),
                    ..Default::default()
                };
                Some(schema)
            }
            serde_json::Value::Bool(_) => {
                let schema = openapi3::Schema {
                    schema_type: Some("boolean".to_string()),
                    example: Some(value.clone()),
                    ..Default::default()
                };
                Some(schema)
            }
            serde_json::Value::Null => {
                let schema = openapi3::Schema {
                    nullable: Some(true),
                    example: Some(value.clone()),
                    ..Default::default()
                };
                Some(schema)
            }
        }
    }

    fn merge_schemas(mut original: openapi3::Schema, new: &openapi3::Schema) -> openapi3::Schema {
        // If the new schema has a nullable Option but the original doesn't,
        // set the original nullable to the new one.
        if original.nullable.is_none() && new.nullable.is_some() {
            original.nullable = new.nullable;
        }

        // If both original and new have a nullable Option,
        // If any of their values is true, set to true.
        if let Some(original_nullable) = original.nullable {
            if let Some(new_nullable) = new.nullable {
                if new_nullable != original_nullable {
                    original.nullable = Some(true);
                }
            }
        }

        if let Some(ref mut any_of) = original.any_of {
            any_of.push(openapi3::ObjectOrReference::Object(new.clone()));
            return original;
        }

        // Reset the schema type.
        if original.schema_type.is_none() && new.schema_type.is_some() && new.any_of.is_none() {
            original.schema_type = new.schema_type.clone();
        }

        // If both types are objects, merge the schemas of each property.
        if let Some(t) = &original.schema_type {
            if let "object" = t.as_str() {
                if let Some(original_properties) = &mut original.properties {
                    if let Some(new_properties) = &new.properties {
                        for (key, val) in original_properties.iter_mut() {
                            if let Some(v) = new_properties.get(key) {
                                let prop = v;
                                *val = Self::merge_schemas(val.clone(), prop);
                            }
                        }

                        for (key, val) in new_properties.iter() {
                            if !original_properties.contains_key(key) {
                                original_properties.insert(key.to_string(), val.clone());
                            }
                        }
                    }
                }
            }
        }

        if let Some(ref original_type) = original.schema_type {
            if let Some(ref new_type) = new.schema_type {
                if new_type != original_type {
                    let cloned = original.clone();
                    original.schema_type = None;
                    original.properties = None;
                    original.items = None;
                    original.any_of = Some(vec![
                        openapi3::ObjectOrReference::Object(cloned),
                        openapi3::ObjectOrReference::Object(new.clone()),
                    ]);
                }
            }
        }

        original
    }
}

#[derive(Debug, Clone)]
struct CreateOperationParams<'a> {
    auth: Option<postman::Auth>,
    item: &'a postman::Items,
    request: &'a postman::RequestClass,
    request_name: &'a str,
    path_elements: &'a [postman::PathElement],
    url: &'a postman::UrlClass,
}

#[derive(Debug, Default, Clone)]
struct Variables {
    map: BTreeMap<String, serde_json::value::Value>,
    replace_credits: usize,
}

impl Variables {
    fn resolve(&self, segment: &str) -> String {
        self.resolve_with_credits(segment, self.replace_credits)
    }

    fn resolve_with_credits(&self, segment: &str, sub_replace_credits: usize) -> String {
        self.resolve_with_credits_and_replace_fn(segment, sub_replace_credits, |s| s)
    }

    fn resolve_with_credits_and_replace_fn(
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
                    let capture = &cap[n].to_string();
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

trait Frontend {
    fn convert<T: Backend>(&mut self, backend: &mut T, state: &mut State, items: &[postman::Items]);
    fn convert_folder<T: Backend>(
        &mut self,
        backend: &mut T,
        state: &mut State,
        items: &[postman::Items],
        name: &str,
        description: Option<String>,
        auth: &Option<postman::Auth>,
    );
    fn convert_request<T: Backend>(
        &mut self,
        backend: &mut T,
        state: &mut State,
        item: &postman::Items,
        name: &str,
    );
}

trait Backend {
    fn create_server(&mut self, state: &mut State, url: &postman::UrlClass, parts: &[String]);
    fn create_tag(&mut self, state: &mut State, name: &str, description: Option<String>);
    fn create_operation(&mut self, state: &mut State, params: CreateOperationParams);
    fn create_security(
        &mut self,
        state: &mut State,
        auth: postman::Auth,
        add_to_root: bool,
    ) -> Option<Option<(String, Vec<String>)>>;
}

impl Transpiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn transpile(spec: postman::Spec) -> openapi::OpenApi {
        let description: Option<String> = spec.info.description.as_ref().map(|d| d.into());

        let mut oas = OpenApi30Backend::generate(spec.info.name, description);
        let mut variable_map = BTreeMap::<String, serde_json::value::Value>::new();
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

        let hierarchy = Vec::<String>::new();
        let auth_stack = Vec::<postman::Auth>::new();

        let state = &mut State {
            auth_stack,
            hierarchy,
            variables,
        };
        let mut backend = OpenApi30Backend {
            oas: &mut oas,
            operation_ids: BTreeMap::<String, usize>::new(),
        };
        let mut transpiler = Transpiler {};

        if let Some(auth) = spec.auth {
            state.auth_stack.push(auth.clone());
            backend.create_security(state, auth, true);
        }

        transpiler.convert(&mut backend, state, &spec.item);
        openapi::OpenApi::V3_0(Box::new(oas))
    }
}

impl Frontend for Transpiler {
    fn convert<T: Backend>(
        &mut self,
        backend: &mut T,
        state: &mut State,
        items: &[postman::Items],
    ) {
        for item in items {
            if let Some(i) = &item.item {
                let name = match &item.name {
                    Some(n) => n,
                    None => "<folder>",
                };
                let description: Option<String> = item.description.as_ref().map(|d| d.into());

                self.convert_folder(backend, state, i, name, description, &item.auth);
            } else {
                let name = match &item.name {
                    Some(n) => n,
                    None => "<request>",
                };
                self.convert_request(backend, state, item, name);
            }
        }
    }

    fn convert_folder<T: Backend>(
        &mut self,
        backend: &mut T,
        state: &mut State,
        items: &[postman::Items],
        name: &str,
        description: Option<String>,
        auth: &Option<postman::Auth>,
    ) {
        let mut pushed_auth = false;

        backend.create_tag(state, name.clone(), description);
        state.hierarchy.push(name.to_string());

        if let Some(auth) = auth {
            state.auth_stack.push(auth.clone());
            pushed_auth = true;
        }

        self.convert(backend, state, items);

        state.hierarchy.pop();

        if pushed_auth {
            state.auth_stack.pop();
        }
    }

    fn convert_request<T: Backend>(
        &mut self,
        backend: &mut T,
        state: &mut State,
        item: &postman::Items,
        name: &str,
    ) {
        if let Some(postman::RequestUnion::RequestClass(request)) = &item.request {
            if let Some(postman::Url::UrlClass(u)) = &request.url {
                if let Some(postman::Host::StringArray(parts)) = &u.host {
                    backend.create_server(state, u, parts);
                }

                let root_path: Vec<postman::PathElement> = vec![];
                let path_elements = match &u.path {
                    Some(postman::UrlPath::UnionArray(p)) => p,
                    _ => &root_path,
                };

                let auth = if let Some(auth) = &request.auth {
                    Some(auth.clone())
                } else if !state.auth_stack.is_empty() {
                    Some(state.auth_stack.last().unwrap().clone())
                } else {
                    None
                };

                let params = CreateOperationParams {
                    item,
                    request,
                    request_name: name,
                    url: u,
                    path_elements,
                    auth,
                };
                backend.create_operation(state, params)
            }
        }
    }
}

impl<'a> Backend for OpenApi30Backend<'a> {
    fn create_server(&mut self, state: &mut State, url: &postman::UrlClass, parts: &[String]) {
        let host = parts.join(".");
        let mut proto = "".to_string();
        if let Some(protocol) = &url.protocol {
            proto = format!("{protocol}://", protocol = protocol.clone());
        }
        if let Some(s) = &mut self.oas.servers {
            let mut server_url = format!("{proto}{host}");
            server_url = state.variables.resolve(&server_url);
            if !s.iter_mut().any(|srv| srv.url == server_url) {
                let server = openapi3::Server {
                    url: server_url,
                    description: None,
                    variables: None,
                };
                s.push(server);
            }
        }
    }

    fn create_tag(&mut self, _state: &mut State, name: &str, description: Option<String>) {
        if let Some(t) = &mut self.oas.tags {
            let mut tag = openapi3::Tag {
                name: name.to_string(),
                description,
            };

            let mut i: usize = 0;
            while t.contains(&tag) {
                i += 1;
                tag.name = format!("{tagName}{i}", tagName = tag.name);
            }

            t.insert(tag);
        };
    }

    fn create_operation(&mut self, state: &mut State, params: CreateOperationParams) {
        let CreateOperationParams {
            auth,
            item,
            request,
            request_name,
            path_elements: paths,
            url,
        } = params;

        let sr = if let Some(auth) = auth {
            let security = self.create_security(state, auth.clone(), false);
            match security {
                Some(Some((name, scopes))) => Some(SecurityRequirement {
                    requirement: Some(BTreeMap::from([(name, scopes)])),
                }),
                Some(None) => Some(SecurityRequirement { requirement: None }),
                _ => None,
            }
        } else {
            None
        };

        let resolved_segments = paths
            .iter()
            .map(|segment| {
                let mut seg = match segment {
                    postman::PathElement::PathClass(c) => c.clone().value.unwrap_or_default(),
                    postman::PathElement::String(c) => c.to_string(),
                };
                seg = state.variables.resolve_with_credits_and_replace_fn(
                    &seg,
                    state.variables.replace_credits,
                    |s| VARIABLE_RE.replace_all(&s, "{$1}").to_string(),
                );
                if !seg.is_empty() {
                    match &seg[0..1] {
                        ":" => format!("{{{}}}", &seg[1..]),
                        _ => seg.to_string(),
                    }
                } else {
                    seg
                }
            })
            .collect::<Vec<String>>();
        let segments = "/".to_string() + &resolved_segments.join("/");

        // TODO: Because of variables, we can actually get duplicate paths.
        // - /admin/{subresource}/{subresourceId}
        // - /admin/{subresource2}/{subresource2Id}
        // Throw a warning?
        if !self.oas.paths.contains_key(&segments) {
            self.oas
                .paths
                .insert(segments.clone(), openapi3::PathItem::default());
        }

        let path = self.oas.paths.get_mut(&segments).unwrap();
        let method = match &request.method {
            Some(m) => m.to_lowercase(),
            None => "get".to_string(),
        };
        let op_ref = match method.as_str() {
            "get" => &mut path.get,
            "post" => &mut path.post,
            "put" => &mut path.put,
            "delete" => &mut path.delete,
            "patch" => &mut path.patch,
            "options" => &mut path.options,
            "trace" => &mut path.trace,
            _ => &mut path.get,
        };
        let is_merge = op_ref.is_some();
        if op_ref.is_none() {
            *op_ref = Some(openapi3::Operation::default());
        }
        let op = op_ref.as_mut().unwrap();

        path.parameters = Self::create_path_parameters(state, &resolved_segments, &url.variable);
        if let Some(sr) = sr {
            if let Some(op_security) = &mut op.security {
                if !op_security.contains(&sr) {
                    op_security.push(sr);
                }
            } else {
                op.security = Some(vec![sr]);
            }
        }

        if !is_merge {
            let mut op_id = request_name
                .chars()
                .map(|c| match c {
                    'A'..='Z' | 'a'..='z' | '0'..='9' => c,
                    _ => ' ',
                })
                .collect::<String>()
                .from_case(Case::Title)
                .to_case(Case::Camel);

            match self.operation_ids.get_mut(&op_id) {
                Some(v) => {
                    *v += 1;
                    op_id = format!("{op_id}{v}");
                }
                None => {
                    self.operation_ids.insert(op_id.clone(), 0);
                }
            }

            op.operation_id = Some(op_id);
        }

        if let Some(qp) = &url.query {
            if let Some(mut query_params) = Self::create_query_parameters(state, qp) {
                match &op.parameters {
                    Some(params) => {
                        let mut cloned = params.clone();
                        for p1 in &mut query_params {
                            if let ObjectOrReference::Object(p1) = p1 {
                                let found = cloned.iter_mut().find(|p2| {
                                    if let ObjectOrReference::Object(p2) = p2 {
                                        p2.location == p1.location && p2.name == p1.name
                                    } else {
                                        false
                                    }
                                });
                                if let Some(ObjectOrReference::Object(p2)) = found {
                                    p2.schema = Some(Self::merge_schemas(
                                        p2.schema.clone().unwrap(),
                                        &p1.schema.clone().unwrap(),
                                    ));
                                } else {
                                    cloned.push(ObjectOrReference::Object(p1.clone()));
                                }
                            }
                        }
                        op.parameters = Some(cloned);
                    }
                    None => op.parameters = Some(query_params),
                };
            }
        }

        let mut content_type: Option<String> = None;

        if let Some(postman::HeaderUnion::HeaderArray(headers)) = &request.header {
            for header in headers.iter() {
                let key = header.key.to_lowercase();
                if key == "accept" || key == "authorization" {
                    continue;
                }
                if key == "content-type" {
                    let content_type_parts: Vec<&str> = header.value.split(';').collect();
                    content_type = Some(content_type_parts[0].to_owned());
                } else {
                    let param = Parameter {
                        location: "header".to_owned(),
                        name: header.key.to_owned(),
                        description: header.description.as_ref().map(|d| d.into()),
                        schema: Some(openapi3::Schema {
                            schema_type: Some("string".to_owned()),
                            example: Some(serde_json::Value::String(header.value.to_owned())),
                            ..openapi3::Schema::default()
                        }),
                        ..Parameter::default()
                    };

                    if op.parameters.is_none() {
                        op.parameters = Some(vec![ObjectOrReference::Object(param)]);
                    } else {
                        let params = op.parameters.as_mut().unwrap();
                        let mut has_pushed = false;
                        for p in params {
                            if let ObjectOrReference::Object(p) = p {
                                if p.name == param.name && p.location == param.location {
                                    if let Some(schema) = &p.schema {
                                        has_pushed = true;
                                        p.schema = Some(Self::merge_schemas(
                                            schema.clone(),
                                            &param.schema.clone().unwrap(),
                                        ));
                                    }
                                }
                            }
                        }
                        if !has_pushed {
                            op.parameters
                                .as_mut()
                                .unwrap()
                                .push(ObjectOrReference::Object(param));
                        }
                    }
                }
            }
        }

        if let Some(body) = &request.body {
            Self::create_request_body(state, body, op, request_name, content_type);
        }

        if !is_merge {
            let description = match request.description.as_ref().map(|d| d.into()) {
                Some(desc) => Some(desc),
                None => Some(request_name.to_string()),
            };

            op.summary = Some(request_name.to_string());
            op.description = description;
        }

        if !state.hierarchy.is_empty() {
            op.tags = Some(state.hierarchy.clone());
        }

        if let Some(responses) = &item.response {
            for r in responses.iter().flatten() {
                if let Some(or) = &r.original_request {
                    if let Some(body) = &or.body {
                        content_type = Some("text/plain".to_string());
                        if let Some(options) = body.options.clone() {
                            if let Some(raw_options) = options.raw {
                                if raw_options.language.is_some() {
                                    content_type = match raw_options.language.unwrap().as_str() {
                                        "xml" => Some("application/xml".to_string()),
                                        "json" => Some("application/json".to_string()),
                                        "html" => Some("text/html".to_string()),
                                        _ => Some("text/plain".to_string()),
                                    }
                                }
                            }
                        }
                        Self::create_request_body(state, body, op, request_name, content_type);
                    }
                }
                let mut oas_response = openapi3::Response::default();
                let mut response_media_types = BTreeMap::<String, openapi3::MediaType>::new();

                if let Some(name) = &r.name {
                    oas_response.description = Some(name.clone());
                }
                if let Some(postman::Headers::UnionArray(headers)) = &r.header {
                    let mut oas_headers =
                        BTreeMap::<String, openapi3::ObjectOrReference<openapi3::Header>>::new();
                    for h in headers {
                        if let postman::HeaderElement::Header(hdr) = h {
                            if hdr.value.is_empty() || hdr.key.to_lowercase() == "content-type" {
                                continue;
                            }
                            let mut oas_header = openapi3::Header::default();
                            let header_schema = openapi3::Schema {
                                schema_type: Some("string".to_string()),
                                example: Some(serde_json::Value::String(hdr.value.to_string())),
                                ..Default::default()
                            };
                            oas_header.schema = Some(header_schema);

                            oas_headers.insert(
                                hdr.key.clone(),
                                openapi3::ObjectOrReference::Object(oas_header),
                            );
                        }
                    }
                    if !oas_headers.is_empty() {
                        oas_response.headers = Some(oas_headers);
                    }
                }
                let mut response_content = openapi3::MediaType::default();
                if let Some(raw) = &r.body {
                    let mut response_content_type: Option<String> = None;
                    let resolved_body = state.variables.resolve(raw);
                    let example_val;

                    match serde_json::from_str(&resolved_body) {
                        Ok(v) => match v {
                            serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                                response_content_type = Some("application/json".to_string());
                                if let Some(schema) = Self::create_schema(&v) {
                                    response_content.schema =
                                        Some(openapi3::ObjectOrReference::Object(schema));
                                }
                                example_val = v;
                            }
                            _ => {
                                example_val = serde_json::Value::String(resolved_body);
                            }
                        },
                        _ => {
                            // TODO: Check if XML, HTML, JavaScript
                            response_content_type = Some("text/plain".to_string());
                            example_val = serde_json::Value::String(resolved_body);
                        }
                    }
                    let mut example_map =
                        BTreeMap::<String, openapi3::ObjectOrReference<openapi3::Example>>::new();

                    let ex = openapi3::Example {
                        summary: None,
                        description: None,
                        value: Some(example_val),
                    };

                    let example_name = match &r.name {
                        Some(n) => n.to_string(),
                        None => "".to_string(),
                    };

                    example_map.insert(example_name, openapi3::ObjectOrReference::Object(ex));
                    let example = openapi3::MediaTypeExample::Examples {
                        examples: example_map,
                    };

                    response_content.examples = Some(example);

                    if response_content_type.is_none() {
                        response_content_type = Some("application/octet-stream".to_string());
                    }

                    response_media_types
                        .insert(response_content_type.clone().unwrap(), response_content);
                }
                oas_response.content = Some(response_media_types);

                if let Some(code) = &r.code {
                    if let Some(existing_response) = op.responses.get_mut(&code.to_string()) {
                        let new_response = oas_response.clone();
                        if let Some(name) = &new_response.description {
                            existing_response.description = Some(
                                existing_response
                                    .description
                                    .clone()
                                    .unwrap_or("".to_string())
                                    + " / "
                                    + name,
                            );
                        }

                        if let Some(headers) = new_response.headers {
                            let mut cloned_headers = headers.clone();
                            for (key, val) in headers {
                                cloned_headers.insert(key, val);
                            }
                            existing_response.headers = Some(cloned_headers);
                        }

                        let mut existing_content =
                            existing_response.content.clone().unwrap_or_default();
                        for (media_type, new_content) in new_response.content.unwrap() {
                            if let Some(existing_response_content) =
                                existing_content.get_mut(&media_type)
                            {
                                if let Some(openapi3::ObjectOrReference::Object(existing_schema)) =
                                    existing_response_content.schema.clone()
                                {
                                    if let Some(openapi3::ObjectOrReference::Object(new_schema)) =
                                        new_content.schema
                                    {
                                        existing_response_content.schema =
                                            Some(openapi3::ObjectOrReference::Object(
                                                Self::merge_schemas(existing_schema, &new_schema),
                                            ))
                                    }
                                }

                                if let Some(openapi3::MediaTypeExample::Examples {
                                    examples: existing_examples,
                                }) = &mut existing_response_content.examples
                                {
                                    let new_example_map = match new_content.examples.unwrap() {
                                        openapi3::MediaTypeExample::Examples { examples } => {
                                            examples.clone()
                                        }
                                        _ => BTreeMap::<String, _>::new(),
                                    };
                                    for (key, value) in new_example_map.iter() {
                                        existing_examples.insert(key.clone(), value.clone());
                                    }
                                }
                            }
                        }
                        existing_response.content = Some(existing_content.clone());
                    } else {
                        op.responses.insert(code.to_string(), oas_response);
                    }
                }
            }
        }

        if !op.responses.contains_key("200")
            && !op.responses.contains_key("201")
            && !op.responses.contains_key("202")
            && !op.responses.contains_key("203")
            && !op.responses.contains_key("204")
            && !op.responses.contains_key("205")
            && !op.responses.contains_key("206")
            && !op.responses.contains_key("207")
            && !op.responses.contains_key("208")
            && !op.responses.contains_key("226")
        {
            op.responses.insert(
                "200".to_string(),
                openapi3::Response {
                    description: Some("".to_string()),
                    ..openapi3::Response::default()
                },
            );
        }
    }

    fn create_security(
        &mut self,
        state: &mut State,
        auth: postman::Auth,
        add_to_root: bool,
    ) -> Option<Option<(String, Vec<String>)>> {
        if self.oas.components.is_none() {
            self.oas.components = Some(openapi3::Components::default());
        }
        if self
            .oas
            .components
            .as_ref()
            .unwrap()
            .security_schemes
            .is_none()
        {
            self.oas.components.as_mut().unwrap().security_schemes = Some(BTreeMap::new());
        }
        let security_schemes = self
            .oas
            .components
            .as_mut()
            .unwrap()
            .security_schemes
            .as_mut()
            .unwrap();
        let security = match auth.auth_type {
            AuthType::Noauth => Some(None),
            AuthType::Basic => {
                let scheme = openapi3::SecurityScheme::Http {
                    scheme: "basic".to_string(),
                    bearer_format: None,
                };
                let name = "basicAuth".to_string();
                security_schemes.insert(name.clone(), ObjectOrReference::Object(scheme));
                Some(Some((name, vec![])))
            }
            AuthType::Digest => {
                let scheme = openapi3::SecurityScheme::Http {
                    scheme: "digest".to_string(),
                    bearer_format: None,
                };
                let name = "digestAuth".to_string();
                security_schemes.insert(name.clone(), ObjectOrReference::Object(scheme));
                Some(Some((name, vec![])))
            }
            AuthType::Bearer => {
                let scheme = openapi3::SecurityScheme::Http {
                    scheme: "bearer".to_string(),
                    bearer_format: None,
                };
                let name = "bearerAuth".to_string();
                security_schemes.insert(name.clone(), ObjectOrReference::Object(scheme));
                Some(Some((name, vec![])))
            }
            AuthType::Jwt => {
                let scheme = openapi3::SecurityScheme::Http {
                    scheme: "bearer".to_string(),
                    bearer_format: Some("jwt".to_string()),
                };
                let name = "jwtBearerAuth".to_string();
                security_schemes.insert(name.clone(), ObjectOrReference::Object(scheme));
                Some(Some((name, vec![])))
            }
            AuthType::Apikey => {
                let name = "apiKey".to_string();
                if let Some(apikey) = &auth.apikey {
                    let scheme = openapi3::SecurityScheme::ApiKey {
                        name: state
                            .variables
                            .resolve(apikey.key.as_ref().unwrap_or(&"Authorization".to_string())),
                        location: match apikey.location {
                            postman::ApiKeyLocation::Header => "header".to_string(),
                            postman::ApiKeyLocation::Query => "query".to_string(),
                        },
                    };
                    security_schemes.insert(name.clone(), ObjectOrReference::Object(scheme));
                } else {
                    let scheme = openapi3::SecurityScheme::ApiKey {
                        name: "Authorization".to_string(),
                        location: "header".to_string(),
                    };
                    security_schemes.insert(name.clone(), ObjectOrReference::Object(scheme));
                }
                Some(Some((name, vec![])))
            }
            AuthType::Oauth2 => {
                let name = "oauth2".to_string();
                if let Some(oauth2) = &auth.oauth2 {
                    let mut flows: openapi3::Flows = Default::default();
                    let scopes = BTreeMap::from_iter(
                        oauth2
                            .scope
                            .clone()
                            .unwrap_or_default()
                            .iter()
                            .map(|s| state.variables.resolve(s))
                            .map(|s| (s.to_string(), s.to_string())),
                    );
                    let authorization_url = state
                        .variables
                        .resolve(oauth2.auth_url.as_ref().unwrap_or(&"".to_string()));
                    let token_url = state
                        .variables
                        .resolve(oauth2.access_token_url.as_ref().unwrap_or(&"".to_string()));
                    let refresh_url = oauth2
                        .refresh_token_url
                        .as_ref()
                        .map(|url| state.variables.resolve(url));
                    match oauth2.grant_type {
                        postman::Oauth2GrantType::AuthorizationCode
                        | postman::Oauth2GrantType::AuthorizationCodeWithPkce => {
                            flows.authorization_code = Some(openapi3::AuthorizationCodeFlow {
                                authorization_url,
                                token_url,
                                refresh_url,
                                scopes,
                            });
                        }
                        postman::Oauth2GrantType::ClientCredentials => {
                            flows.client_credentials = Some(openapi3::ClientCredentialsFlow {
                                token_url,
                                refresh_url,
                                scopes,
                            });
                        }
                        postman::Oauth2GrantType::PasswordCredentials => {
                            flows.password = Some(openapi3::PasswordFlow {
                                token_url,
                                refresh_url,
                                scopes,
                            });
                        }
                        postman::Oauth2GrantType::Implicit => {
                            flows.implicit = Some(openapi3::ImplicitFlow {
                                authorization_url,
                                refresh_url,
                                scopes,
                            });
                        }
                    }
                    let scheme = openapi3::SecurityScheme::OAuth2 {
                        flows: Box::new(flows),
                    };
                    security_schemes.insert(name.clone(), ObjectOrReference::Object(scheme));
                    Some(Some((name, oauth2.scope.clone().unwrap_or_default())))
                } else {
                    let scheme = openapi3::SecurityScheme::OAuth2 {
                        flows: Default::default(),
                    };
                    security_schemes.insert(name.clone(), ObjectOrReference::Object(scheme));
                    Some(Some((name, vec![])))
                }
            }
            _ => None,
        };

        let security_requirement = match security.clone() {
            Some(Some((name, scopes))) => Some(SecurityRequirement {
                requirement: Some(BTreeMap::from([(name, scopes)])),
            }),
            Some(None) => Some(SecurityRequirement { requirement: None }),
            _ => None,
        };

        if add_to_root {
            if let Some(security_requirement) = security_requirement {
                if self.oas.security.is_none() {
                    self.oas.security = Some(vec![security_requirement]);
                } else {
                    self.oas
                        .security
                        .as_mut()
                        .unwrap()
                        .push(security_requirement);
                }
            }
        }

        security
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use openapi::v3_0::{MediaTypeExample, ObjectOrReference, Parameter, Schema};
    use openapi::OpenApi;
    use postman::Spec;

    #[test]
    fn test_generate_path_parameters() {
        let mut state = State::default();
        let postman_variables = Some(vec![postman::Variable {
            key: Some("test".to_string()),
            value: Some(serde_json::Value::String("test_value".to_string())),
            description: None,
            ..postman::Variable::default()
        }]);
        let path_params = ["/test/".to_string(), "{{test_value}}".to_string()];
        let params =
            OpenApi30Backend::create_path_parameters(&mut state, &path_params, &postman_variables);
        assert_eq!(params.unwrap().len(), 1);
    }

    #[test]
    fn test_generate_query_parameters() {
        let mut state = State::default();
        let query_params = vec![postman::QueryParam {
            key: Some("test".to_string()),
            value: Some("{{test}}".to_string()),
            description: None,
            ..postman::QueryParam::default()
        }];
        let params = OpenApi30Backend::create_query_parameters(&mut state, &query_params);
        assert_eq!(params.unwrap().len(), 1);
    }

    #[test]
    fn it_preserves_order_on_paths() {
        let spec: Spec = serde_json::from_str(get_fixture("echo.postman.json").as_ref()).unwrap();
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
        let spec: Spec = serde_json::from_str(get_fixture("echo.postman.json").as_ref()).unwrap();
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
        let spec: Spec = serde_json::from_str(get_fixture("echo.postman.json").as_ref()).unwrap();
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
        let spec: Spec =
            serde_json::from_str(get_fixture("only-root-path.postman.json").as_ref()).unwrap();
        let oas = Transpiler::transpile(spec);
        match oas {
            OpenApi::V3_0(oas) => {
                assert!(oas.paths.contains_key("/"));
            }
        }
    }

    #[test]
    fn it_parses_graphql_request_bodies() {
        let spec: Spec =
            serde_json::from_str(get_fixture("graphql.postman.json").as_ref()).unwrap();
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
        let spec: Spec =
            serde_json::from_str(get_fixture("duplicate-query-params.postman.json").as_ref())
                .unwrap();
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
        let spec: Spec = serde_json::from_str(get_fixture("echo.postman.json").as_ref()).unwrap();
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
