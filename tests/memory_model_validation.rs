use std::process::Command;

#[test]
fn ownership_reference_and_resource_contracts() {
    let root=env!("CARGO_MANIFEST_DIR");
    for optimize in [false,true] {
        let mut command=Command::new("python3");
        command.arg(format!("{root}/tests/stdlib/run.py"))
            .args(["--binary",env!("CARGO_BIN_EXE_fishyc"),"--manifest"])
            .arg(format!("{root}/tests/memory_model/cases.json"));
        if optimize {command.arg("--custom-opt");}
        if let Some(output)=std::env::var_os("FISHYC_MEMORY_MODEL_OUTPUT") {
            command.arg("--output").arg(std::path::PathBuf::from(output).join(if optimize {"custom-O"}else{"plain"}));
        }
        assert!(command.status().expect("Python 3, C linker and LLVM verification tools are required").success(),"ownership contract suite failed (custom optimizer: {optimize})");
    }
}

#[test]
fn deterministic_destruction_obeys_source_order_contract() {
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output=std::env::var_os("FISHYC_MEMORY_MODEL_OUTPUT").map(std::path::PathBuf::from)
        .unwrap_or_else(||std::env::temp_dir().join(format!("fishy-drop-order-{}",std::process::id()))).join("drop-order");
    let status=Command::new("python3").arg(root.join("tests/memory_model/drop_order.py"))
        .args(["--binary",env!("CARGO_BIN_EXE_fishyc"),"--output"]).arg(output)
        .status().expect("LLVM tools and a C linker are required for destruction-order checks");
    assert!(status.success(),"deterministic destruction-order regression");
}
