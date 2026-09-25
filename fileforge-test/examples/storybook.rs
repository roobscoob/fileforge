//! Prints every story in colour: `cargo run -p fileforge-test --example storybook`.

// Stories are registered by the crates that declare them, so every crate must be linked.
extern crate fileforge_nintendo;
extern crate fileforge_std;

fn main() {
  fileforge::storybook::invoke();
}
