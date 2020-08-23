use clap::{App, Arg};
use std::io::{stdin, Read};

fn main() {
    let mut app = App::new("postman2openapi")
        .version("0.1.0")
        .author("Kevin Swiber <kswiber@gmail.com>")
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
    let output_format = matches.value_of_t("output").unwrap_or_else(|e| e.exit());
    match &matches.value_of("INPUT") {
        Some(filename) => match postman2openapi::from_path(filename, output_format) {
            Ok(oas) => println!("{}", oas),
            Err(err) => eprintln!("{}", err),
        },
        None => match stdin().read_to_string(&mut buffer) {
            Ok(_) => match postman2openapi::from_str(&buffer, output_format) {
                Ok(oas) => println!("{}", oas),
                Err(err) => eprintln!("{}", err),
            },
            Err(_) => eprintln!("postman2openapi: warning: recursive search of stdin"),
        },
    };
}
