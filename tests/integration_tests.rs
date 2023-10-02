macro_rules! test_fixture {
    ($name:ident, $filename:expr) => {
        #[test]
        fn $name() {
            let filename = get_fixture($filename);
            let options = postman2openapi::TranspileOptions {
                format: postman2openapi::TargetFormat::Json,
            };
            match postman2openapi::from_path(&filename, options) {
                Ok(_oas) => assert!(true),
                Err(err) => assert!(false, "{:?}", err),
            }
        }
    };
}

test_fixture!(it_parses_github_api_collection, "github.postman.json");
test_fixture!(it_parses_postman_api_collection, "postman-api.postman.json");
test_fixture!(it_parses_pdf_co_collection, "pdfco.postman.json");
test_fixture!(it_parses_postman_echo_collection, "echo.postman.json");
test_fixture!(it_parses_twitter_api_collection, "twitter-api.postman.json");
test_fixture!(it_parses_fastly_api_collection, "fastly.postman.json");
test_fixture!(it_parses_users_api_collection, "users.postman.json");
test_fixture!(it_parses_graphql_api_collection, "graphql.postman.json");
test_fixture!(it_parses_todo_api_collection, "todo.postman.json");
test_fixture!(
    it_parses_gotomeeting_api_collection,
    "gotomeeting.postman.json"
);
test_fixture!(
    it_parses_calculator_soap_collection,
    "calculator-soap.postman.json"
);

fn get_fixture(filename: &str) -> String {
    let filename: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "./tests/fixtures/", filename]
        .iter()
        .collect();
    filename.into_os_string().into_string().unwrap()
}
