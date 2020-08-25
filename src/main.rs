use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use lazy_static::lazy_static;
use postman2openapi::{from_path, from_str, TranspileOptions};
use std::io::{stdin, Read};

fn main() {
    lazy_static! {
        static ref LONG_VERSION: String = long_version(None, None);
    }

    let authors = crate_authors!("\n");
    let mut app = App::new("postman2openapi")
        .version(crate_version!())
        .long_version(LONG_VERSION.as_str())
        .author(authors)
        .setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name("output")
                .short('o')
                .long("output")
                .about("The output format")
                .value_name("format")
                .possible_values(&["yaml", "json"])
                .default_value("yaml"),
        )
        .arg(
            Arg::with_name("INPUT")
                .value_name("input-file")
                .about("The Postman collection to convert; data may also come from stdin")
                .index(1),
        );

    if std::env::args().len() < 2 && atty::is(atty::Stream::Stdin) {
        let _ = app.print_help();
        return;
    }

    let matches = app.get_matches();

    let mut buffer = String::new();
    let format = matches.value_of_t("output").unwrap_or_else(|e| e.exit());
    match &matches.value_of("INPUT") {
        Some(filename) => match from_path(filename, TranspileOptions { format }) {
            Ok(oas) => println!("{}", oas),
            Err(err) => eprintln!("{}", err),
        },
        None => match stdin().read_to_string(&mut buffer) {
            Ok(_) => match from_str(&buffer, TranspileOptions { format }) {
                Ok(oas) => println!("{}", oas),
                Err(err) => eprintln!("{}", err),
            },
            Err(_) => eprintln!("postman2openapi: warning: recursive search of stdin"),
        },
    };
}

pub fn long_version(revision_hash: Option<&str>, date: Option<&str>) -> String {
    let hash = match revision_hash.or(option_env!("POSTMAN2OPENAPI_BUILD_GIT_HASH")) {
        None => String::new(),
        Some(hash) => format!("commit: {}", hash),
    };

    let date = match date.or(option_env!("POSTMAN2OPENAPI_BUILD_DATE")) {
        None => String::new(),
        Some(date) => format!("date: {}", date),
    };

    format!("{}\n{}\n{}\n", crate_version!(), hash, date)
}
