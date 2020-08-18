use postman2openapi::{postman, Transpiler};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).unwrap();
    match std::fs::File::open(filename) {
        Ok(r) => match serde_json::from_reader::<_, postman::Spec>(r) {
            Ok(spec) => {
                if let Ok(yaml) = Transpiler::transpile(spec) {
                    println!("{}", yaml);
                }
            }
            Err(err) => eprintln!("{}", err),
        },
        Err(err) => eprintln!("{}", err),
    }
}
