//! Reusing crates compiled at image build.
//!
//! Without this, every run of an exercise with dependencies (such as `time`
//! for gigasecond or `pretty_assertions` for pov) compiles those crates from
//! source, which takes one to six seconds. `bin/build-dep-cache.sh` compiles
//! the crates used by the track's exercises at image build. Before each run,
//! `link` puts a symbolic link to each of those files in the solution's
//! target directory. Cargo finds the crates already built and only compiles
//! the solution and its tests.
//!
//! Cargo names each compiled crate with a hash of its version, features and
//! compiler flags. If a solution asks for a crate with other features, or for
//! a crate that isn't in the cache, cargo doesn't find a match and compiles
//! it, the same as without the cache.

use std::path::Path;

/// The target directory filled at image build.
const CACHE_DIR: &str = "/opt/test-runner/dep-cache";

/// The folders of a cargo target directory that hold compiled dependencies:
/// the libraries, cargo's records of how each was built, and build script
/// outputs.
const FOLDERS: [&str; 3] = ["deps", ".fingerprint", "build"];

/// Links the dev profile's compiled crates from the cache into `target`, if
/// the manifest has any dependencies and `target` doesn't exist yet. Errors
/// are ignored, because the run works the same without the cache, only
/// slower.
pub fn link(manifest: &str, target: &Path) {
    if !manifest.contains("dependencies") || target.exists() {
        return;
    }
    for folder in FOLDERS {
        let from = Path::new(CACHE_DIR).join("debug").join(folder);
        let to = target.join("debug").join(folder);
        let Ok(entries) = std::fs::read_dir(&from) else {
            continue;
        };
        if std::fs::create_dir_all(&to).is_err() {
            continue;
        }
        for entry in entries.flatten() {
            let _ = std::os::unix::fs::symlink(entry.path(), to.join(entry.file_name()));
        }
    }
}
