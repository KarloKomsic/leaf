{
  description = "Leaf - personal e-book library manager";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "leaf";
        version = "1.1.0";
        src = ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        nativeBuildInputs = [ pkgs.pkg-config pkgs.wrapGAppsHook4 ];
        buildInputs = [ pkgs.gtk4 ];
        propagatedBuildInputs = [ pkgs.poppler-utils ];

        desktopItems = [
          (pkgs.makeDesktopItem {
            name = "leaf";
            exec = "leaf --gui"; 
            icon = "accessories-dictionary"; 
            desktopName = "Leaf";
            comment = "Personal e-book library manager";
            categories = [ "Office" "Utility" ];
          })
        ];

        postInstall = ''
          mkdir -p $out/share/applications
          cp -r $out/share/applications/* $out/share/applications/ || true
        '';

        meta = {
          description = "E-book library manager for Linux, written in Rust";
          homepage = "https://codeberg.org/KarloKomsic/leaf";
          mainProgram = "leaf";
        };
      };
    };
}
