use std::process::Command;

#[test]
fn current_arc_lifetimes_match_values_and_allocation_accounting() {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut command = Command::new("python3");
    command
        .arg(format!("{root}/tests/ownership/run.py"))
        .args(["--binary", env!("CARGO_BIN_EXE_fishyc")]);
    if let Some(output) = std::env::var_os("FISHYC_OWNERSHIP_TEST_OUTPUT") {
        command.arg("--output").arg(output);
    }
    let status = command
        .status()
        .expect("Python 3, LLVM 17 opt/llc and a C linker are required");
    assert!(status.success(), "ownership/ARC regression suite failed");
}
