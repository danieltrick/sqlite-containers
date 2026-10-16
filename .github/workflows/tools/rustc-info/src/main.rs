use std::{
    env::consts,
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    let output = Command::new(env!["X_CARGO_PATH"])
        .arg("rustc")
        .arg("--locked")
        .arg("--quiet")
        .arg("--")
        .arg("-vV")
        .output()
        .expect("Failed to execute CARGO!");

    for line in String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|str| str.trim_ascii())
        .filter(|str| !str.is_empty())
    {
        if let Some(rust_version) = line
            .strip_prefix("rustc")
            .map(|str| str.trim_ascii())
            .filter(|str| !str.is_empty())
        {
            println!("Rust Version: {}", rust_version);
            println!("Target: {}/{}", consts::OS, consts::ARCH);
            return ExitCode::SUCCESS;
        }
    }

    panic!("Failed to determine `rustc` version!")
}
