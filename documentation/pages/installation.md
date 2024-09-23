# Installation

Instructions to compile from source can be found below. If your OS isn't listed,
you'll need to follow the manual installation instructions and install the
specified dependencies (build dependencies can be removed after compilation,
if desired).

Adding support for your preferred distro is a great way to contribute to the
project!

## NixOS

If you're using flakes, you can add Amp as an input to your configuration and
track the main branch. Here's what that looks like in a simplified example,
with Amp available in plain NixOS and Home Manager configurations:

```nix title="flake.nix" hl_lines="12-15 22 32 45"
{
  description = "System config";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";

    home-manager = {
      url = "github:nix-community/home-manager/release-24.05";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    amp = {
      url = "github:jmacdonald/amp/main";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    home-manager,
    amp,
    ...
  } @ inputs: let
    inherit (self) outputs;
  in {
    nixosConfigurations = {
      desktop = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";

        specialArgs = {
          inherit inputs outputs amp; # (1)!
        };

        # Main configuration
        modules = [ ./nixos ];
      };
    };

    homeConfigurations = {
      "jmacdonald@desktop" = home-manager.lib.homeManagerConfiguration {
        pkgs = nixpkgs.legacyPackages.x86_64-linux;

        extraSpecialArgs = {
          inherit inputs outputs amp; # (2)!
          host = "desktop";
        };

        # Main configuration
        modules = [ ./home-manager/default.nix ];
      };
    };
  };
}
```

1.  This adds the Amp flake to your NixOS config.

2.  This adds the Amp flake to your Home Manager config.

You can then use the flake in your NixOS/Home Manager modules:

```nix title="home-manager/default.nix" hl_lines="1 5"
{ pkgs, amp, ... }: { # (1)!

  # Text editors
  home.packages = [
    amp.packages.${pkgs.system}.default # (2)!
    pkgs.vim
  ];
}
```

1. Specify the flake as an argument to your module to get a reference to it.
2. Add Amp to the list of installed packages. This long format including the
   reference to `pkgs.system` is a necessary evil, since the Amp flake needs to
   know which system/architecture to target, and I've yet to find a way to set a
   default package that is able to automatically take that into consideration.

Now you can update Amp by running the following:

```shell
# Bump flake.lock
nix flake lock --update-input amp

# Build and switch to the new version
home-manager switch --flake /path-to-your-nixos-conf
```

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
brew install amp
```

## Manual installation

### Dependencies

* `git`
* `libxcb` (X11 clipboard support)
* `openssl`
* `zlib`

### Build dependencies

* `cmake`
* `rust`

### Building

!!! info "Supported Release Channels"
    Amp's automated test suite is run using Rust's **stable** release channel;
    beta and nightly release channels are not officially supported. The oldest
    version of Rust currently supported is **1.38.0**.

1. Install Rust, either through your system's package manager or using [Rust's `rustup` toolchain management utility](https://www.rust-lang.org/en-US/install.html).
2. Install both the regular and build dependencies listed above.
3. Build and install:

    ```
    cargo install amp
    ```
