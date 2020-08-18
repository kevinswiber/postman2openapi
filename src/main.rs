#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod postman;

use convert_case::{Case, Casing};
use openapi::v3_0 as openapi3;
use std::collections::BTreeMap;

static VAR_REPLACE_CREDITS: usize = 20;

lazy_static! {
    static ref VARIABLE_RE: regex::Regex = regex::Regex::new(r"\{\{([^{}]*?)\}\}").unwrap();
    static ref URI_TEMPLATE_VARIABLE_RE: regex::Regex =
        regex::Regex::new(r"\{([^{}]*?)\}").unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).unwrap();
    match std::fs::File::open(filename) {
        Ok(r) => match serde_json::from_reader::<_, postman::Spec>(r) {
            Ok(spec) => begin(spec),
            Err(err) => eprintln!("{}", err),
        },
        Err(err) => eprintln!("{}", err),
    }
}

fn begin(spec: postman::Spec) {
    let description = match &spec.info.description {
        Some(d) => match d {
            postman::DescriptionUnion::String(s) => Some(s.to_string()),
            postman::DescriptionUnion::Description(desc) => match &desc.content {
                Some(c) => Some(c.to_string()),
                None => None,
            },
        },
        None => None,
    };

    let mut oas = openapi3::Spec {
        openapi: String::from("3.0.3"),
        info: openapi3::Info {
            license: None,
            contact: Some(openapi3::Contact::default()),
            description: description,
            terms_of_service: None,
            version: String::from("1.0.0"),
            title: spec.info.name,
        },
        components: None,
        external_docs: None,
        paths: BTreeMap::new(),
        servers: Some(Vec::<openapi3::Server>::new()),
        tags: Some(Vec::<openapi3::Tag>::new()),
    };

    let mut variable_map = BTreeMap::<String, serde_json::value::Value>::new();
    &spec.variable.map(|var| {
        for v in var {
            if let Some(v_name) = v.key {
                if let Some(v_val) = v.value {
                    if v_val != serde_json::Value::String("".to_string()) {
                        variable_map.insert(v_name, v_val);
                    }
                }
            }
        }
    });

    let mut operation_ids = BTreeMap::<String, usize>::new();
    transform(
        &spec.item,
        &mut oas,
        &mut Vec::<String>::new(),
        &mut variable_map,
        &mut operation_ids,
    );

    println!(
        "{}",
        openapi::to_yaml(&openapi::OpenApi::V3_0(oas)).unwrap()
    );
}

fn transform(
    items: &Vec<postman::Items>,
    oas: &mut openapi3::Spec,
    hierarchy: &mut Vec<String>,
    variable_map: &BTreeMap<String, serde_json::value::Value>,
    operation_ids: &mut BTreeMap<String, usize>,
) {
    for item in items {
        if let Some(i) = &item.item {
            let name = match &item.name {
                Some(n) => n,
                None => "<folder>",
            };
            let description = match &item.description {
                Some(d) => match d {
                    postman::DescriptionUnion::String(s) => Some(s.to_string()),
                    postman::DescriptionUnion::Description(desc) => match &desc.content {
                        Some(c) => Some(c.to_string()),
                        None => None,
                    },
                },
                None => None,
            };

            transform_folder(
                &i,
                name,
                description,
                oas,
                hierarchy,
                variable_map,
                operation_ids,
            );
        } else {
            transform_request(&item, oas, hierarchy, variable_map, operation_ids);
        }
    }
}

fn transform_folder(
    items: &Vec<postman::Items>,
    name: &str,
    description: Option<String>,
    oas: &mut openapi3::Spec,
    hierarchy: &mut Vec<String>,
    variable_map: &BTreeMap<String, serde_json::value::Value>,
    operation_ids: &mut BTreeMap<String, usize>,
) {
    if let Some(t) = &mut oas.tags {
        t.push(openapi3::Tag {
            name: name.to_string(),
            description: description,
        });
    };

    hierarchy.push(name.to_string());
    transform(items, oas, hierarchy, variable_map, operation_ids);
    hierarchy.pop();
}

fn transform_request(
    item: &postman::Items,
    oas: &mut openapi3::Spec,
    hierarchy: &mut Vec<String>,
    variable_map: &BTreeMap<String, serde_json::value::Value>,
    operation_ids: &mut BTreeMap<String, usize>,
) {
    let name = match &item.name {
        Some(n) => n,
        None => "<request>",
    };
    if let Some(r) = &item.request {
        if let postman::RequestUnion::RequestClass(request) = r {
            if let Some(postman::Url::UrlClass(u)) = &request.url {
                if let Some(postman::Host::StringArray(parts)) = &u.host {
                    let host = parts.join(".");
                    let mut proto = "".to_string();
                    if let Some(protocol) = &u.protocol {
                        proto = format!("{}://", protocol.clone());
                    }
                    if let Some(s) = &mut oas.servers {
                        let mut server_url = format!("{}{}", proto, host);
                        server_url =
                            resolve_variables(&variable_map, &server_url, VAR_REPLACE_CREDITS);
                        if !s.into_iter().any(|srv| srv.url == server_url) {
                            let server = openapi3::Server {
                                url: server_url,
                                description: None,
                                variables: None,
                            };
                            s.push(server);
                        }
                    }
                }

                if let Some(postman::UrlPath::UnionArray(p)) = &u.path {
                    let resolved_segments = &p
                        .iter()
                        .map(|segment| {
                            let mut seg = match segment {
                                postman::PathElement::PathClass(c) => {
                                    c.clone().value.unwrap_or_default()
                                }
                                postman::PathElement::String(c) => c.to_string(),
                            };
                            seg = resolve_variables_with_replace_fn(
                                &variable_map,
                                &seg,
                                VAR_REPLACE_CREDITS,
                                |s| VARIABLE_RE.replace_all(&s, "{$1}").to_string(),
                            );
                            match &seg[0..1] {
                                ":" => format!("{{{}}}", &seg[1..]),
                                _ => seg.to_string(),
                            }
                        })
                        .collect::<Vec<String>>();
                    let segments = "/".to_string() + &resolved_segments.join("/");

                    // TODO: Because of variables, we can actually get duplicate paths.
                    // - /admin/{subresource}/{subresourceId}
                    // - /admin/{subresource2}/{subresource2Id}
                    // Throw a warning?
                    if !oas.paths.contains_key(&segments) {
                        oas.paths
                            .insert(segments.clone(), openapi3::PathItem::default());
                    }

                    if let Some(path) = oas.paths.get_mut(&segments) {
                        let description = match &request.description {
                            Some(d) => match d {
                                postman::DescriptionUnion::String(s) => Some(s.to_string()),
                                postman::DescriptionUnion::Description(desc) => {
                                    match &desc.content {
                                        Some(c) => Some(c.to_string()),
                                        None => Some(name.to_string()),
                                    }
                                }
                            },
                            None => Some(name.to_string()),
                        };

                        let mut op = openapi3::Operation::default();

                        op.parameters = generate_path_parameters(
                            &variable_map,
                            &resolved_segments,
                            &u.variable,
                        );

                        let mut content_type: Option<String> = None;
                        if let Some(postman::HeaderUnion::HeaderArray(headers)) = &request.header {
                            let content_type_header = headers
                                .iter()
                                .find(|h| h.key.to_lowercase() == "content-type");
                            if let Some(t) = content_type_header {
                                let content_type_parts: Vec<&str> = t.value.split(';').collect();
                                content_type = Some(content_type_parts[0].to_string());
                            }
                        }

                        if let Some(body) = &request.body {
                            let mut request_body = openapi3::RequestBody::default();
                            let mut content = openapi3::MediaType::default();

                            if let Some(mode) = &body.mode {
                                match mode {
                                    postman::Mode::Raw => {
                                        content_type = Some("application/octet-stream".to_string());
                                        if let Some(raw) = &body.raw {
                                            let resolved_body = resolve_variables(
                                                &variable_map,
                                                &raw,
                                                VAR_REPLACE_CREDITS,
                                            );
                                            let example_val;

                                            //set content type based on options or inference.
                                            match serde_json::from_str(&resolved_body) {
                                                Ok(v) => match v {
                                                    serde_json::Value::Object(_)
                                                    | serde_json::Value::Array(_) => {
                                                        content_type =
                                                            Some("application/json".to_string());
                                                        if let Some(schema) = generate_schema(&v) {
                                                            content.schema = Some(
                                                                openapi3::ObjectOrReference::Object(
                                                                    schema,
                                                                ),
                                                            );
                                                        }
                                                        example_val = v;
                                                    }
                                                    _ => {
                                                        example_val = serde_json::Value::String(
                                                            resolved_body,
                                                        );
                                                    }
                                                },
                                                _ => {
                                                    // TODO: Check if XML, HTML, JavaScript
                                                    content_type = Some("text/plain".to_string());
                                                    example_val =
                                                        serde_json::Value::String(resolved_body);
                                                }
                                            }

                                            let example = openapi3::MediaTypeExample::Example {
                                                example: example_val,
                                            };
                                            content.examples = Some(example);
                                        }
                                    }
                                    postman::Mode::Urlencoded => {
                                        content_type =
                                            Some("application/form-urlencoded".to_string());
                                        if let Some(urlencoded) = &body.urlencoded {
                                            let mut oas_data = serde_json::Map::new();
                                            for i in urlencoded {
                                                if let Some(v) = &i.value {
                                                    let value =
                                                        serde_json::Value::String(v.to_string());
                                                    oas_data.insert(i.key.clone(), value);
                                                }
                                            }
                                            let oas_obj = serde_json::Value::Object(oas_data);
                                            if let Some(schema) = generate_schema(&oas_obj) {
                                                content.schema = Some(
                                                    openapi3::ObjectOrReference::Object(schema),
                                                );
                                            }
                                            let example = openapi3::MediaTypeExample::Example {
                                                example: oas_obj,
                                            };
                                            content.examples = Some(example);
                                        }
                                    }
                                    _ => {
                                        content_type = Some("application/octet-stream".to_string())
                                    }
                                }
                            }

                            if content_type.is_none() {
                                content_type = Some("application/octet-stream".to_string())
                            }

                            request_body.content = BTreeMap::<String, openapi3::MediaType>::new();
                            request_body
                                .content
                                .insert(content_type.unwrap().to_string(), content);
                            op.request_body =
                                Some(openapi3::ObjectOrReference::Object(request_body));
                        }

                        op.summary = Some(name.to_string());
                        op.description = description;

                        if hierarchy.len() > 0 {
                            op.tags = Some(hierarchy.clone());
                        }

                        if let Some(responses) = &item.response {
                            for r in responses.iter() {
                                let mut oas_response = openapi3::Response::default();
                                let mut response_media_types =
                                    BTreeMap::<String, openapi3::MediaType>::new();
                                if let Some(res) = r {
                                    // TODO: Use Postman schema that includes response name.
                                    if let Some(name) = &res.name {
                                        oas_response.description = Some(name.clone());
                                    }
                                    let mut response_content = openapi3::MediaType::default();
                                    if let Some(raw) = &res.body {
                                        let mut response_content_type: Option<String> = None;
                                        let resolved_body = resolve_variables(
                                            &variable_map,
                                            &raw,
                                            VAR_REPLACE_CREDITS,
                                        );
                                        let example_val;

                                        //set content type based on options or inference.
                                        match serde_json::from_str(&resolved_body) {
                                            Ok(v) => match v {
                                                serde_json::Value::Object(_)
                                                | serde_json::Value::Array(_) => {
                                                    response_content_type =
                                                        Some("application/json".to_string());
                                                    if let Some(schema) = generate_schema(&v) {
                                                        response_content.schema = Some(
                                                            openapi3::ObjectOrReference::Object(
                                                                schema,
                                                            ),
                                                        );
                                                    }
                                                    example_val = v;
                                                }
                                                _ => {
                                                    example_val =
                                                        serde_json::Value::String(resolved_body);
                                                }
                                            },
                                            _ => {
                                                // TODO: Check if XML, HTML, JavaScript
                                                response_content_type =
                                                    Some("text/plain".to_string());
                                                example_val =
                                                    serde_json::Value::String(resolved_body);
                                            }
                                        }
                                        let example = openapi3::MediaTypeExample::Example {
                                            example: example_val,
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
                            let mut op_id = name.clone().to_case(Case::Camel);
                            match operation_ids.get_mut(&op_id) {
                                Some(v) => {
                                    *v = *v + 1;
                                    op_id = format!("{}{}", op_id, v);
                                }
                                None => {
                                    operation_ids.insert(op_id.clone(), 0);
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
            }
        }
    }
}

fn resolve_variables(
    variable_map: &BTreeMap<String, serde_json::value::Value>,
    segment: &str,
    sub_replace_credits: usize,
) -> String {
    return resolve_variables_with_replace_fn(variable_map, segment, sub_replace_credits, |s| s);
}

fn resolve_variables_with_replace_fn(
    variable_map: &BTreeMap<String, serde_json::value::Value>,
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
            for n in 1..=cap.len() - 1 {
                let capture = &cap[n].to_string();
                if let Some(v) = variable_map.get(capture) {
                    if let Some(v2) = v.as_str() {
                        let re = regex::Regex::new(&regex::escape(&cap[0])).unwrap();
                        return resolve_variables(
                            variable_map,
                            &re.replace_all(&s, v2).to_string(),
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
            let mut schema = openapi3::Schema::default();
            schema.schema_type = Some("object".to_string());
            let mut properties = BTreeMap::<String, openapi3::Schema>::new();

            for (key, val) in m.iter() {
                if let Some(v) = generate_schema(val) {
                    properties.insert(key.to_string(), v);
                }
            }

            schema.properties = Some(properties);
            Some(schema)
        }
        serde_json::Value::Array(a) => {
            let mut schema = openapi3::Schema::default();
            schema.schema_type = Some("array".to_string());
            if let Some(i) = &a.get(0) {
                if let Some(item_schema) = generate_schema(i) {
                    let mut mut_schema = item_schema.clone();
                    for n in 1..=a.len() - 1 {
                        if let Some(i2) = &a.get(n) {
                            if let Some(i2_inner) = generate_schema(i2) {
                                mut_schema = merge_schemas(&mut_schema, &i2_inner);
                            }
                        }
                    }
                    schema.items = Some(Box::new(mut_schema));
                }
            }
            Some(schema)
        }
        serde_json::Value::String(_) => {
            let mut schema = openapi3::Schema::default();
            schema.schema_type = Some("string".to_string());
            schema.example = Some(value.clone());
            Some(schema)
        }
        serde_json::Value::Number(_) => {
            let mut schema = openapi3::Schema::default();
            schema.schema_type = Some("number".to_string());
            schema.example = Some(value.clone());
            Some(schema)
        }
        serde_json::Value::Bool(_) => {
            let mut schema = openapi3::Schema::default();
            schema.schema_type = Some("boolean".to_string());
            schema.example = Some(value.clone());
            Some(schema)
        }
        serde_json::Value::Null => {
            let mut schema = openapi3::Schema::default();
            schema.nullable = Some(true);
            schema.example = Some(value.clone());
            Some(schema)
        }
    }
}

fn merge_schemas(original: &openapi3::Schema, new: &openapi3::Schema) -> openapi3::Schema {
    let mut cloned = original.clone();

    if cloned.nullable.is_none() && new.nullable.is_some() {
        cloned.nullable = new.nullable.clone();
    }

    if let Some(cloned_nullable) = cloned.nullable {
        if let Some(new_nullable) = new.nullable {
            if new_nullable != cloned_nullable {
                cloned.nullable = Some(true);
            }
        }
    }

    if cloned.schema_type.is_none() && new.schema_type.is_some() {
        cloned.schema_type = new.schema_type.clone();
    }
    if let Some(t) = &cloned.schema_type {
        match t.as_str() {
            "object" => {
                if let Some(properties) = &mut cloned.properties {
                    if let Some(new_properties) = &new.properties {
                        for (key, val) in properties.iter_mut() {
                            if let Some(v) = &new_properties.get(key) {
                                let prop_clone = v.clone();
                                *val = merge_schemas(&val, &prop_clone);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    cloned
}

fn generate_path_parameters(
    variable_map: &BTreeMap<String, serde_json::value::Value>,
    resolved_segments: &Vec<String>,
    postman_variables: &Option<Vec<postman::Variable>>,
) -> Option<Vec<openapi3::ObjectOrReference<openapi3::Parameter>>> {
    let params: Vec<openapi3::ObjectOrReference<openapi3::Parameter>> = resolved_segments
        .into_iter()
        .flat_map(|segment| {
            URI_TEMPLATE_VARIABLE_RE
                .captures_iter(segment.as_str())
                .map(|capture| {
                    let var = capture.get(1).unwrap().as_str();
                    let mut param = openapi3::Parameter::default();
                    param.name = var.to_string();
                    param.location = "path".to_string();
                    param.required = Some(true);
                    let mut schema = openapi3::Schema::default();
                    schema.schema_type = Some("string".to_string());
                    if let Some(path_val) = &postman_variables {
                        if let Some(p) = path_val.iter().find(|p| match &p.key {
                            Some(k) => k == &var,
                            _ => false,
                        }) {
                            if let Some(pval) = &p.value {
                                if let Some(pval_val) = pval.as_str() {
                                    schema.example =
                                        Some(serde_json::Value::String(resolve_variables(
                                            &variable_map,
                                            pval_val,
                                            VAR_REPLACE_CREDITS,
                                        )));
                                }
                            }
                        }
                    }

                    param.schema = Some(schema);
                    openapi3::ObjectOrReference::Object(param)
                })
        })
        .collect();

    if params.len() > 0 {
        Some(params)
    } else {
        None
    }
}
