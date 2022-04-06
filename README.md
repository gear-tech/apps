# Gear Apps

[![Build][build_badge]][build_href]
[![License][lic_badge]][lic_href]

[build_badge]: https://github.com/gear-tech/apps/workflows/Build/badge.svg
[build_href]: https://github.com/gear-tech/apps/actions/workflows/build.yml

[lic_badge]: https://img.shields.io/badge/License-GPL%203.0-success
[lic_href]: https://github.com/gear-tech/apps/blob/master/LICENSE

## Prebuilt Binaries

Raw, optimized, and meta WASM binaries can be found in the [Releases section](https://github.com/gear-tech/apps/releases/tag/build).

## Building Locally

### ⚙️ Install Rust

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### ⚒️ Add specific toolchains

```shell
rustup toolchain add nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

... or ...

```shell
make prepare
```

### 🏗️ Build

```shell
cargo +nightly build --release
```

... or ...

```shell
make
```

## License

The source code is licensed under [GPL v3.0 license](LICENSE).
