//! Loading the toolchain from disk in the background.
//!
//! When the toolchain's files are not in the page cache, which happens on the
//! first Rust run on a machine or after other images have pushed them out,
//! cargo and rustc load about 200 MB of the image through thousands of small
//! page faults, one after another. On a busy production machine, that can take
//! longer than the whole 15 second limit for the tests.
//!
//! At image build, `record_prefetch` runs the test runner on sample exercises
//! and writes down which parts of which files it read. At the start of each
//! run, `start` reads those parts in background threads, in large requests and
//! several at a time. By the time cargo and rustc need a page, it is usually
//! already in memory. Before reading a part, `start` asks the kernel which of
//! its pages are already in memory, and skips the parts that are. When the
//! toolchain is already cached, a run then copies nothing.

use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::fs::FileExt;
use std::path::Path;
use std::thread;

/// The list written at image build: one `path offset length` line per range.
pub const LIST_PATH: &str = "/opt/test-runner/prefetch.txt";

/// How many threads read at the same time. Several threads keep several disk
/// requests in flight, which is faster on EBS than one request at a time.
/// Each thread waits for its own reads, so cargo's and rustc's own reads
/// don't queue behind the whole list.
const THREADS: usize = 4;

/// Each read asks for up to 1 MB, so the kernel reads from disk in large
/// requests.
const CHUNK: usize = 1 << 20;

/// Starts loading the ranges in `list` in background threads, and returns at
/// once. The threads are never waited for. If the list is missing or a file in
/// it can't be opened, that part is skipped, because the run works the same
/// without it, only slower when the cache is cold.
pub fn start(list: &Path) {
    let Ok(content) = std::fs::read_to_string(list) else {
        return;
    };
    let ranges: Vec<(String, u64, u64)> = content
        .lines()
        .filter_map(|line| {
            let mut parts = line.rsplitn(3, ' ');
            let len = parts.next()?.parse().ok()?;
            let offset = parts.next()?.parse().ok()?;
            Some((parts.next()?.to_string(), offset, len))
        })
        .collect();
    let ranges = std::sync::Arc::new(ranges);
    for worker in 0..THREADS {
        let ranges = ranges.clone();
        thread::spawn(move || {
            let mut buf = vec![0u8; CHUNK];
            for (path, offset, len) in ranges.iter().skip(worker).step_by(THREADS) {
                let Ok(file) = File::open(path) else { continue };
                let (mut at, end) = (*offset, offset + len);
                while at < end {
                    let want = CHUNK.min((end - at) as usize);
                    if is_cached(&file, at, want) {
                        at += want as u64;
                        continue;
                    }
                    match file.read_at(&mut buf[..want], at) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => at += n as u64,
                    }
                }
            }
        });
    }
}

/// Whether every page of `len` bytes at `offset` in `file` is already in the
/// page cache. Mapping the file and asking `mincore` doesn't read anything
/// from disk, so this costs far less than reading pages that are already
/// cached. `offset` must be a multiple of the page size, as every offset in
/// the list is. If the check fails, the part counts as not cached, so it is
/// read.
fn is_cached(file: &File, offset: u64, len: usize) -> bool {
    let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
    let mut resident = vec![0u8; len.div_ceil(page)];
    unsafe {
        let addr = libc::mmap(
            std::ptr::null_mut(),
            len,
            libc::PROT_READ,
            libc::MAP_SHARED,
            file.as_raw_fd(),
            offset as libc::off_t,
        );
        if addr == libc::MAP_FAILED {
            return false;
        }
        let ok = libc::mincore(addr, len, resident.as_mut_ptr()) == 0;
        libc::munmap(addr, len);
        ok && resident.iter().all(|r| r & 1 == 1)
    }
}
