{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
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
