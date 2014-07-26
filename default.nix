let
  pkgs = import <nixpkgs> { };
in {
  sphinx = pkgs.buildEnv {
    name = "vagga-sphinx-env";
    paths = with pkgs; with pkgs.pythonPackages; [
      gnumake
      bash
      coreutils
      sphinx
    ];
  };
}
