use std::env;
use std::process::Command;

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");

    emit("FUSION_BUILD_GIT_TAG", git_tag());
    emit("FUSION_BUILD_GIT_COMMIT", git_commit());
    emit("FUSION_BUILD_TIMESTAMP", build_timestamp());
    emit("FUSION_BUILD_RUST_VERSION", rust_version());
}

fn emit(key: &str, value: String) {
    println!("cargo:rustc-env={}={}", key, value);
}

fn git_tag() -> String {
    git(&["describe", "--tags", "--abbrev=0"]) // prefer annotated tags
        .or_else(|| git(&["describe", "--tags"]))
        .unwrap_or_else(|| env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string()))
}

fn git_commit() -> String {
    git(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string())
}

fn build_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string())
}

fn rust_version() -> String {
    run("rustc", &["--version"]).unwrap_or_else(|| "unknown".to_string())
}

fn git(args: &[&str]) -> Option<String> {
    run("git", args)
}

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    Some(stdout.trim().to_string())
}
