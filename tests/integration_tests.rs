#[cfg(test)]
mod integration_tests {
    macro_rules! test_fixture {
        ($name:ident, $filename:expr) => {
            #[test]
            fn $name() {
                let filename = get_fixture($filename);
                match postman2openapi::from_path(&filename, postman2openapi::TargetFormat::Yaml) {
                    Ok(_oas) => assert!(true),
                    Err(_err) => assert!(false),
                }
            }
        };
    }

    test_fixture!(it_parses_github_api_collection, "github.postman.json");
    test_fixture!(it_parses_postman_api_collection, "postman-api.postman.json");
    test_fixture!(it_parses_postman_echo_collection, "echo.postman.json");
    test_fixture!(it_parses_twitter_api_collection, "twitter-api.postman.json");
    test_fixture!(it_parses_fastly_api_collection, "fastly.postman.json");
    test_fixture!(it_parses_users_api_collection, "users.postman.json");

    fn get_fixture(filename: &str) -> String {
        let filename: std::path::PathBuf =
            [env!("CARGO_MANIFEST_DIR"), "./tests/fixtures/", filename]
                .iter()
                .collect();
        filename.into_os_string().into_string().unwrap()
    }
}
