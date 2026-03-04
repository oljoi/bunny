{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
    kernel = pkgs.linuxPackages.kernel;

    bunnyModule = pkg: pkg.callPackage ({ stdenv, lib }: stdenv.mkDerivation {
      pname = "bunny";
      version = "1.0.0";
      src = ./.;

      nativeBuildInputs = pkg.kernel.moduleBuildDependencies;

      buildPhase = ''
        make -C ${pkg.kernel.dev}/lib/modules/${pkg.kernel.modDirVersion}/build \
          M=$(pwd) modules
      '';

      installPhase = ''
        make -C ${pkg.kernel.dev}/lib/modules/${pkg.kernel.modDirVersion}/build \
          M=$(pwd) INSTALL_MOD_PATH=$out modules_install
      '';
    }) {};

    bunny = bunnyModule pkgs.linuxPackages;
  in {
    packages.${system} = {
      bunny = bunny;
      default = bunny;
    };

    nixosModules.bunny = { config, ... }: {
      boot.extraModulePackages = [
        (bunnyModule config.boot.kernelPackages)
      ];
    };

    devShells.${system}.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        rustc
        rustfmt
        rust-bindgen
        clippy
        llvmPackages.clang
        llvmPackages.lld
        llvmPackages.libclang
        gcc
        gnumake
        just
      ];

      KDIR = "${kernel.dev}/lib/modules/${kernel.modDirVersion}/build";
      LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      RUST_LIB_SRC = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    };
  };
}
