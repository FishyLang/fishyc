use std::process::Command;

#[test]
fn custom_ir_optimization_preserves_observable_results() {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut command = Command::new("python3");
    command
        .arg(format!("{root}/tests/custom_ir/run.py"))
        .args(["--binary", env!("CARGO_BIN_EXE_fishyc")]);
    if let Some(output) = std::env::var_os("FISHYC_IR_TEST_OUTPUT") {
        command.arg("--output").arg(output);
    }
    let status = command
        .status()
        .expect("Python 3 and a C linker are required");
    assert!(status.success(), "custom IR regression suite failed");
}
