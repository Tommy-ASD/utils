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

/// Sets a custom traceback callback for error handling in a Rust program.
///
/// This macro allows you to define and set a custom traceback callback function,
/// which will be called when a TracebackError goes out of scope.
/// The traceback callback provides a way to customize how error information is
/// handled and reported.
///
/// # Usage
///
/// To use this macro, provide the name of the callback function you want to use
/// as the custom traceback callback. This function should take an argument of
/// type `utils::error_types::TracebackError`. The macro generates a unique
/// struct and function to wrap your callback and sets it as the traceback
/// callback using `utils::set_traceback_callback`.
///
/// # Example
///
/// ```rust
/// // Define a custom traceback callback function
/// fn my_traceback_callback(error: utils::error_types::TracebackError) {
///     // Custom error handling logic here
///     println!("Custom traceback callback called: {:?}", error);
/// }
///
/// // Use the set_traceback macro to set the custom traceback callback
/// set_traceback!(my_traceback_callback);
///
/// // Any TracebackErrors will now be handled by my_traceback_callback when dropped
/// ```
///
/// ```rust
/// // The same is possible with asynchronous functions
/// async fn my_traceback_callback(error: utils::error_types::TracebackError) {
///     // Custom error handling logic here
///     println!("Async custom traceback callback called: {:?}", error);
/// }
///
/// // But you have to specify that it is asynchronous
/// set_traceback!(async my_traceback_callback);
///
/// // Any TracebackErrors will now be handled by my_traceback_callback when dropped
/// ```
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

/// Sets a custom panic handler for handling panics in Rust programs.
///
/// This macro allows you to easily set a custom panic handler function using the
/// `std::panic::set_hook` function. A panic handler is a function that is called
/// when a panic occurs in your program, allowing you to customize how panics are
/// handled.
///
/// # Usage
///
/// To use this macro, provide the name of the function you want to use as the
/// custom panic handler. This function should have the following signature:
///
/// ```rust
/// fn my_panic_handler(info: &std::panic::PanicInfo);
/// ```
///
/// # Example
///
/// ```rust
/// // Define a custom panic handler function
/// fn my_panic_handler(info: &std::panic::PanicInfo) {
///     println!("Custom panic handler called: {:?}", info);
///     // You can add your custom panic handling logic here
/// }
///
/// // Use the set_panic_handler macro to set the custom panic handler
/// set_panic_handler!(my_panic_handler);
///
/// // Any panics that occur in the program will now be handled by my_panic_handler.
/// ```
///
/// For more information on panic handling in Rust, see the Rust documentation on
/// panic handling: https://doc.rust-lang.org/std/panic/index.html
#[macro_export]
macro_rules! set_panic_handler {
    ($handler:ident) => {
        std::panic::set_hook(Box::new($handler))
    };
}
