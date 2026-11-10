#[path = "../../build_support/implementation.rs"]
mod implementation;

fn main() {
    implementation::emit("../../Cargo.lock");
}
