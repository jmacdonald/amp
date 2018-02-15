# Installation

Instructions to compile from source can be found below. If your OS isn't listed,
you'll need to follow the manual installation instructions and install the
specified dependencies (build dependencies can be removed after compilation,
if desired).

Adding support for your preferred distro is a great way to contribute to the
project!

## Arch Linux

Available via [AUR](https://aur.archlinux.org/packages/amp):

```bash
git clone https://aur.archlinux.org/amp.git
cd amp
makepkg -isr
```

## macOS

Available via [Homebrew](https://brew.sh):

```bash
brew tap jmacdonald/amp && brew install amp
```

## Manual installation

### Dependencies

* `git`
* `libxcb` (X11 clipboard support)
* `openssl`
* `zlib`

### Build Dependencies

* `cmake`
* `python3`
* `rust`

### Building

!!! info "Supported Release Channels"
    Amp's automated test suite is run using Rust's **stable** release channel;
    beta and nightly release channels are not officially supported.

1. Install Rust, either through your system's package manager or using [Rust's `rustup` toolchain management utility](https://www.rust-lang.org/en-US/install.html).
2. Install both the regular and build dependencies listed above.
3. Build and install:

    ```
    cargo install amp
    ```
