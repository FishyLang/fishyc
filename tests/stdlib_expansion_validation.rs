use std::process::Command;

#[test]
fn monolithic_synchronous_stdlib_core_and_output() {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut command = Command::new("python3");
    command.arg(format!("{root}/tests/stdlib_expansion/run.py"))
        .args(["--binary", env!("CARGO_BIN_EXE_fishyc")]);
    if let Some(output) = std::env::var_os("FISHYC_EXPANSION_TEST_OUTPUT") {
        command.arg("--output").arg(output);
    }
    let status = command.status().expect("Python 3, LLVM 17 and a system linker are required");
    assert!(status.success(), "stdlib expansion regression failed");
}
