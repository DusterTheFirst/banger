use std::{
    env,
    fs::DirBuilder,
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

    let pkg_name = env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME environment variable not set");

    DirBuilder::new()
        .recursive(true)
        .create("../client/dist")
        .expect("failed to create trunk dist directory");

    if env::var("SKIP_TRUNK_BUILD").is_ok() {
        println!("cargo:warning={pkg_name}(build.rs) trunk build has been skipped through the use of SKIP_TRUNK_BUILD");

        return ExitCode::SUCCESS;
    }

    if env::var("PROFILE").expect("PROFILE environment variable not set") == "debug" {
        println!(
            "cargo:warning={pkg_name}(build.rs) trunk build has been skipped due to debug build"
        );

        return ExitCode::SUCCESS;
    }

    let output = Command::new(env::var("TRUNK_PATH").unwrap_or_else(|_error| "trunk".to_string()))
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
}
