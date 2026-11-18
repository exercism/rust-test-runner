//! Writes the list of file ranges for `prefetch::start` to load at the start
//! of each run. It runs once, at image build.
//!
//! usage: record_prefetch <output> <dir>... -- <command> [args]...
//!
//! It removes every file under the given directories from the page cache,
//! runs the command (the test runner on sample exercises), and then writes
//! down which pages of those files the command loaded. If the files can't be
//! removed from the cache, the list would include whole files that the runner
//! never reads, so it writes an empty list instead.

use std::fs::File;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// If more than this is still cached after the eviction, it didn't work.
const MAX_LEFT_AFTER_EVICTION: usize = 16 << 20;

/// If the command loaded more than this, something other than the runner's
/// own reads is in the list.
const MAX_TOTAL: usize = 512 << 20;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(split) = args.iter().position(|a| a == "--") else {
        bail!("usage: record_prefetch <output> <dir>... -- <command> [args]...");
    };
    let (output, dirs) = args[..split].split_first().context("missing output path")?;
    let command = &args[split + 1..];
    let files: Vec<PathBuf> = dirs
        .iter()
        .flat_map(|d| regular_files(Path::new(d)))
        .collect();

    unsafe { libc::sync() };
    for file in &files {
        evict(file);
    }
    let left: usize = files.iter().map(|f| cached_bytes(f)).sum();

    let status = Command::new(&command[0])
        .args(&command[1..])
        .status()
        .context("failed to run the training command")?;
    if !status.success() {
        bail!("the training command failed: {status}");
    }

    let mut list = String::new();
    let mut total = 0;
    for file in &files {
        for (offset, len) in cached_ranges(file) {
            list.push_str(&format!("{} {offset} {len}\n", file.display()));
            total += len;
        }
    }
    if left > MAX_LEFT_AFTER_EVICTION || total > MAX_TOTAL {
        eprintln!(
            "record_prefetch: {} MB stayed cached and {} MB were loaded, so the list would be wrong; writing an empty list",
            left >> 20,
            total >> 20
        );
        list.clear();
    } else {
        eprintln!(
            "record_prefetch: {} MB in {} ranges",
            total >> 20,
            list.lines().count()
        );
    }
    File::create(output)?.write_all(list.as_bytes())?;
    Ok(())
}

fn regular_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                stack.push(entry.path());
            } else if kind.is_file() {
                out.push(entry.path());
            }
        }
    }
    out
}

fn evict(path: &Path) {
    if let Ok(file) = File::open(path) {
        unsafe { libc::posix_fadvise(file.as_raw_fd(), 0, 0, libc::POSIX_FADV_DONTNEED) };
    }
}

fn cached_bytes(path: &Path) -> usize {
    cached_ranges(path).iter().map(|(_, len)| len).sum()
}

/// Returns (offset, length) for each run of the file's pages that is in the
/// page cache.
fn cached_ranges(path: &Path) -> Vec<(usize, usize)> {
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };
    let Ok(size) = file.metadata().map(|m| m.len() as usize) else {
        return Vec::new();
    };
    if size == 0 {
        return Vec::new();
    }
    let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
    let pages = size.div_ceil(page);
    let mut resident = vec![0u8; pages];
    unsafe {
        let addr = libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ,
            libc::MAP_SHARED,
            file.as_raw_fd(),
            0,
        );
        if addr == libc::MAP_FAILED {
            return Vec::new();
        }
        let ok = libc::mincore(addr, size, resident.as_mut_ptr()) == 0;
        libc::munmap(addr, size);
        if !ok {
            return Vec::new();
        }
    }
    let mut ranges = Vec::new();
    let mut i = 0;
    while i < pages {
        if resident[i] & 1 == 0 {
            i += 1;
            continue;
        }
        let start = i;
        while i < pages && resident[i] & 1 == 1 {
            i += 1;
        }
        let (from, to) = (start * page, (i * page).min(size));
        ranges.push((from, to - from));
    }
    ranges
}
