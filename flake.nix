{
  description = "Uploads files based on templates";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
  };
  outputs = {
    self,
    nixpkgs,
  }: let
    supportedSystems = ["x86_64-linux"];
    forEachSystem = nixpkgs.lib.genAttrs supportedSystems;
    overlayList = [self.overlays.default];
    pkgsBySystem = forEachSystem (system:
      import nixpkgs {
        inherit system;
        overlays = overlayList;
      });
  in {
    overlays.default = final: prev: {upf = final.callPackage ./package.nix {};};

    packages = forEachSystem (system: {
      upf = pkgsBySystem.${system}.upf;
      default = pkgsBySystem.${system}.upf;
    });
  };
}
