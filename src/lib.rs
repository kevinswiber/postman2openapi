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
use openapi::v3_0::{self as openapi3, ObjectOrReference, Parameter};
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

pub struct Transpiler<'a> {
    variable_map: &'a BTreeMap<String, serde_json::value::Value>,
}

struct TranspileState<'a> {
    oas: &'a mut openapi3::Spec,
    operation_ids: &'a mut BTreeMap<String, usize>,
    hierarchy: &'a mut Vec<String>,
}

impl<'a> Transpiler<'a> {
    pub fn transpile(spec: postman::Spec) -> openapi::OpenApi {
        let description = extract_description(&spec.info.description);

        let mut oas = openapi3::Spec {
            openapi: String::from("3.0.3"),
            info: openapi3::Info {
                license: None,
                contact: Some(openapi3::Contact::default()),
                description,
                terms_of_service: None,
                version: String::from("1.0.0"),
                title: spec.info.name,
            },
            components: None,
            external_docs: None,
            paths: IndexMap::new(),
            servers: Some(Vec::<openapi3::Server>::new()),
            tags: Some(IndexSet::<openapi3::Tag>::new()),
        };

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

        let mut operation_ids = BTreeMap::<String, usize>::new();
        let mut hierarchy = Vec::<String>::new();
        let mut state = TranspileState {
            oas: &mut oas,
            operation_ids: &mut operation_ids,
            hierarchy: &mut hierarchy,
        };

        let transpiler = Transpiler {
            variable_map: &mut variable_map,
        };

        transpiler.transform(&mut state, &spec.item);

        openapi::OpenApi::V3_0(Box::new(oas))
    }

    fn transform(&self, state: &mut TranspileState, items: &[postman::Items]) {
        for item in items {
            if let Some(i) = &item.item {
                let name = match &item.name {
                    Some(n) => n,
                    None => "<folder>",
                };
                let description = extract_description(&item.description);

                self.transform_folder(state, i, name, description);
            } else {
                self.transform_request(state, item);
            }
        }
    }

    fn transform_folder(
        &self,
        state: &mut TranspileState,
        items: &[postman::Items],
        name: &str,
        description: Option<String>,
    ) {
        if let Some(t) = &mut state.oas.tags {
            let mut tag = openapi3::Tag {
                name: name.to_string(),
                description,
            };

            let mut i: usize = 0;
            while t.contains(&tag) {
                i += 1;
                tag.name = format!("{tagName}{i}", tagName = tag.name);
            }

            let name = tag.name.clone();
            t.insert(tag);

            state.hierarchy.push(name);
            self.transform(state, items);
            state.hierarchy.pop();
        };
    }

    fn transform_request(&self, state: &mut TranspileState, item: &postman::Items) {
        let name = match &item.name {
            Some(n) => n,
            None => "<request>",
        };
        if let Some(postman::RequestUnion::RequestClass(request)) = &item.request {
            if let Some(postman::Url::UrlClass(u)) = &request.url {
                if let Some(postman::Host::StringArray(parts)) = &u.host {
                    self.transform_server(state, u, parts);
                }

                let root_path: Vec<postman::PathElement> = vec![];
                let paths = match &u.path {
                    Some(postman::UrlPath::UnionArray(p)) => p,
                    _ => &root_path,
                };

                self.transform_paths(state, item, request, name, u, paths)
            }
        }
    }

    fn transform_server(
        &self,
        state: &mut TranspileState,
        url: &postman::UrlClass,
        parts: &[String],
    ) {
        let host = parts.join(".");
        let mut proto = "".to_string();
        if let Some(protocol) = &url.protocol {
            proto = format!("{protocol}://", protocol = protocol.clone());
        }
        if let Some(s) = &mut state.oas.servers {
            let mut server_url = format!("{proto}{host}");
            server_url = self.resolve_variables(&server_url, VAR_REPLACE_CREDITS);
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

    fn transform_paths(
        &self,
        state: &mut TranspileState,
        item: &postman::Items,
        request: &postman::RequestClass,
        request_name: &str,
        url: &postman::UrlClass,
        paths: &[postman::PathElement],
    ) {
        let resolved_segments = paths
            .iter()
            .map(|segment| {
                let mut seg = match segment {
                    postman::PathElement::PathClass(c) => c.clone().value.unwrap_or_default(),
                    postman::PathElement::String(c) => c.to_string(),
                };
                seg = self.resolve_variables_with_replace_fn(&seg, VAR_REPLACE_CREDITS, |s| {
                    VARIABLE_RE.replace_all(&s, "{$1}").to_string()
                });
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
        if !state.oas.paths.contains_key(&segments) {
            state
                .oas
                .paths
                .insert(segments.clone(), openapi3::PathItem::default());
        }

        if let Some(path) = state.oas.paths.get_mut(&segments) {
            // description must exist on a path
            let description = match extract_description(&request.description) {
                Some(desc) => Some(desc),
                None => Some(request_name.to_string()),
            };

            path.parameters = self.generate_path_parameters(&resolved_segments, &url.variable);

            let mut op = openapi3::Operation::default();

            if let Some(qp) = &url.query {
                if let Some(mut query_params) = self.generate_query_parameters(qp) {
                    match &op.parameters {
                        Some(params) => {
                            let mut cloned = params.clone();
                            cloned.append(&mut query_params);
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
                        let param = ObjectOrReference::Object(Parameter {
                            location: "header".to_owned(),
                            name: header.key.to_owned(),
                            description: extract_description(&header.description),
                            schema: Some(openapi3::Schema {
                                schema_type: Some("string".to_owned()),
                                example: Some(serde_json::Value::String(header.value.to_owned())),
                                ..openapi3::Schema::default()
                            }),
                            ..Parameter::default()
                        });

                        if op.parameters.is_none() {
                            op.parameters = Some(vec![param]);
                        } else {
                            op.parameters.as_mut().unwrap().push(param);
                        }
                    }
                }
            }

            if let Some(body) = &request.body {
                self.extract_request_body(body, &mut op, content_type);
            }

            op.summary = Some(request_name.to_string());
            op.description = description;

            if !state.hierarchy.is_empty() {
                op.tags = Some(state.hierarchy.clone());
            }

            if let Some(responses) = &item.response {
                for r in responses.iter() {
                    let mut oas_response = openapi3::Response::default();
                    let mut response_media_types = BTreeMap::<String, openapi3::MediaType>::new();
                    if let Some(res) = r {
                        if let Some(name) = &res.name {
                            oas_response.description = Some(name.clone());
                        }
                        if let Some(postman::Headers::UnionArray(headers)) = &res.header {
                            let mut oas_headers = BTreeMap::<
                                String,
                                openapi3::ObjectOrReference<openapi3::Header>,
                            >::new();
                            for h in headers {
                                if let postman::HeaderElement::Header(hdr) = h {
                                    if hdr.value.is_empty()
                                        || hdr.key.to_lowercase() == "content-type"
                                    {
                                        continue;
                                    }
                                    let mut oas_header = openapi3::Header::default();
                                    let header_schema = openapi3::Schema {
                                        schema_type: Some("string".to_string()),
                                        example: Some(serde_json::Value::String(
                                            hdr.value.to_string(),
                                        )),
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
                        if let Some(raw) = &res.body {
                            let mut response_content_type: Option<String> = None;
                            let resolved_body = self.resolve_variables(raw, VAR_REPLACE_CREDITS);
                            let example_val;

                            match serde_json::from_str(&resolved_body) {
                                Ok(v) => match v {
                                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                                        response_content_type =
                                            Some("application/json".to_string());
                                        if let Some(schema) = Self::generate_schema(&v) {
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
                            let mut example_map = BTreeMap::<
                                String,
                                openapi3::ObjectOrReference<openapi3::Example>,
                            >::new();

                            let ex = openapi3::Example {
                                summary: None,
                                description: None,
                                value: Some(example_val),
                            };

                            let example_name = match &res.name {
                                Some(n) => n.to_string(),
                                None => "".to_string(),
                            };

                            example_map
                                .insert(example_name, openapi3::ObjectOrReference::Object(ex));
                            let example = openapi3::MediaTypeExample::Examples {
                                examples: example_map,
                            };

                            response_content.examples = Some(example);

                            if response_content_type.is_none() {
                                response_content_type =
                                    Some("application/octet-stream".to_string());
                            }

                            response_media_types.insert(
                                response_content_type.unwrap().to_string(),
                                response_content,
                            );
                        }
                        oas_response.content = Some(response_media_types);
                        if let Some(code) = &res.code {
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

            if let Some(method) = &request.method {
                let m = method.to_lowercase();
                let mut op_id = request_name
                    .chars()
                    .map(|c| match c {
                        'A'..='Z' | 'a'..='z' | '0'..='9' => c,
                        _ => ' ',
                    })
                    .collect::<String>()
                    .from_case(Case::Title)
                    .to_case(Case::Camel);
                match state.operation_ids.get_mut(&op_id) {
                    Some(v) => {
                        *v += 1;
                        op_id = format!("{op_id}{v}");
                    }
                    None => {
                        state.operation_ids.insert(op_id.clone(), 0);
                    }
                }

                op.operation_id = Some(op_id);
                match m.as_str() {
                    "get" => {
                        path.get = Some(op);
                    }
                    "post" => {
                        path.post = Some(op);
                    }
                    "put" => {
                        path.put = Some(op);
                    }
                    "delete" => {
                        path.delete = Some(op);
                    }
                    "patch" => {
                        path.patch = Some(op);
                    }
                    "options" => {
                        path.options = Some(op);
                    }
                    "trace" => {
                        path.trace = Some(op);
                    }
                    _ => {}
                }
            }
        }
    }

    fn extract_request_body(
        &self,
        body: &postman::Body,
        op: &mut openapi3::Operation,
        ct: Option<String>,
    ) {
        let mut content_type = ct;
        let mut request_body = openapi3::RequestBody::default();
        let mut content = openapi3::MediaType::default();

        if let Some(mode) = &body.mode {
            match mode {
                postman::Mode::Raw => {
                    content_type = Some("application/octet-stream".to_string());
                    if let Some(raw) = &body.raw {
                        let resolved_body = self.resolve_variables(raw, VAR_REPLACE_CREDITS);
                        let example_val;

                        //set content type based on options or inference.
                        match serde_json::from_str(&resolved_body) {
                            Ok(v) => match v {
                                serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                                    content_type = Some("application/json".to_string());
                                    if let Some(schema) = Self::generate_schema(&v) {
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
                                // TODO: Check if XML, HTML, JavaScript
                                content_type = Some("text/plain".to_string());
                                example_val = serde_json::Value::String(resolved_body);
                            }
                        }

                        let example = openapi3::MediaTypeExample::Example {
                            example: example_val,
                        };
                        content.examples = Some(example);
                    }
                }
                postman::Mode::Urlencoded => {
                    content_type = Some("application/x-www-form-urlencoded".to_string());
                    if let Some(urlencoded) = &body.urlencoded {
                        let mut oas_data = serde_json::Map::new();
                        for i in urlencoded {
                            if let Some(v) = &i.value {
                                let value = serde_json::Value::String(v.to_string());
                                oas_data.insert(i.key.clone(), value);
                            }
                        }
                        let oas_obj = serde_json::Value::Object(oas_data);
                        if let Some(schema) = Self::generate_schema(&oas_obj) {
                            content.schema = Some(openapi3::ObjectOrReference::Object(schema));
                        }
                        let example = openapi3::MediaTypeExample::Example { example: oas_obj };
                        content.examples = Some(example);
                    }
                }
                postman::Mode::Formdata => {
                    content_type = Some("multipart/form-data".to_string());
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
                                    let prop_schema = Self::generate_schema(&value);
                                    if let Some(mut prop_schema) = prop_schema {
                                        if is_binary {
                                            prop_schema.format = Some("binary".to_string());
                                        }
                                        prop_schema.description =
                                            extract_description(&i.description);
                                        properties.insert(i.key.clone(), prop_schema);
                                    }
                                } else {
                                    let mut prop_schema = openapi3::Schema {
                                        schema_type: Some("string".to_string()),
                                        description: extract_description(&i.description),
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
            content_type = Some("application/octet-stream".to_string())
        }

        request_body.content = BTreeMap::<String, openapi3::MediaType>::new();
        request_body.content.insert(content_type.unwrap(), content);
        op.request_body = Some(openapi3::ObjectOrReference::Object(request_body));
    }

    fn resolve_variables(&self, segment: &str, sub_replace_credits: usize) -> String {
        self.resolve_variables_with_replace_fn(segment, sub_replace_credits, |s| s)
    }

    fn resolve_variables_with_replace_fn(
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
                    if let Some(v) = self.variable_map.get(capture) {
                        if let Some(v2) = v.as_str() {
                            let re = regex::Regex::new(&regex::escape(&cap[0])).unwrap();
                            return self.resolve_variables(
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

    fn generate_schema(value: &serde_json::Value) -> Option<openapi3::Schema> {
        match value {
            serde_json::Value::Object(m) => {
                let mut schema = openapi3::Schema {
                    schema_type: Some("object".to_string()),
                    ..Default::default()
                };

                let mut properties = BTreeMap::<String, openapi3::Schema>::new();

                for (key, val) in m.iter() {
                    if let Some(v) = Self::generate_schema(val) {
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
                        if let Some(i) = Self::generate_schema(i) {
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

    fn generate_path_parameters(
        &self,
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
                                param.description = extract_description(&p.description);
                                if let Some(pval) = &p.value {
                                    if let Some(pval_val) = pval.as_str() {
                                        schema.example = Some(serde_json::Value::String(
                                            self.resolve_variables(pval_val, VAR_REPLACE_CREDITS),
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

    fn generate_query_parameters(
        &self,
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
                        description: extract_description(&qp.description),
                        location: "query".to_owned(),
                        schema: Some(openapi3::Schema {
                            schema_type: Some("string".to_string()),
                            example: qp.value.as_ref().map(|pval| {
                                serde_json::Value::String(
                                    self.resolve_variables(pval, VAR_REPLACE_CREDITS),
                                )
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
}

fn extract_description(description: &Option<postman::DescriptionUnion>) -> Option<String> {
    match description {
        Some(d) => match d {
            postman::DescriptionUnion::String(s) => Some(s.to_string()),
            postman::DescriptionUnion::Description(desc) => {
                desc.content.as_ref().map(|c| c.to_string())
            }
        },
        None => None,
    }
}
