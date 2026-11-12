use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");

    let package_version =
        std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    let version = std::env::var("GITHUB_REF_NAME")
        .ok()
        .filter(|value| value.starts_with('v'))
        .or_else(|| git_value(&["describe", "--tags", "--exact-match", "HEAD"]))
        .unwrap_or(package_version);
    let commit = std::env::var("GITHUB_SHA")
        .ok()
        .map(|value| value.chars().take(12).collect())
        .or_else(|| git_value(&["rev-parse", "--short=12", "HEAD"]))
        .unwrap_or_else(|| "unknown-commit".to_string());

    println!("cargo:rustc-env=PROJECT_TWN_BUILD_VERSION={version}");
    println!("cargo:rustc-env=PROJECT_TWN_BUILD_COMMIT={commit}");
}

fn git_value(arguments: &[&str]) -> Option<String> {
    Command::new("git")
        .args(arguments)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
}
