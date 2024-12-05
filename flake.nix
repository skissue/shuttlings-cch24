{
  inputs = {
    # Newer Shuttle CLI version; https://github.com/NixOS/nixpkgs/pull/361827
    nixpkgs.url = "github:nixos/nixpkgs/9f609517c8343ffb711ebe928805e8fbfcc1ecc7";
  };

  outputs = {nixpkgs, ...} @ inputs: let
    pkgs = nixpkgs.legacyPackages.x86_64-linux;
  in {
    devShells.x86_64-linux.default = with pkgs;
      mkShell {
        nativeBuildInputs = [rustc cargo];
        buildInputs = [cargo-shuttle rustfmt];
      };
  };
}
