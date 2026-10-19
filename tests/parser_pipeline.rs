use std::process::Command;

#[test]
fn parser_cases_verify_and_execute_with_and_without_optimization() {
    let status = Command::new("python3")
        .arg(format!(
            "{}/tests/run_parser_pipeline.py",
            env!("CARGO_MANIFEST_DIR")
        ))
        .args(["--binary", env!("CARGO_BIN_EXE_fishyc"), "--assert-results"])
        .status()
        .expect("Python 3, LLVM 17 opt/llc, and a C linker are required");
    assert!(status.success(), "parser pipeline regression suite failed");
}
