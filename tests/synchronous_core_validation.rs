use std::process::Command;

#[test]
fn synchronous_core_has_checked_arc_without_a_native_runtime() {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut command = Command::new("python3");
    command.arg(format!("{root}/tests/synchronous_core/run.py"))
        .args(["--binary", env!("CARGO_BIN_EXE_fishyc")]);
    if let Some(output) = std::env::var_os("FISHYC_SYNC_TEST_OUTPUT") {
        command.arg("--output").arg(output);
    }
    assert!(command.status().expect("Python 3, LLVM 17 and a system linker are required").success());
}
