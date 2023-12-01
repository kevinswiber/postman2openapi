#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod wasm_browser {
    use js_sys::{Function, JSON};
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn it_transpiles() {
        let collection: &'static str = include_str!("./fixtures/wasm/collection.json");
        let openapi: &'static str = include_str!("./fixtures/wasm/openapi.json");

        let replacer = Function::new_with_args(
            "key, value",
            "if (value instanceof Map) { return Object.fromEntries(value); } return value;",
        );
        match postman2openapi::transpile(JSON::parse(collection).unwrap()) {
            Ok(oas) => assert_eq!(
                JSON::stringify(&JSON::parse(openapi).unwrap()).unwrap(),
                JSON::stringify_with_replacer(&oas, &JsValue::from(replacer)).unwrap()
            ),
            Err(err) => panic!("Couldn't convert collection to OpenAPI: {:?}", err),
        };
    }
}
