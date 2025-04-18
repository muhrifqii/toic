mod repositories;
mod services;
mod structure;
mod types;
mod utils;

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
