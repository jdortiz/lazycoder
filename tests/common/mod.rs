//! Helpers to run the lazycoder binary isolated from the configuration of the user.
#![allow(dead_code)] // Each test file uses a subset of them.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use tempfile::TempDir;

/// Temporary directory used as the home directory of lazycoder, so its configuration is written there instead of in
/// the one of the user. Snippet files are written there too, and commands run in it.
pub struct Sandbox {
    home: TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        Sandbox {
            home: TempDir::new().expect("Failed to create temporary directory"),
        }
    }

    pub fn path(&self) -> &Path {
        self.home.path()
    }

    /// Write a file in the sandbox and return its path.
    pub fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.path().join(name);
        fs::write(&path, contents).expect("Failed to write file");
        path
    }

    /// Run lazycoder with the given arguments.
    pub fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_lazycoder"))
            .args(args)
            .current_dir(self.path())
            .env("HOME", self.path())
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("RUST_LOG")
            .output()
            .expect("Failed to run lazycoder")
    }

    /// Run lazycoder expecting it to succeed, and return its standard output.
    pub fn succeed(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "lazycoder {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("Output isn't UTF-8")
    }

    /// Run lazycoder expecting it to fail, and return its standard error.
    pub fn fail(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            !output.status.success(),
            "lazycoder {args:?} succeeded with output: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        String::from_utf8_lossy(&output.stderr).into_owned()
    }
}
