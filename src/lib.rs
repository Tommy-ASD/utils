pub mod async_utils;
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

/// Gets the type of a value as a string,
/// and returns it.
/// Example:
/// ```
/// let x = 5;
/// type_of(&x);
/// // Returns "i32"
/// ```
pub fn type_of<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}

/// Gets the type of a value as a string,
/// and prints it.
/// Example:
/// ```
/// let x = 5;
/// print_type_of(&x);
/// // Prints "i32"
/// ```
pub fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

use std::future::Future;
use std::pin::Pin;

use crate::error_types::{warn_devs, TracebackError};

pub trait TracebackCallback {
    fn call(&self, error: TracebackError);
}

// Define a trait that represents a function returning a Future
pub trait TracebackCallbackAsync {
    fn call(&self, error: TracebackError) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>;
}

pub enum TracebackCallbackType {
    Async(Box<dyn TracebackCallbackAsync + Send + Sync>),
    Sync(Box<dyn TracebackCallback + Send + Sync>),
}

// Define a concrete type that implements the trait
struct WarnDevsFunction;
impl TracebackCallbackAsync for WarnDevsFunction {
    fn call(&self, error: TracebackError) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> {
        Box::pin(warn_devs(error)) // Use Box::pin to pin the future
    }
}

pub static mut TRACEBACK_ERROR_CALLBACK: Option<TracebackCallbackType> = None;

pub fn set_traceback_callback(callback: TracebackCallbackType) {
    unsafe {
        TRACEBACK_ERROR_CALLBACK = Some(callback);
    }
}

pub fn init() {
    set_traceback_callback(TracebackCallbackType::Async(Box::new(WarnDevsFunction {})));
}
