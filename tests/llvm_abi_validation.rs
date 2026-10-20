use std::process::Command;

#[test]
fn llvm_abi_and_optimization_preserve_observable_results() {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut command = Command::new("python3");
    command
        .arg(format!("{root}/tests/llvm_abi/run.py"))
        .args(["--binary", env!("CARGO_BIN_EXE_fishyc")]);
    if let Some(output) = std::env::var_os("FISHYC_LLVM_TEST_OUTPUT") {
        command.arg("--output").arg(output);
    }
    let status = command
        .status()
        .expect("Python 3, LLVM 17 opt/llc and a C linker are required");
    assert!(status.success(), "LLVM ABI regression suite failed");
}
