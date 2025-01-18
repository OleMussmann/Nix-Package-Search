[![Testing CI](https://github.com/OleMussmann/Nix-Package-Search/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/OleMussmann/Nix-Package-Search/actions/workflows/test.yml) [![CodeCov](https://codecov.io/gh/OleMussmann/Nix-Package-Search/graph/badge.svg?token=1QLZ9AG8N1)](https://codecov.io/gh/OleMussmann/Nix-Package-Search)

# Nix-Package-Search
Cache the nix package list, query and sort by relevance.

Find installable packages at lightning speed and sort the result by relevance, split by ...

- indirect hits - packages that _contain the search string_,
- direct hits - packages that _start with the search string_,
- exact hits - the package that _matches your exact string_,

... in configurable individual colors, optionally separated by a newline. Have a look:

![Color output of nps neovim](https://i.imgur.com/wNnWdxC.png "nps avahi")

## Installation
### Try It Without Installing
#### Flakes ❄️
```bash
nix run github:OleMussmann/Nix-Package-Search -- COMMAND_LINE_OPTIONS
```

#### No Flakes ☀️
```bash
nix --extra-experimental-features "nix-command flakes" run github:OleMussmann/Nix-Package-Search -- COMMAND_LINE_OPTIONS
```

### Declarative Installation (Recommended)
#### Flakes ❄️
> ⚠️ The way of installing third-party flakes is highly dependent on your personal configuration. As far as I know there is no standardized, canonical way to do this. Instead, here is a generic approach via overlays. You will need to adapt it to your config files.

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
  third-party-packages = final: prev: {
    nps = inputs.nps.packages.${prev.system}.default;
    <other third party flakes you have>
  };
in {
  nixosConfigurations."<hostname>" = nixpkgs.lib.nixosSystem {
    system = "<your_system_architecture>";
    modules = [
      ({ config, pkgs, ... }: { nixpkgs.overlays = [ third-party-packages ]; })
      ./configuration.nix
    ];
  };
};
```

Finally, add `nps` to your `systemPackages` in `configuration.nix`:

```nix
environment.systemPackages = with pkgs; [
    other_packages
    third-party-packages.nps
    ...
];
```

#### No Flakes ☀️
Add `nps` to your `systemPackages` in `configuration.nix`:

```nix
environment.systemPackages = with pkgs; [
    other_packages
    (builtins.getFlake "github:OleMussmann/Nix-Package-Search").packages.${builtins.currentSystem}.nps
    ...
];
```

### By Hand
- Clone this repository and `cd Nix-Package-Search` into it.
- Build with `cargo build --release`. Dependencies needed: `gcc`, `cargo`
- Copy or symlink the `target/release/nps` executable to a folder in your `PATH`, or include it in your `PATH`.

## Automate Package Scanning (Optional)
You can run `nps -r` (or `nps -e -r` for using the nix "experimental" features a.k.a flakes) every once in a while to refresh the package cache, or you can set up a systemd timer at regular intervals. If you automate it, make sure to do so with your local user environment.

```nix
systemd.timers."refresh-nps-cache" = {
    wantedBy = [ "timers.target" ];
    timerConfig = {
        OnCalendar = "weekly";  # or however often you'd like
        Persistent = true;
        Unit = "refresh-nps-cache.service";
    };
};

systemd.services."refresh-nps-cache" = {
    # Make sure `nix` and `nix-env` are findable by systemd.services.
    path = ["/run/current-system/sw/"];
    serviceConfig = {
        Type = "oneshot";
        User = "REPLACE_ME";  # ⚠️ replace with your "username" or "${user}", if it's defined
    };
    script = ''
        set -eu
        echo "Start refreshing nps cache..."
        # ⚠️ note the use of overlay (as described above), adjust if needed
        # ⚠️ use `nps -dddd -e -r` if you use flakes
        ${pkgs.third-party-packages.nps}/bin/nps -r -dddd
        echo "... finished nps cache with exit code $?."
    '';
};
```

### Testing Automated Package Scanning
- Test the service by starting it by hand and checking the logs.
  ```bash
  sudo systemctl start refresh-nps-cache.service
  journalctl -xeu refresh-nps-cache.service
  ```

- Test your timer by letting it run, for example, every 5 minutes. Adjust the timers as follows.

  ```diff
  -OnCalendar = "weekly";
  +OnBootSec = "5m";
  +OnUnitActiveSec = "5m";
  ```

  Check the logs if it ran successfully.
  ```bash
  journalctl -xeu refresh-nps-cache.service
  ```

  ⚠️  Don't forget to revert your changes afterwards.

## Usage

- `nps PACKAGE_NAME` searches the cache file for packages matching the `PACKAGE_NAME` search string.
- The cache is created on the first call. Be patient, it might take a while. This is done under the hood by capturing the output of `nix-env -qaP`  (or `nix search nixpkgs ^` for "experimental"/flake mode). Subsequent queries are much faster.

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

  -q, --quiet[=<QUIET>]
          Suppress non-debug messages

          [env: NIX_PACKAGE_SEARCH_QUIET=]
          [default: false]
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

### Configuration

Apart from command line flags, `nps` can be configured with environment variables. You can set these in your config file. Below are the defaults. Uncomment and change the ones you need.

```nix
environment.sessionVariables = {
    #NIX_PACKAGE_SEARCH_EXPERIMENTAL = "false";  # Set to "true" for flakes
    #NIX_PACKAGE_SEARCH_FLIP = "false";
    #NIX_PACKAGE_SEARCH_CACHE_FOLDER_ABSOLUTE_PATH = "/home/YOUR_USERNAME/.nix-package-search";
    #NIX_PACKAGE_SEARCH_COLUMNS = "all";
    #NIX_PACKAGE_SEARCH_EXACT_COLOR = "magenta";
    #NIX_PACKAGE_SEARCH_DIRECT_COLOR = "blue";
    #NIX_PACKAGE_SEARCH_INDIRECT_COLOR = "green";
    #NIX_PACKAGE_SEARCH_COLOR_MODE = "auto";
    #NIX_PACKAGE_SEARCH_PRINT_SEPARATOR = "true";
    #NIX_PACKAGE_SEARCH_IGNORE_CASE = "true";
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

- default: /home/YOUR_USERNAME/.nix-package-search
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

#### `NIX_PACKAGE_SEARCH_QUIET`
Suppress non-debug messages?

- default: false
- possible values: true, false

#### `NIX_PACKAGE_SEARCH_IGNORE_CASE`
Search ignore capitalization for the search?

- default: true
- possible values: true, false

## Contributing

1. Check existing issues or open a new one to suggest a feature or report a bug.
1. Fork the repository, check the [DEVELOPMENT.md](DEVELOPMENT.md) and make your changes
1. Open a pull request
