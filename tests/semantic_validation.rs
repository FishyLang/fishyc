use std::process::Command;

#[test]
fn semantic_type_regressions_stop_before_lowering() {
    let root = env!("CARGO_MANIFEST_DIR");
    let status = Command::new("python3")
        .arg(format!("{root}/tests/semantic/run.py"))
        .args(["--binary", env!("CARGO_BIN_EXE_fishyc")])
        .status()
        .expect("Python 3 is required for semantic regression fixtures");
    assert!(status.success(), "semantic/type regression suite failed");
}
