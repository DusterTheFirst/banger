use std::{
    io::{stderr, stdout, Write},
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    println!("cargo:rerun-if-changed=../client/index.html");
    println!("cargo:rerun-if-changed=../client/Cargo.toml");
    println!("cargo:rerun-if-changed=../client/manifest.json");
    println!("cargo:rerun-if-changed=../client/Trunk.toml");
    println!("cargo:rerun-if-changed=../client/src");
    println!("cargo:rerun-if-changed=../client/styles");
    println!("cargo:rerun-if-changed=../client/img");

    let profile = std::env::var("PROFILE").expect("no PROFILE environment variable");
    let run_trunk = match profile.as_str() {
        "debug" => false,
        "release" => true,
        _ => false,
    };

    if run_trunk {
        let output =
            Command::new(std::env::var("TRUNK_PATH").unwrap_or_else(|_error| "trunk".to_string()))
                .current_dir("../client")
                .args(["build", "--release"])
                .output()
                .expect("failed to run trunk");

        if output.status.success() {
            ExitCode::SUCCESS
        } else {
            stdout()
                .write_all(&output.stdout)
                .expect("failed to write to stdout");
            stderr()
                .write_all(&output.stderr)
                .expect("failed to write to stderr");
            eprintln!("trunk failed to run");
            ExitCode::FAILURE
        }
    } else {
        ExitCode::SUCCESS
    }
}
