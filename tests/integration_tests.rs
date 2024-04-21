use std::fs::File;
use std::io::prelude::*;


macro_rules! test_fixture {
    ($name:ident, $filename:expr) => {
        #[test]
        fn $name() {
            let filename = get_fixture($filename);
                let options = postman2openapi::TranspileOptions {
                    format: postman2openapi::TargetFormat::Json,
                };
            match postman2openapi::from_path(&filename, options) {
                Ok(_oas) => {
                   // Specify the target file path convert filename to filename_oas.json
                   let file_path = filename.replace("postman.json", "oas.json");

                    // Remove file if it exists
                    std::fs::remove_file(file_path.clone()).unwrap_or_default();

                    // Open or create the file
                    let mut file = File::create(file_path).expect("Failed to create file");

                    // Write data to the file
                    match file.write_all(_oas.as_bytes()) {
                        Ok(_) => assert!(true),
                        Err(err) => assert!(false, "{:?}", err),
                    }
                },
                Err(err) => {
                    assert!(false, "{:?}", err);
                },
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
test_fixture!(it_parses_oauth2_code_collection, "oauth2-code.postman.json");
test_fixture!(it_parses_api_key_collection, "api-key.postman.json");
test_fixture!(
    it_parses_empty_header_object_collection,
    "empty-header-object.postman.json"
);

fn get_fixture(filename: &str) -> String {
    let filename: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "./tests/fixtures/", filename]
        .iter()
        .collect();
    filename.into_os_string().into_string().unwrap()
}
