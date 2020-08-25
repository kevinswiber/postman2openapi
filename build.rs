extern crate chrono;

use chrono::prelude::*;
use std::env;
use std::fs::{self, File};
use std::path::Path;
use std::process;

fn main() {
    // OUT_DIR is set by Cargo and it's where any additional build artifacts
    // are written.
    let outdir = match env::var_os("OUT_DIR") {
        Some(outdir) => outdir,
        None => {
            eprintln!(
                "OUT_DIR environment variable not defined. \
                 Please file a bug: \
                 https://github.com/kevinswiber/postman2openapi/issues/new"
            );
            process::exit(1);
        }
    };
    fs::create_dir_all(&outdir).unwrap();

    let stamp_path = Path::new(&outdir).join("postman2openapi-stamp");
    if let Err(err) = File::create(&stamp_path) {
        panic!("failed to write {}: {}", stamp_path.display(), err);
    }
    // Make the current git hash available to the build.
    if let Some(rev) = git_revision_hash() {
        println!("cargo:rustc-env=POSTMAN2OPENAPI_BUILD_GIT_HASH={}", rev);
    }
    if let Some(branch) = git_branch() {
        println!(
            "cargo:rustc-env=POSTMAN2OPENAPI_BUILD_GIT_BRANCH={}",
            branch
        );
    }
    println!(
        "cargo:rustc-env=POSTMAN2OPENAPI_BUILD_DATE={}",
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
    );
}

fn git_revision_hash() -> Option<String> {
    let result = process::Command::new("git")
        .args(&["rev-parse", "--short=10", "HEAD"])
        .output();
    result.ok().and_then(|output| {
        let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    })
}

fn git_branch() -> Option<String> {
    let result = process::Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output();
    result.ok().and_then(|output| {
        let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    })
}
