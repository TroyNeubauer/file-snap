use std::env;

pub fn main() {
    if Ok("debug".to_owned()) != env::var("PROFILE") {
        // Compile UI and embed into application
    } else {
        // Nop, we dynamically re-compile it at runtime for debugging
    }
}
