use std::{
    io::{self, stderr, stdout, ErrorKind, Write},
    process::Command,
};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=../client/index.html");
    println!("cargo:rerun-if-changed=../client/Cargo.toml");
    println!("cargo:rerun-if-changed=../client/manifest.json");
    println!("cargo:rerun-if-changed=../client/Trunk.toml");
    println!("cargo:rerun-if-changed=../client/src");
    println!("cargo:rerun-if-changed=../client/styles");
    println!("cargo:rerun-if-changed=../client/img");

    let profile = std::env::var("PROFILE").unwrap();
    let run_trunk = match profile.as_str() {
        "debug" => false,
        "release" => true,
        _ => false,
    };

    if run_trunk {
        let output = Command::new("trunk")
            .current_dir("../client")
            .args(["build", "--release"])
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            stdout().write_all(&output.stdout)?;
            stderr().write_all(&output.stderr)?;
            Err(io::Error::new(ErrorKind::Other, "trunk failed to run"))
        }
    } else {
        Ok(())
    }
}
