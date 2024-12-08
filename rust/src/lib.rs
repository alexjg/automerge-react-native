uniffi::setup_scaffolding!();

#[uniffi::export]
fn greet() -> String {
  String::from("Hello, world!")
}
