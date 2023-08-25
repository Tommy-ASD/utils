use tokio::runtime::Runtime;

// Function to run an asynchronous task synchronously using a shared Tokio runtime
pub fn sync_execute<F, R>(task: F) -> R
where
    F: std::future::Future<Output = R>,
{
    let rt = Runtime::new().unwrap();
    rt.block_on(task)
}

#[macro_export]
macro_rules! sync_if_no_runtime {
    ($e:expr) => {{
        // Check if a runtime is already available
        if let Ok(_) = Handle::try_current() {
            println!("Runtime already available");
            // Spawn the task if a runtime is available
            task::spawn(async move { $e.await });
        } else {
            println!("No runtime available");
            // Run synchronously if no runtime is available
            crate::async_utils::sync_execute($e)
        }
    }};
}
