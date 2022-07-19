#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod unit_tests {
    use postman2openapi::openapi::OpenApi;
    use postman2openapi::postman::Spec;
    use postman2openapi::Transpiler;

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
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
        if let OpenApi::V3_0(s) = oas {
            let keys = s.paths.keys().enumerate();
            for (i, k) in keys {
                assert_eq!(k, ordered_paths[i])
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
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
