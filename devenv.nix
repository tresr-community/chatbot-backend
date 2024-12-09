{
  pkgs,
  config,
  lib,
  ...
}:
let
  packages = with pkgs; [ ];

  devPackages = with pkgs; [
    cargo-generate
    cargo-leptos
    leptosfmt
    direnv
    figlet
    git
    hello
    jq
    just
    nodePackages.wrangler
    rclone
    toml-cli
    trivy
    worker-build
    yq-go
  ];
in
{
  name = "chatbot-backend";

  env = {
    PROJECT = config.name;
  };

  cachix = {
    pull = [
      "pre-commit-hooks"
      "nftreasure-community"
    ];
    push = "nftreasure-community";
  };

  devenv = {
    warnOnNewVersion = true;
  };

  dotenv = {
    enable = true;
    disableHint = false;
  };

  packages = packages ++ lib.optionals (!config.container.isBuilding) devPackages;

  enterShell = ''
    figlet -f starwars -w 180 $PROJECT

    hello --greeting="Hello ''${USER:-user}, welcome to the $PROJECT project!"

    echo ""
    echo "#########################"
    echo "#### Helper scripts #####"
    echo "#########################"
    echo "🦾"
    ${pkgs.gnused}/bin/sed -e 's| |••|g' -e 's|=| |' <<EOF | ${pkgs.util-linuxMinimal}/bin/column -t | ${pkgs.gnused}/bin/sed -e 's|^|🦾 |' -e 's|••| |g'
    ${lib.generators.toKeyValue { } (lib.mapAttrs (_name: value: value.description) config.scripts)}
    EOF
    echo "🦾"
    echo "#########################"
  '';

  languages = {
    nix = {
      enable = true;
    };
    shell = {
      enable = true;
    };
    rust = {
      enable = true;
      channel = "stable";
      components = [
        "rustc"
        "cargo"
        "clippy"
        "rustfmt"
        #"rust-analyzer"
      ];
      targets = [ "wasm32-unknown-unknown" ];
    };
    javascript = {
      enable = true;
      bun = {
        enable = true;
      };
      npm = {
        enable = true;
      };
    };
  };

  difftastic = {
    enable = true;
  };

  git-hooks = {
    excludes = [
      ".direnv/"
      ".git/"
      ".vscode/"
      ".target/"
      "vendor/"
      "themes/"
    ];
    hooks = {
      actionlint.enable = true;
      cargo-check.enable = true;
      check-json.enable = true;
      check-merge-conflicts.enable = true;
      check-shebang-scripts-are-executable.enable = true;
      check-symlinks.enable = true;
      check-yaml.enable = true;
      clippy = {
        enable = true;
        settings = {
          allFeatures = true;
        };
      };
      commitizen.enable = true;
      convco.enable = true;
      deadnix.enable = true;
      dialyzer.enable = true;
      editorconfig-checker.enable = true;
      gptcommit.enable = true;
      markdownlint = {
        enable = true;
        settings = {
          configuration = {
            MD013 = {
              line_length = 200;
            };
            MD033 = {
              allowed_elements = [
                "a"
                "br"
                "nobr"
                "pre"
                "sup"
              ];
            };
          };
        };
      };
      mixed-line-endings.enable = true;
      nixfmt-rfc-style.enable = true;
      pre-commit-hook-ensure-sops.enable = true;
      prettier = {
        enable = true;
        settings = {
          configPath = ".prettierrc.yaml";
        };
      };
      pretty-format-json = {
        enable = false;
        excludes = [
          "workers/.*/package.json"
          "workers/.*/package-lock.json"
        ];
      };
      revive = {
        enable = true;
        fail_fast = false;
      };
      ripsecrets = {
        enable = true;
        excludes = [ ];
      };
      rustfmt.enable = true;
      shellcheck = {
        enable = true;
        excludes = [ ];
      };
      shfmt.enable = true;
      staticcheck.enable = true;
      statix.enable = true;
      trim-trailing-whitespace.enable = true;
      trufflehog.enable = true;
      typos.enable = true;
      yamllint = {
        enable = true;
        settings = {
          configuration = ''
            extends: relaxed
            rules:
              line-length: disable
              indentation: enable
          '';
        };
      };
    };
  };

  starship = {
    enable = true;
    config = {
      enable = false;
    };
  };

  devcontainer = {
    enable = true;
    settings = {
      customizations = {
        vscode = {
          extensions = [
            "arrterian.nix-env-selector"
            "esbenp.prettier-vscode"
            "github.vscode-github-actions"
            "gruntfuggly.todo-tree"
            "johnpapa.vscode-peacock"
            "mkhl.direnv"
            "nhoizey.gremlins"
            "pinage404.nix-extension-pack"
            "redhat.vscode-yaml"
            "rust-lang.rust-analyzer"
            "streetsidesoftware.code-spell-checker"
            "tekumura.typos-vscode"
            "timonwong.shellcheck"
            "tuxtina.json2yaml"
            "vscodevim.vim"
            "wakatime.vscode-wakatime"
            "yzhang.markdown-all-in-one"
          ];
        };
      };
    };
  };

  enterTest = ''
    echo "Running devenv tests..."
  '';
}
