{
  description = "Zap - a cross-platform application launcher";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    bun2nix = {
      url = "github:nix-community/bun2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      bun2nix,
    }:
    flake-utils.lib.eachSystem
      [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ]
      (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ bun2nix.overlays.default ];
          };
          lib = pkgs.lib;
          isLinux = pkgs.stdenv.isLinux;
          isDarwin = pkgs.stdenv.isDarwin;

          version = "0.1.5";

          # Linux: GTK, WebKit, X11/Wayland libraries
          linuxDeps = with pkgs; [
            webkitgtk_4_1
            gtk3
            libsoup_3
            glib
            glib-networking
            cairo
            pango
            gdk-pixbuf
            openssl
            dbus
            librsvg
            libayatana-appindicator
            libX11
            libXtst
            libxcb
            xdotool
            wayland
            libxkbcommon
          ];

          # macOS: Tauri uses native WebView — only need Apple frameworks
          darwinDeps =
            with pkgs;
            [
              openssl
              libiconv
            ]
            ++ (with pkgs.darwin.apple_sdk.frameworks; [
              AppKit
              CoreServices
              CoreFoundation
              Foundation
              Security
              WebKit
              Cocoa
              Carbon
            ]);

          buildInputs = if isDarwin then darwinDeps else linuxDeps;

        in
        {
          packages = rec {
            zap = pkgs.rustPlatform.buildRustPackage {
              pname = "zap";
              inherit version;

              src = lib.cleanSource self;

              cargoLock.lockFile = ./Cargo.lock;

              # bun2nix: per-package fetches from bun.nix (regenerate with `bunx bun2nix`)
              bunDeps = pkgs.bun2nix.fetchBunDeps {
                bunNix = ./bun.nix;
              };

              nativeBuildInputs =
                with pkgs;
                [
                  pkg-config
                  bun
                  pkgs.bun2nix.hook # Sets up node_modules from pre-fetched bun cache
                  jq
                ]
                ++ lib.optionals isLinux [ wrapGAppsHook3 ];

              inherit buildInputs;

              postPatch = ''
                # Remove beforeBuildCommand — we build the frontend ourselves in preBuild
                ${pkgs.jq}/bin/jq 'del(.build.beforeBuildCommand)' \
                  src-tauri/tauri.conf.json > $TMPDIR/tauri.conf.json
                cp $TMPDIR/tauri.conf.json src-tauri/tauri.conf.json
              '';

              # bun2nix.hook has already set up node_modules; build the SvelteKit frontend
              preBuild = ''
                export HOME=$TMPDIR
                bun run build
              '';

              cargoBuildFlags = [ "-p" "zap" ];

              # Disable bun2nix's default phases — rustPlatform handles build/check/install
              dontUseBunBuild = true;
              dontUseBunCheck = true;
              dontUseBunInstall = true;

              doCheck = false;

              # Linux: ensure dlopen'd libraries (X11, Wayland, WebKit) are found at runtime
              preFixup = lib.optionalString isLinux ''
                gappsWrapperArgs+=(
                  --set WEBKIT_DISABLE_DMABUF_RENDERER 1
                  --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath linuxDeps}"
                )
              '';

              postInstall =
                if isLinux then
                  ''
                    # Desktop entry
                    install -Dm644 /dev/stdin $out/share/applications/zap.desktop <<'EOF'
[Desktop Entry]
Name=Zap
Comment=Application Launcher
Exec=zap
Icon=zap
Type=Application
Categories=Utility;
StartupWMClass=zap
EOF

                    # Icons
                    install -Dm644 src-tauri/icons/32x32.png $out/share/icons/hicolor/32x32/apps/zap.png
                    install -Dm644 src-tauri/icons/128x128.png $out/share/icons/hicolor/128x128/apps/zap.png
                    install -Dm644 src-tauri/icons/128x128@2x.png $out/share/icons/hicolor/256x256/apps/zap.png
                  ''
                else
                  ''
                    # macOS: install icon
                    mkdir -p $out/share/icons
                    cp src-tauri/icons/icon.icns $out/share/icons/zap.icns
                  '';

              meta = with lib; {
                description = "A cross-platform application launcher";
                license = licenses.mit;
                platforms = platforms.linux ++ [ "aarch64-darwin" ];
                mainProgram = "zap";
              };
            };

            default = zap;
          };

          apps.default = {
            type = "app";
            program = "${self.packages.${system}.zap}/bin/zap";
          };

          # Development shell with all Tauri build dependencies
          devShells.default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.zap ];
            packages = with pkgs; [
              bun
              rustup
              cargo-tauri
              pkgs.bun2nix
            ];
          };
        }
      )
    // {
      overlays.default = final: _prev: {
        zap = self.packages.${final.stdenv.hostPlatform.system}.default;
      };
    };
}
