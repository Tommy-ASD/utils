pub mod async_utils;
pub mod csv2json;
pub mod error_types;
pub mod geojson;
pub mod http;
pub mod json;

pub use utils_derive;

pub use paste;
pub use serde_json;

use std::future::Future;
use std::pin::Pin;

use crate::error_types::{warn_devs, TracebackError};

/*
This function is simply the input() function from Python, as getting input is a bit annoying in Rust.
*/
pub fn input() -> String {
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(_) => {}
        Err(e) => {
            println!("Error reading line: {}", e);
            println!("Please try again");
            return input();
        }
    }
    line.trim().to_string()
}

#[macro_export]
macro_rules! input {
    ($($arg:expr),*) => {{
        $(print!("{} ", $arg);)* // Print each argument followed by a space
        println!(); // Print a newline at the end

        input()
    }};
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

pub fn set_default_traceback_callback() {
    set_traceback_callback(TracebackCallbackType::Async(Box::new(WarnDevsFunction {})));
}

// macro to define and set the callback, with a function as a parameter
// should match to "set_traceback!(sync function)"
// and "set_traceback!(async function)"
// IMPORTANT: $callback must be a function. Cannot be a closure.
#[macro_export]
macro_rules! set_traceback {
    ($callback:ident) => {
        $crate::paste::unique_paste! {
            // Generate a unique identifier for the struct
            #[allow(non_camel_case_types)]
            mod [<_private_ $callback _ TempStruct>] {
                pub struct [<$callback _ TempStruct>];

                impl $crate::TracebackCallback for [<$callback _ TempStruct>] {
                    fn call(&self, error: $crate::error_types::TracebackError) {
                        super::$callback(error)
                    }
                }
            }

            // Expose the generated struct through a function
            pub fn [<$callback _ temp_struct>]() -> [<_private_ $callback _ TempStruct>]::[<$callback _ TempStruct>] {
                [<_private_ $callback _ TempStruct>]::[<$callback _ TempStruct>]
            }

            // Call the macro to set the traceback callback
            $crate::set_traceback_callback($crate::TracebackCallbackType::Sync(Box::new([<$callback _ temp_struct>]())));
        }
    };
    (async $callback:ident) => {
        $crate::paste::unique_paste! {
            // Generate a unique identifier for the struct
            #[allow(non_camel_case_types)]
            mod [<_private_ $callback _ TempStruct>] {
                pub struct [<$callback _ TempStruct>];

                impl $crate::TracebackCallbackAsync for [<$callback _ TempStruct>] {
                    fn call(
                        &self,
                        error: $crate::error_types::TracebackError,
                    ) -> std::pin::Pin<
                        Box<dyn std::future::Future<Output = ()> + std::marker::Send + std::marker::Sync>,
                    > {
                        Box::pin(super::$callback(error))
                    }
                }
            }

            // Expose the generated struct through a function
            pub fn [<$callback _ temp_struct>]() -> [<_private_ $callback _ TempStruct>]::[<$callback _ TempStruct>] {
                [<_private_ $callback _ TempStruct>]::[<$callback _ TempStruct>]
            }

            // Call the macro to set the traceback callback
            $crate::set_traceback_callback($crate::TracebackCallbackType::Async(Box::new([<$callback _ temp_struct>]())));
        }
    };
}
