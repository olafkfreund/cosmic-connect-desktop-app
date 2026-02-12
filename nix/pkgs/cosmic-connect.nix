{
  lib,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  cmake,
  makeWrapper,
  openssl,
  libxkbcommon,
  wayland,
  wayland-protocols,
  libGL,
  libglvnd,
  mesa,
  pixman,
  libinput,
  libxcb,
  xcbutil,
  xcbutilwm,
  xcbutilimage,
  libdrm,
  fontconfig,
  freetype,
  udev,
  dbus,
  libpulseaudio,
  expat,
  glib,
  gtk3,
  pango,
  cairo,
  gdk-pixbuf,
  atk,
  pipewire,
  webkitgtk_4_1,
  gobject-introspection,
  gst_all_1,
  libopus,
  libgbm,
  stdenv,
}:

rustPlatform.buildRustPackage rec {
  pname = "cosmic-ext-connect";
  version = "0.18.0";

  src = fetchFromGitHub {
    owner = "olafkfreund";
    repo = "cosmic-ext-connect-desktop-app";
    rev = "v${version}";
    hash = ""; # IMPORTANT: Update this hash for nixpkgs submission
    # To get the hash, run: nix-prefetch-url --unpack https://github.com/olafkfreund/cosmic-ext-connect-desktop-app/archive/refs/tags/v0.1.0.tar.gz
  };

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
    outputHashes = {
      "accesskit-0.16.0" = "sha256-uoLcd116WXQTu1ZTfJDEl9+3UPpGBN/QuJpkkGyRADQ=";
      "atomicwrites-0.4.2" = "sha256-QZSuGPrJXh+svMeFWqAXoqZQxLq/WfIiamqvjJNVhxA=";
      "clipboard_macos-0.1.0" = "sha256-+8CGmBf1Gl9gnBDtuKtkzUE5rySebhH7Bsq/kNlJofY=";
      "cosmic-client-toolkit-0.1.0" = "sha256-KvXQJ/EIRyrlmi80WKl2T9Bn+j7GCfQlcjgcEVUxPkc=";
      "cosmic-config-1.0.0" = "sha256-pfT6/cYjA3CGrXr2d7aAwfW+7FUNdfQvAeOWkknu/Y8=";
      "cosmic-ext-connect-core-0.9.0" = "sha256-KRwM9DA8yoUJiJlLLrcrjhTa9D3X6wZYEhyA7/1X6zk=";
      "cosmic-freedesktop-icons-0.4.0" = "sha256-D4bWHQ4Dp8UGiZjc6geh2c2SGYhB7mX13THpCUie1c4=";
      "cosmic-panel-config-0.1.0" = "sha256-1Xwe1uONJbl4wq6QBbTI1suLiSlTzU4e/5WBccvghHE=";
      "cosmic-settings-daemon-0.1.0" = "sha256-1yVIL3SQnOEtTHoLiZgBH21holNxcOuToyQ+QdvqoBg=";
      "cosmic-text-0.17.1" = "sha256-NHjJBE/WSMhN29CKTuB7PyJv4y2JByi5pyTUDtVoF7g=";
      "dpi-0.1.1" = "sha256-Saw9LIWIbOaxD5/yCSqaN71Tzn2NXFzJMorH8o58ktw=";
      "iced_glyphon-0.6.0" = "sha256-u1vnsOjP8npQ57NNSikotuHxpi4Mp/rV9038vAgCsfQ=";
      "smithay-clipboard-0.8.0" = "sha256-4InFXm0ahrqFrtNLeqIuE3yeOpxKZJZx+Bc0yQDtv34=";
      "softbuffer-0.4.1" = "sha256-/ocK79Lr5ywP/bb5mrcm7eTzeBbwpOazojvFUsAjMKM=";
    };
  };

  nativeBuildInputs = [
    pkg-config
    cmake
    makeWrapper
    rustPlatform.bindgenHook
  ];

  buildInputs = [
    openssl
    libxkbcommon
    wayland
    wayland-protocols
    libGL
    libglvnd
    mesa
    pixman
    libinput
    libxcb
    xcbutil
    xcbutilwm
    xcbutilimage
    libdrm
    fontconfig
    freetype
    udev
    dbus
    libpulseaudio
    expat
    glib
    gtk3
    pango
    cairo
    gdk-pixbuf
    atk
    pipewire
    webkitgtk_4_1
    gobject-introspection
    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-good
    gst_all_1.gst-plugins-bad
    gst_all_1.gst-plugins-ugly
    gst_all_1.gst-libav
    # Opus codec for audio streaming
    libopus
    # DMA-BUF / GBM support for extended display capture
    libgbm
  ];

  # Build all workspace members with all plugin features
  cargoBuildFlags = [
    "--workspace"
    "--bins"
    "--features"
    "cosmic-ext-connect-daemon/remotedesktop,cosmic-ext-connect-daemon/screenshare,cosmic-ext-connect-daemon/video,cosmic-ext-connect-daemon/audiostream,cosmic-ext-connect-daemon/audiostream-opus,cosmic-ext-connect-daemon/extendeddisplay,cosmic-ext-connect-protocol/remotedesktop,cosmic-ext-connect-protocol/screenshare,cosmic-ext-connect-protocol/video,cosmic-ext-connect-protocol/audiostream,cosmic-ext-connect-protocol/audiostream-opus,cosmic-ext-connect-protocol/extendeddisplay,cosmic-ext-connect-protocol/low_latency,cosmic-ext-applet-connect/screenshare"
  ];

  # Skip tests for now - requires running dbus session
  doCheck = false;

  # Tell audiopus_sys to use system opus library instead of building from source
  OPUS_LIB_DIR = "${libopus}/lib";
  OPUS_INCLUDE_DIR = "${libopus}/include/opus";

  postInstall = ''
    # Install systemd user service
    install -Dm644 cosmic-ext-connect-daemon/cosmic-ext-connect-daemon.service \
      $out/lib/systemd/user/cosmic-ext-connect-daemon.service

    # Patch ExecStart path in systemd service
    substituteInPlace $out/lib/systemd/user/cosmic-ext-connect-daemon.service \
      --replace-fail "%h/.cargo/bin/cosmic-ext-connect-daemon" "$out/bin/cosmic-ext-connect-daemon" \
      --replace-fail "ProtectHome=read-only" "" \
      --replace-fail "ReadWritePaths=%h/.config/kdeconnect %h/.local/share/kdeconnect" ""

    # Install DBus service for activation
    mkdir -p $out/share/dbus-1/services
    cat > $out/share/dbus-1/services/io.github.olafkfreund.CosmicExtConnect.service << EOF
    [D-BUS Service]
    Name=io.github.olafkfreund.CosmicExtConnect
    Exec=$out/bin/cosmic-ext-connect-daemon
    SystemdService=cosmic-ext-connect-daemon.service
    EOF

    # Install desktop entries
    install -Dm644 cosmic-ext-applet-connect/data/cosmic-ext-applet-connect.desktop \
      $out/share/applications/cosmic-ext-applet-connect.desktop

    install -Dm644 cosmic-ext-messages-popup/data/io.github.olafkfreund.CosmicExtMessagesPopup.desktop \
      $out/share/applications/io.github.olafkfreund.CosmicExtMessagesPopup.desktop

    # Install desktop entry for manager (standalone window application)
    cat > $out/share/applications/cosmic-ext-connect-manager.desktop << EOF
    [Desktop Entry]
    Type=Application
    Name=COSMIC Connect Manager
    Comment=Manage connected devices for COSMIC Desktop
    GenericName=Device Manager
    Keywords=Cosmic;Iced;connect;phone;device;sync;manager;
    Icon=phone-symbolic
    Exec=$out/bin/cosmic-ext-connect-manager
    Categories=Settings;HardwareSettings;
    Terminal=false
    StartupNotify=true
    EOF

    # Install applet icon (using symbolic icon from theme)
    # Note: COSMIC Connect uses phone-symbolic from icon theme
    # No custom icons needed as it relies on system theme icons
  '';

  # Wrap binaries with required runtime library paths
  postFixup = ''
    # Wrap GUI binaries with display library paths
    for bin in cosmic-ext-applet-connect cosmic-ext-connect-manager cosmic-ext-messages-popup cosmic-ext-display-mirror; do
      wrapProgram $out/bin/$bin \
        --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath [
          wayland
          libxkbcommon
          libGL
          libglvnd
          mesa
        ]}"
    done

    # Wrap daemon with GStreamer plugin paths for video encoding/decoding
    wrapProgram $out/bin/cosmic-ext-connect-daemon \
      --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath [
        wayland
        libxkbcommon
        libGL
        libglvnd
        mesa
        pipewire
        libpulseaudio
      ]}" \
      --prefix GST_PLUGIN_SYSTEM_PATH_1_0 : "${lib.makeSearchPathOutput "lib" "lib/gstreamer-1.0" [
        gst_all_1.gstreamer
        gst_all_1.gst-plugins-base
        gst_all_1.gst-plugins-good
        gst_all_1.gst-plugins-bad
        gst_all_1.gst-plugins-ugly
        gst_all_1.gst-libav
      ]}"
  '';

  meta = {
    description = "Device connectivity for COSMIC Desktop";
    longDescription = ''
      COSMIC Connect provides seamless integration between your Android devices
      and COSMIC Desktop. Features include:

      - File sharing between devices
      - Clipboard synchronization
      - Notification mirroring
      - Battery status monitoring
      - Media player control (MPRIS)
      - Remote input (mouse and keyboard)
      - Remote desktop (VNC-based screen sharing)
      - SMS messaging and telephony notifications
      - Wake-on-LAN support
      - System monitoring

      This package includes:
      - cosmic-ext-applet-connect: Panel applet for quick device status
      - cosmic-ext-connect-manager: Standalone device manager window
      - cosmic-ext-connect-daemon: Background service with DBus activation
      - cosmic-ext-messages-popup: Web-based messaging interface

      Built with RemoteDesktop plugin support (requires PipeWire and Wayland).
    '';
    homepage = "https://github.com/olafkfreund/cosmic-ext-connect-desktop-app";
    changelog = "https://github.com/olafkfreund/cosmic-ext-connect-desktop-app/releases";
    license = lib.licenses.gpl3Plus;
    maintainers = with lib.maintainers; [ ]; # Add maintainer here for nixpkgs submission
    mainProgram = "cosmic-ext-applet-connect";
    platforms = lib.platforms.linux;

    # Requires COSMIC Desktop Environment (libcosmic)
    # Works on any Linux with Wayland support
  };
}
