use std::{fs, path::PathBuf};

mod github;
mod output;
mod unicode;
mod util;

fn main() {
    let unicode_data = unicode::build().unwrap();

    let generated_code = output::generate_rust_code(&unicode_data);

    let out_dir = PathBuf::from("crates/emojeez/src/lib.rs");
    fs::write(out_dir, generated_code).unwrap();
}
