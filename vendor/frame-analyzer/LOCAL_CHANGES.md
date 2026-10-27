This directory contains frame-analyzer 0.3.4 from crates.io, under GPL-3.0.
Original source checksum: ecf8b73c073dc52621b2026b06d712df02eb17de48a1f7aa85273dd4ec464744.

Local changes:

- build.rs uses a preinstalled bpf-linker instead of compiling an unpinned latest
  release during every build. It explicitly builds frame-analyzer-ebpf 0.2.0 with
  nightly and its lockfile, and checks subprocess exit codes. The BPF register
  layout is explicitly aarch64, including when the build host is Linux x86_64.
- src/ebpf.rs overrides the BPF ring buffer capacity with the kernel's page size
  (at least 4 KiB). This supports both 4 KiB and 16 KiB devices, and 64 KiB pages.
- src/uprobe.rs keeps the existing Android <=16 symbols and adds the Android 17
  GraphicBuffer/Fence queueBuffer overload. The first argument remains Surface*,
  which is the identifier recorded by the existing BPF program.
- Cargo.toml omits the package-level resolver because this copy is a workspace member.

Android 17 signature source:
https://android.googlesource.com/platform/frameworks/native/+/refs/heads/android17-release/libs/gui/Surface.cpp

Runtime behavior on Android 17/OEM builds still requires device validation.
