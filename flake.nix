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
        packages = [
          rustc
          cargo
          gcc
          rustfmt

          cargo-shuttle
          postgresql
          sqlx-cli
        ];
        commands = [
          {
            name = "database:init";
            help = "Initialize the Postgres database";
            command = ''
              initdb pgdata; \
              chmod -R 700 pgdata; \
              echo -e "Use the devshell command 'database:start'"
            '';
          }
        ];
        serviceGroups.database = {
          description = "Run a development PostgreSQL server";
          services.postgres = {command = "postgres -D ./pgdata";};
        };
        env = [
          {
            name = "DATABASE_URL";
            value = "postgresql://localhost:5432/";
          }
        ];
      };
  };
}
