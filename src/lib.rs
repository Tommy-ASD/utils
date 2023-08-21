pub mod csv2json;
pub mod error_types;
pub mod geojson;
pub mod http;
pub mod json;

/*
This function is simply the input() function from Python, as getting input is a bit annoying in Rust.
*/
pub fn input() -> String {
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(_) => {}
        Err(e) => {
            println!("Error reading line: {}", e);
            return input();
        }
    }
    line.trim().to_string()
}

/// Prints the type of a value
/// Example:
/// ```
/// let x = 5;
/// print_type_of(&x);
/// // Prints "i32"
/// ```
pub fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
