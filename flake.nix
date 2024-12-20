{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    devshell.url = "github:numtide/devshell";
  };

  outputs = {
    self,
    nixpkgs,
    devshell,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [devshell.overlays.default];
    };
  in {
    devShells.${system}.default = with pkgs;
      pkgs.devshell.mkShell {
        packages = [rustc cargo rustfmt cargo-shuttle];
      };
  };
}
