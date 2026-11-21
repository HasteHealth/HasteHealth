//! Fails when a committed `.min.json` no longer matches what its upstream
//! source, patches and rules produce, e.g. after editing a patch without
//! running `cargo run artifacts build <manifest>`.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn manifests() -> Vec<PathBuf> {
    let artifacts = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../artifacts");
    WalkDir::new(artifacts)
        .into_iter()
        .filter_entry(|e| e.file_name() != "node_modules")
        .filter_map(Result::ok)
        .map(walkdir::DirEntry::into_path)
        .filter(|p| p.ends_with("patches/manifest.toml"))
        .collect()
}

#[test]
fn outputs_are_up_to_date() {
    let manifests = manifests();
    assert!(!manifests.is_empty(), "no patch manifests found");

    for manifest in manifests {
        let build = haste_artifact_patcher::build(&manifest)
            .unwrap_or_else(|e| panic!("{}: {e}", manifest.display()));
        for output in build.outputs {
            let committed = std::fs::read_to_string(&output.path).unwrap_or_default();
            let committed: serde_json::Value = serde_json::from_str(&committed).unwrap_or_default();
            let expected: serde_json::Value = serde_json::from_str(&output.contents).unwrap();
            assert!(
                committed == expected,
                "{} is out of date; from backend/ run `cargo run artifacts build {}`",
                output.path.display(),
                manifest.display()
            );
        }
    }
}
