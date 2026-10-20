fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program: &String = &args[0];
    let program_args: &[String] = &args[1..];
    println!("Running {:?} with arguments {:?}", program, program_args);
    println!("Hello, world!");
}
