/*
 * Copyright (c) 2024 shadow3aaa@gitbub.com
 *
 * This file is part of frame-analyzer-ebpf.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::io;

use aya::{Ebpf, EbpfLoader, include_bytes_aligned};
use ctor::ctor;

use crate::error::Result;

#[ctor]
fn ebpf_workround() {
    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
}

pub fn load_bpf() -> Result<Ebpf> {
    // This will include eBPF object file as raw bytes at compile-time and load it at runtime.
    #[cfg(debug_assertions)]
    let data = include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/ebpf_target/bpfel-unknown-none/debug/frame-analyzer-ebpf"
    ));
    #[cfg(not(debug_assertions))]
    let data = include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/ebpf_target/bpfel-unknown-none/release/frame-analyzer-ebpf"
    ));

    // BPF ring buffers must be a power-of-two multiple of the kernel page
    // size. The embedded program declares 4 KiB, which fails on 16 KiB kernels.
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    let size = ring_buffer_size(page_size)?;
    Ok(EbpfLoader::new()
        .set_max_entries("RING_BUF", size)
        .load(data)?)
}

fn ring_buffer_size(page_size: libc::c_long) -> io::Result<u32> {
    let size = u32::try_from(page_size)
        .ok()
        .filter(|size| size.is_power_of_two())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid kernel page size"))?;
    Ok(size.max(4096))
}

#[cfg(test)]
mod tests {
    use super::ring_buffer_size;

    #[test]
    fn ring_buffer_matches_kernel_page_size() {
        for size in [4096, 16384, 65536] {
            assert_eq!(ring_buffer_size(size).unwrap(), size as u32);
        }
    }

    #[test]
    fn invalid_page_size_is_rejected() {
        for size in [-1, 0, 6144] {
            assert!(ring_buffer_size(size).is_err());
        }
    }
}
