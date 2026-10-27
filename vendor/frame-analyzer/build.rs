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
use std::{env, fs, path::PathBuf, process::Command};

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PATH");
    let status = Command::new("bpf-linker")
        .arg("--version")
        .status()
        .context("Install bpf-linker and add it to PATH before building (see README.md)")?;
    ensure!(status.success(), "bpf-linker --version failed: {status}");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let target_dir = out_dir.join("ebpf_target");
    let mut command = Command::new("cargo");
    command.args([
        "+nightly",
        "install",
        "frame-analyzer-ebpf",
        "--version",
        "0.2.0",
        "--locked",
        "--force",
        "--target",
        "bpfel-unknown-none",
        "-Z",
        "build-std=core",
    ]);
    command.arg("--target-dir").arg(&target_dir);
    command.arg("--root").arg(&target_dir);
    // The probe reads Android ARM64 registers, regardless of the build host.
    command.env("CARGO_CFG_BPF_TARGET_ARCH", "aarch64");
    // Android compiler/linker flags must not leak into the BPF compilation.
    command
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS");
    if cfg!(debug_assertions) {
        command.arg("--debug");
    }
    let status = command
        .status()
        .context("Failed to build frame-analyzer-ebpf 0.2.0")?;
    ensure!(
        status.success(),
        "frame-analyzer-ebpf build failed: {status}"
    );

    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let destination = target_dir.join("bpfel-unknown-none").join(profile);
    fs::create_dir_all(&destination)?;
    fs::copy(
        target_dir.join("bin/frame-analyzer-ebpf"),
        destination.join("frame-analyzer-ebpf"),
    )?;
    Ok(())
}
