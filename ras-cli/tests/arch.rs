use std::fs;

#[test]
fn domain_modules_do_not_import_third_party_sdks() {
    let banned = [
        "use chromiumoxide",
        "use reqwest",
        "use keyring",
        "use security_framework",
        "use ffmpeg_next",
    ];
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");

    let mut violations = Vec::new();
    for entry in walkdir::WalkDir::new(workspace_root)
        .max_depth(6)
        .into_iter()
        .filter_map(Result::ok)
    {
        let p = entry.path();
        if !p.extension().is_some_and(|e| e == "rs") {
            continue;
        }
        let s = p.to_string_lossy();
        if !s.contains("/src/domain/") && !s.contains("/src/application/") {
            continue;
        }
        if s.contains("/target/") {
            continue;
        }
        let Ok(text) = fs::read_to_string(p) else {
            continue;
        };
        for needle in banned {
            if text.contains(needle) {
                violations.push(format!("{} imports {}", p.display(), needle));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "layer violations:\n{}",
        violations.join("\n")
    );
}
