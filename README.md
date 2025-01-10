[![Testing CI](https://github.com/OleMussmann/Nix-Package-Search/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/OleMussmann/Nix-Package-Search/actions/workflows/test.yml) [![CodeCov](https://codecov.io/gh/OleMussmann/Nix-Package-Search/graph/badge.svg?token=1QLZ9AG8N1)](https://codecov.io/gh/OleMussmann/Nix-Package-Search)

# Nix-Package-Search
Cache the nix package list, query and sort by relevance.

Find installable packages at lightning speed and sort the result by relevance, split by ...

- exact hits, the package _matching your exact string_
- direct hits, packages that _start with the search string_
- indirect hits, packages that _contain the search string_

... in configurable individual colors, optionally separated by a newline. Have a look:

![Color output of nps neovim](https://i.imgur.com/XpSo8qW.png "nps neovim")

## Installation
### Try It Without Installing
    nix run github:OleMussmann/Nix-Package-Search

### "Installing" The Cheater Way
Add `nps = "nix run github:OleMussmann/Nix-Package-Search -- "` to your shell aliases. Don't forget the trailing double-dash. The program might be garbage collected every once in a while and will be automatically downloaded when needed.

```nix
programs.bash.shellAliases = {  # Replace `bash` with your shell name, if necessary.
  nps = "nix run github:OleMussmann/Nix-Package-Search -- "
};
```

### Declarative Installation (Recommended)
> :warning: The way of installing third-party flakes is highly dependent on your personal configuration. As far as I know there is no standardized, canonical way to do this. Instead, here is a generic approach via overlays. You will need to adapt it to your config files.

Add `nps` to your inputs:

```nix
inputs = {
  nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";

  nps.url = "github:OleMussmann/Nix-Package-Search";
  nps.inputs.nixpkgs.follows = "nixpkgs";
};
```

Add an overlay to your outputs:

```nix
outputs = { self, nixpkgs, ... }@inputs:
let
  overlays-third-party = final: prev: {
    nps = inputs.nps.defaultPackage.${prev.system};
    <other third party flakes you have>
  };
in {
  nixosConfigurations."<hostname>" = nixpkgs.lib.nixosSystem {
    system = "<your_system_architecture>";
    modules = [
      ({ config, pkgs, ... }: { nixpkgs.overlays = [ overlays-third-party ]; })
      ./configuration.nix
    ];
  };
};
```

Finally, add `nps` to your `systemPackages` in `configuration.nix`:

```nix
environment.systemPackages = with pkgs; [
    git
    nps
    ...
];
```

### Local Installation
Directly installing in your `nix profile` is generally discouraged, since it is not declarative.

```nix
nix profile install github:OleMussmann/Nix-Package-Search
```

### By Hand
- Clone this repository.
- Build with `cargo build --release`.
- Copy or symlink the `target/release/nps` executable to a folder in your `PATH`, or include it in your `PATH`.
- Dependencies: `rustc`, `cargo`

## Automate package scanning (optional)
- Set up a cron job or a systemd timer for `nps -r` (or `nps -e -r` for using the nix experimental features) at regular intervals. Make sure to do so with your local user environment.

```nix
systemd.timers."refresh-nps-cache" = {
  wantedBy = [ "timers.target" ];
    timerConfig = {
        OnCalendar = "weekly";
        Persistent = true;
        Unit = "hello-world.service";
    };
};

systemd.services."refresh-nps-cache" = {
  script = ''
    set -eu
    nps -r  # or `nps -e -r` if you use flakes
  '';
  serviceConfig = {
    Type = "simple";
    User = "YOU";  # replace with your username or ${user}
  };
};
```

## Usage
```markdown
Find SEARCH_TERM in available nix packages and sort results by relevance

List up to three columns, the latter two being optional:
PACKAGE_NAME  <PACKAGE_VERSION>  <PACKAGE_DESCRIPTION>

Matches are sorted by type. Show 'exact' matches first, then 'direct' matches, and finally 'indirect' matches.

  exact     SEARCH_TERM (in PACKAGE_NAME column)
  direct    SEARCH_TERMbar (in PACKAGE_NAME column)
  indirect  fooSEARCH_TERMbar (in any column)

Usage: nps [OPTIONS] [SEARCH_TERM]

Arguments:
  [SEARCH_TERM]
          Search for any SEARCH_TERM in package names, description or versions

Options:
  -c, --color[=<COLOR>]
          Highlight search matches in color

          [env: NIX_PACKAGE_SEARCH_COLOR_MODE=]
          [default: auto]
          [aliases: colour]
          [possible values: auto, always, never]

  -C, --columns[=<COLUMNS>]
          Choose columns to show

          [env: NIX_PACKAGE_SEARCH_COLUMNS=]
          [default: all]

          Possible values:
          - all:         Show all columns
          - none:        Show only PACKAGE_NAME
          - version:     Also show PACKAGE_VERSION
          - description: Also show PACKAGE_DESCRIPTION

  -d, --debug...
          Turn debugging information on

          Use up to four times for increased verbosity

  -e, --experimental[=<EXPERIMENTAL>]
          Use experimental flakes

          [env: NIX_PACKAGE_SEARCH_EXPERIMENTAL=true]
          [default: false]
          [possible values: true, false]

  -f, --flip[=<FLIP>]
          Flip the order of matches and sorting

          [env: NIX_PACKAGE_SEARCH_FLIP=true]
          [default: false]
          [possible values: true, false]

  -i, --ignore-case[=<IGNORE_CASE>]
          Ignore case

          [env: NIX_PACKAGE_SEARCH_IGNORE_CASE=]
          [default: true]
          [possible values: true, false]

  -r, --refresh
          Refresh package cache and exit

  -s, --separate[=<SEPARATE>]
          Separate match types with a newline

          [env: NIX_PACKAGE_SEARCH_PRINT_SEPARATOR=]
          [default: true]
          [possible values: true, false]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

- `nps PACKAGE_NAME` searches the cache file for packages matching the `PACKAGE_NAME` search string, see image above.
- The cache is created on the first call. Be patient, it might take a while. This is done under the hood by calling `nix-env -qaP`  (or `nix search nixpkgs ^` for experimental mode) and writing the output to a cache file. Subsequent queries are much faster.

### Configuration

`nps` can be configured with environment variables. You can set these in your config file. Below are the defaults. You only need to set the ones you want to have changed.

```nix
environment.sessionVariables = rec {
    NIX_PACKAGE_SEARCH_EXPERIMENTAL = "false";  # Set to "true" for flakes
    NIX_PACKAGE_SEARCH_FLIP = "false";
    NIX_PACKAGE_SEARCH_CACHE_FOLDER_ABSOLUTE_PATH = "/home/YOU/.nix-package-search";  # replace "YOU"!
    NIX_PACKAGE_SEARCH_COLUMNS = "all";
    NIX_PACKAGE_SEARCH_EXACT_COLOR = "magenta";
    NIX_PACKAGE_SEARCH_DIRECT_COLOR = "blue";
    NIX_PACKAGE_SEARCH_INDIRECT_COLOR = "green";
    NIX_PACKAGE_SEARCH_COLOR_MODE = "auto";
    NIX_PACKAGE_SEARCH_PRINT_SEPARATOR = "true";
    NIX_PACKAGE_SEARCH_IGNORE_CASE = "true";
};
```

#### `NIX_PACKAGE_SEARCH_EXPERIMENTAL`
Use the experimental `nix search` command. It pulls information from the nix flake registries instead of nix channels. This is useful if no channels are in use, or channels are not updated regularly.

- default: false
- possible values: true, false

#### `NIX_PACKAGE_SEARCH_FLIP`
Flip the order of matches? By default most relevant matches appear below, which is easier to read with long output. Flipping shows most relevant matches on top.

- default: false
- possible values: true, false

#### `NIX_PACKAGE_SEARCH_CACHE_FOLDER_ABSOLUTE_PATH`
Absolute path of the cache folder

- default: /home/ole/.nix-package-search
- possible values: path

#### `NIX_PACKAGE_SEARCH_COLUMNS`
Choose columns to show: PACKAGE_NAME plus any of PACKAGE_VERSION or PACKAGE_DESCRIPTION

- default: all
- possible values: all, none, version, description

#### `NIX_PACKAGE_SEARCH_EXACT_COLOR`
Color of EXACT matches, match SEARCH_TERM in PACKAGE_NAME

- default: magenta
- possible values: black, blue, green, red, cyan, magenta, yellow, white

#### `NIX_PACKAGE_SEARCH_DIRECT_COLOR`
Color of DIRECT matches, match SEARCH_TERMbar in PACKAGE_NAME

- default: blue
- possible values: black, blue, green, red, cyan, magenta, yellow, white

#### `NIX_PACKAGE_SEARCH_INDIRECT_COLOR`
Color of INDIRECT matches, match fooSEARCH_TERMbar in any column

- default: green
- possible values: black, blue, green, red, cyan, magenta, yellow, white

#### `NIX_PACKAGE_SEARCH_COLOR_MODE`
Show search matches in color

- default: auto (only show color if stdout is in terminal, suppress if e.g. piped)
- possible values: always, never, auto

#### `NIX_PACKAGE_SEARCH_PRINT_SEPARATOR`
Separate matches with a newline?

- default: true
- possible values: true, false

#### `NIX_PACKAGE_SEARCH_IGNORE_CASE`
Search ignore capitalization for the search?

- default: true
- possible values: true, false

## Development

### Tests
Most tests run quick, execute them with:

`cargo test`

Refreshing the cache needs a working internet connection and might take a while.
These tests are by default disabled. Include them with

`cargo test -- --include-ignored`

Use `tarpaulin` to check for code coverage. Make sure to `--include-ignored` tests and include the integration tests with `--follow-exec`. This can take a long time. To generate a HTML report, run the check with

`cargo-tarpaulin --all-features --workspace --timeout 360 --out Html --follow-exec -- --include-ignored`

### Benchmarking
First build `nps` in release mode.

`cargo build --release`

Then run benchmarks on the produced executable with:

`hyperfine './target/release/nps -e neovim'`

### Git Hooks
Review the hooks in the [hooks](./hooks) folder and use them with

`git config core.hooksPath hooks`

### Release

Document future changes in the [CHANGELOG.md](./CHANGELOG.md) under "Unreleased". Do a dry-run with:

`cargo-release release [LEVEL|VERSION]`

and review the changes. Possible choices for `LEVEL` are `beta`, `alpha` or `rc` for development (pre-) releases and `major`, `minor`, `patch` or `release` (remove pre-release extension) for production releases. Then execute the release with:

`cargo-release release [LEVEL|VERSION] --execute --no-publish`

## Contributing

1. Check existing issues or open a new one to suggest a feature or report a bug
1. Fork the repository and make your changes
1. Open a pull request
