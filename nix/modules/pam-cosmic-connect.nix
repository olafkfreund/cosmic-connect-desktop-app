{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.cosmic-connect.phoneAuth;

in
{
  options.services.cosmic-connect.phoneAuth = {
    enable = mkEnableOption "COSMIC Connect phone authentication for PAM services";

    timeout = mkOption {
      type = types.int;
      default = 30;
      example = 60;
      description = ''
        Timeout in seconds for phone authentication requests.
        If the phone does not respond within this time, authentication falls back to password.
      '';
    };

    services = mkOption {
      type = types.listOf types.str;
      default = [
        "login"
        "sudo"
      ];
      example = [
        "login"
        "sudo"
        "cosmic-greeter"
        "polkit-1"
      ];
      description = ''
        List of PAM services to enable phone authentication for.
        Common services include:
        - login: Console login
        - sudo: Privilege escalation
        - cosmic-greeter: Display manager login
        - polkit-1: PolicyKit authentication dialogs
      '';
    };

    fallbackToPassword = mkOption {
      type = types.bool;
      default = true;
      description = ''
        Whether to fall back to password authentication if phone authentication fails.
        Disabling this is NOT recommended as it may lock you out of your system.
      '';
    };

    package = mkOption {
      type = types.package;
      default = pkgs.cosmic-connect or (throw "cosmic-connect package not available");
      defaultText = literalExpression "pkgs.cosmic-connect";
      description = ''
        The cosmic-connect package that provides the PAM module.
        The package must include lib/security/pam_cosmic_connect.so
      '';
    };
  };

  config = mkIf cfg.enable {
    # Assertions for safety
    assertions = [
      {
        assertion = cfg.timeout > 0;
        message = "services.cosmic-connect.phoneAuth.timeout must be greater than 0";
      }
      {
        assertion = cfg.services != [ ];
        message = "services.cosmic-connect.phoneAuth.services must not be empty";
      }
      {
        assertion = cfg.fallbackToPassword || (builtins.elem "sudo" cfg.services == false);
        message = ''
          Disabling fallbackToPassword for sudo service is extremely dangerous.
          You may lock yourself out of the system.
        '';
      }
    ];

    # Warning for security considerations
    warnings =
      optional (!cfg.fallbackToPassword) ''
        COSMIC Connect phone authentication is configured WITHOUT password fallback.
        If your phone is unavailable or the daemon fails, you will be locked out.
      ''
      ++ optional (builtins.elem "polkit-1" cfg.services) ''
        Phone authentication is enabled for polkit-1. This affects all PolicyKit
        authentication dialogs.
      '';

    # Ensure the PAM module is available system-wide
    environment.systemPackages = [ cfg.package ];

    # Configure PAM services using the standard NixOS PAM interface
    security.pam.services = genAttrs cfg.services (serviceName: {
      text = mkBefore ''
        # COSMIC Connect phone authentication
        auth  [success=done default=ignore]  pam_cosmic_connect.so timeout=${toString cfg.timeout}
      '';
    });

    # Ensure D-Bus service is available for PAM module communication
    services.dbus.packages = [ cfg.package ];

    # Configure Polkit policy for phone authentication requests
    security.polkit.extraConfig = ''
      polkit.addRule(function(action, subject) {
        if (action.id == "org.cosmicde.PhoneAuth.request" &&
            subject.local && subject.active) {
          return polkit.Result.YES;
        }
      });
    '';

    # Configuration file for phone auth preferences
    environment.etc."xdg/cosmic-connect/phone-auth.toml".text = ''
      # COSMIC Connect Phone Authentication Configuration

      [auth]
      timeout = ${toString cfg.timeout}
      fallback_to_password = ${if cfg.fallbackToPassword then "true" else "false"}
      services = [${concatMapStringsSep ", " (s: ''"${s}"'') cfg.services}]
    '';
  };
}
