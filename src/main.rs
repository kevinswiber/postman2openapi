fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).unwrap();

    match postman2openapi::from_path(filename) {
        Ok(oas) => println!("{}", oas),
        Err(err) => eprintln!("{}", err),
    }
}
