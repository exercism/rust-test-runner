use std::{fs, path::PathBuf};

use cmd_lib::run_cmd;
use toml::Value;

#[test]
fn track_deps_are_up_to_date() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent().unwrap();
    let target_dir = repo_root.join("target");
    fs::create_dir_all(&target_dir).unwrap();

    let track_repo_path = target_dir.join("rust-main");

    if !track_repo_path.exists() {
        run_cmd!(
            curl -L "https://github.com/exercism/rust/archive/refs/heads/main.zip" > $track_repo_path.zip;
            unzip -u $track_repo_path.zip -d $target_dir;
        )
        .unwrap();
    }

    let available_deps = {
        let manifest = fs::read_to_string(repo_root.join("local-registry/Cargo.toml")).unwrap();
        let manifest: Value = toml::from_str(&manifest).unwrap();
        manifest["dependencies"].as_table().unwrap().clone()
    };

    let exercise_manifests = glob::glob(&format!(
        "{}/exercises/*/*/Cargo.toml",
        track_repo_path.display(),
    ))
    .unwrap();

    for path in exercise_manifests.map(Result::unwrap) {
        let exercise = path.parent().unwrap().file_name().unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let manifest: Value = toml::from_str(&content).unwrap();
        let deps = manifest["dependencies"].as_table().unwrap();

        for (name, version) in deps {
            let (major, minor) = parse_semver(version.as_str().unwrap());

            if available_deps
                .iter()
                .filter_map(|(k, v)| {
                    if k == name {
                        return Some(v);
                    }
                    if k.starts_with(name)
                        && let Some(table) = v.as_table()
                    {
                        return table.get("version");
                    }
                    None
                })
                .map(|v| parse_semver(v.as_str().unwrap()))
                .any(|(av_major, av_minor)| {
                    major != 0 && major == av_major || major == 0 && minor == av_minor
                })
            {
                continue;
            } else if major != 0 {
                panic!("{exercise:?} depends on {name} v{major}, which is not available")
            } else {
                panic!("{exercise:?} depends on {name} v0.{minor}, which is not available")
            }
        }
    }
}

fn parse_semver(version: &str) -> (u32, u32) {
    let mut iter = version.split('.');
    let major = iter.next().unwrap().parse().unwrap();
    let minor = iter.next().unwrap().parse().unwrap();
    // don't care about patch
    (major, minor)
}
