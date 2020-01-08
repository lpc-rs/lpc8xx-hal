#![cfg(feature = "compiletest")]

extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

#[test]
fn compile_test() {
    run_mode("compile-fail");
}

fn run_mode(mode: &'static str) {
    let mut config = compiletest::Config::default();

    config.mode = mode.parse().expect("Failed to parse mode");
    config.src_base = PathBuf::from(format!("tests/{}", mode));

    // Needed by the compiler to find other crates
    config.link_deps();

    // Fixes E0464, "multiple matching crates"
    config.clean_rmeta();

    compiletest::run_tests(&config);
}
