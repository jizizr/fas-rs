# 构建

CI 使用 Ubuntu 24.04、NDK r30（30.0.16248370）、Node.js 22、pnpm 10.11.0、cargo-ndk 4.1.2 和 bpf-linker 0.12.0。Rust 使用仓库声明的 nightly；BPF 程序固定为 frame-analyzer-ebpf 0.2.0 并使用它的锁文件。

## Linux

先安装 Node.js 22、Rust/rustup、NDK r30，以及 `gcc-multilib`、`curl`、`zstd`、`unzip`。

```sh
rustup toolchain install nightly --component rust-src --target aarch64-linux-android
cargo install cargo-ndk --version 4.1.2 --locked
npm install -g pnpm@10.11.0

curl --fail --location -o /tmp/bpf-linker.tar.zst https://github.com/aya-rs/bpf-linker/releases/download/v0.12.0/bpf-linker-x86_64-unknown-linux-musl.tar.zst
echo '6789717c8cb6a2a5d920f3b6bfac2bd7efc00f6228ecd149108a99699a75c679  /tmp/bpf-linker.tar.zst' | sha256sum --check
mkdir -p "$HOME/.local/bin"
tar --zstd -xf /tmp/bpf-linker.tar.zst -C "$HOME/.local/bin"
export PATH="$HOME/.local/bin:$PATH"

# 替换为本机安装路径。
export ANDROID_NDK_HOME=/path/to/android-ndk-r30
export LIBCLANG_PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/lib"

cargo xtask build       # output/fas-rs(debug).zip
cargo xtask build -r    # output/fas-rs(release).zip
python3 .github/scripts/check-elf-alignment.py target/aarch64-linux-android/release/fas-rs
```

请使用 NDK 自带的 libclang，避免宿主机较旧的 Clang 无法解析新 NDK 头文件。构建仍以 API 31 为最低版本，无需把最低系统版本提高到 Android 17。

## macOS

使用相应的 NDK macOS 包，将 `LIBCLANG_PATH` 的 host 目录改为 `darwin-x86_64/lib`。Apple Silicon 的 BPF linker 下载地址为：

https://github.com/aya-rs/bpf-linker/releases/download/v0.12.0/bpf-linker-aarch64-apple-darwin.tar.zst

SHA-256：`08be0f9ae1e153944ec7e180d587bd9c2935d8dbdddf95e4d92205ee660867f4`。

## 验证

- Linux 上运行 `cargo test --locked -p frame-analyzer --lib`，检查 4/16/64 KiB 页及无效页大小处理。
- 运行 `actionlint .github/workflows/ci.yml` 检查工作流。
- 修改 Rust 文件后运行 `cargo fmt --all`。
- 使用 `webui/pnpm-lock.yaml` 安装 WebUI；构建会自动执行 `pnpm install --frozen-lockfile`，不再在仓库根目录安装 Next/React。
- `vendor/frame-analyzer/LOCAL_CHANGES.md` 记录了依赖补丁的来源和范围。
