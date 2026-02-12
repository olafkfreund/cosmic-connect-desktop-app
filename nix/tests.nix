{ pkgs ? import <nixpkgs> { }, ... }:

let
  # Import the package
  cosmic-ext-connect = pkgs.callPackage ./package.nix { };

in {
  # Test 1: Basic package build and installation
  package-build = pkgs.runCommand "cosmic-ext-connect-build-test" { } ''
    # Test that the package builds
    ${cosmic-ext-connect}/bin/cosmic-ext-connect-daemon --version > $out || true
    ${cosmic-ext-connect}/bin/cosmic-ext-applet-connect --version >> $out || true

    # Verify binaries exist
    test -f ${cosmic-ext-connect}/bin/cosmic-ext-connect-daemon || exit 1
    test -f ${cosmic-ext-connect}/bin/cosmic-ext-applet-connect || exit 1

    echo "Package build test: PASSED" >> $out
  '';

  # NixOS VM tests disabled until module is stable
  # Run with: nix build .#checks.<system>.package-build
}
