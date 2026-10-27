use std::process::Command;

#[test]
fn stable_string_preserves_values_and_ownership() {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut command = Command::new("python3");
    command
        .arg(format!("{root}/tests/managed_string/run.py"))
        .args(["--binary", env!("CARGO_BIN_EXE_fishyc")]);
    if let Some(output) = std::env::var_os("FISHYC_MANAGED_STRING_TEST_OUTPUT") {
        command.arg("--output").arg(output);
    }
    let status = command.status().expect("Python 3, LLVM 17 and a C linker are required");
    assert!(status.success(), "stable String acceptance failed");
}

#[test]
fn canonical_prelude_shares_one_empty() {
    let root = env!("CARGO_MANIFEST_DIR");
    for custom in [false, true] {
        let mut command = Command::new("python3");
        command
            .arg(format!("{root}/tests/managed_string/profile.py"))
            .args(["--binary", env!("CARGO_BIN_EXE_fishyc"), "--source"])
            .arg(format!("{root}/tests/managed_string/prelude_profile.fih"));
        if custom {
            command.arg("--custom-opt");
        }
        if let Some(output) = std::env::var_os("FISHYC_MANAGED_STRING_TEST_OUTPUT") {
            command.arg("--output").arg(std::path::PathBuf::from(output)
                .join("prelude-profile").join(if custom { "custom-O" } else { "plain" }));
        }
        let status = command.status().expect("Python 3 and a C linker are required");
        assert!(status.success(), "prelude identity/allocation contract failed");
    }
}
