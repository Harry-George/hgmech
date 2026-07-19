#[cfg(not(target_arch = "wasm32"))]
mod args;
#[cfg(not(target_arch = "wasm32"))]
mod state;
#[cfg(not(target_arch = "wasm32"))]
mod topology;
#[cfg(not(target_arch = "wasm32"))]
mod run;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    crate::run::run().await;
}

#[cfg(target_arch = "wasm32")]
fn main() {}
