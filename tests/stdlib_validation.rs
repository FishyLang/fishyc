use std::process::Command;

#[test]
fn current_stdlib_resource_contracts() {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut command = Command::new("python3");
    command
        .arg(format!("{root}/tests/stdlib/run.py"))
        .args(["--binary", env!("CARGO_BIN_EXE_fishyc")]);
    if let Some(output) = std::env::var_os("FISHYC_STDLIB_TEST_OUTPUT") {
        command.arg("--output").arg(output);
    }
    let status = command
        .status()
        .expect("Python 3, a C linker and LLVM 17 for explicitly marked cases are required");
    assert!(status.success(), "stdlib compatibility regression suite failed");
}
