# Nix-Package-Search
Cache the nix package list, query and sort by relevance.

Searching for installable packages in NixOS can be painful. `nps` to the rescue! Find packages at lightning speed and sort the result by relevance, split by ...

- exact hits, the package _matching your exact string_
- direct hits, packages that _start with the search string_
- indirect hits, packages that _contain the search string_

... in configurable individual colors, optionally separated by a newline. Have a look:

![Color output of nps neovim](https://i.imgur.com/XpSo8qW.png "nps neovim")

## Installation
The stable version is the default. To test new features, use `github:OleMussmann/Nix-Package-Search/development` instead. It has always the freshest, always the newest features, but could contain more bugs. Use at your own risk.

### Try It Out Without Installing
    nix run github:OleMussmann/Nix-Package-Search

### "Installing" the Cheater Way
Add `nps = "nix run github:OleMussmann/Nix-Package-Search -- "` to your shell aliases. Don't forget the trailing double-dash. The program might be garbage collected every once in a while and will be automatically downloaded when needed.

    programs.bash.shellAliases = {  # Replace `bash` with your shell name, if necessary.
      nps = "nix run github:OleMussmann/Nix-Package-Search -- "
    };

### Declarative Installation
> :warning: The way of installing third-party flakes is highly dependent on your personal configuration. As far as I know there is no standardized, canonical way to do this. Instead, here is a generic approach via overlays. You will need to adapt it to your config files.

Add `nps` to your inputs:

    inputs = {
      nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";

      nps.url = "github:OleMussmann/Nix-Package-Search";
      nps.inputs.nixpkgs.follows = "nixpkgs";
    };

Add an overlay to your outputs:

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

Finally, add `nps` to your `systemPackages` in `configuration.nix`:

      environment.systemPackages = with pkgs; [
          git
          nps
          ...
      ];

### Local Installation
Directly installing in your `nix profile` is generally discouraged, since it is not declarative.

    nix profile install github:OleMussmann/Nix-Package-Search

### By Hand
- Clone this repository.
- Copy or symlink the `nps` script to a folder in your `PATH`, or include the `Nix-Package-Search` folder in your `PATH`.
- Dependencies: `ripgrep` and GNU `getopt`

## Automate package scanning (optional)
- Set up a cron job or a systemd timer for `nps -r` (or `nps -e true -r` for using the nix experimental features) at regular intervals. Make sure to do so with your local user environment.

## Usage
    Usage: nps [OPTION]... SEARCH_TERM
    Find SEARCH_TERM in nix channels packages and sort results by relevance.

    List up to three columns, the latter two being optional:
    channel.PACKAGE_NAME  [VERSION]  [DESCRIPTION]
    
    Or, with nix experimental features, search the system registry flake instead.
    PACKAGE_NAME  [VERSION]  [DESCRIPTION]

    Mandatory arguments to long options are mandatory for short options too.

      -c, --color=WHEN               highlight search matches in color,
          --colour=WHEN                WHEN=
                                       {always} always emit color codes
                                        never   never emit color codes
                                        auto    only emit color codes when stdout
                                                is a terminal
      -C, --columns=COLUMNS          choose columns to show,
                                       COLUMNS=
                                       {all}         show all columns
                                        none         show only PACKAGE_NAME
                                        version      also show PACKAGE_VERSION
                                        description  also show PACKAGE_DESCRIPTION
      -e, --experimental=true|false  use experimental nix search {false}
      -f, --flip=true|false          flip the order of sorting {false}
      -h, --help                     display a short help message and exit
      -l, --long-help                display a long help message and exit
      -r, --refresh                  refresh package cache
      -s, --separator=true|false     separate match types with a newline {true}
      -v, --version                  print \`nps\` version and exit"

    The `nps --color=WHEN` option follows the `grep` color option, except that
    here the WHEN option is mandatory. Be aware that color codes can trip up
    subsequent commands like `grep`, if they occur within a match string.

    Matches are sorted by type. Show 'exact' matches first, then 'direct' matches,
    and finally 'indirect' matches.
      exact     SEARCH_TERM
                channel.SEARCH_TERM
      direct    SEARCH_TERM-bar
                channel.SEARCH_TERM-bar
      indirect  foo-SEARCH_TERM-bar (or match other columns)
                channel.foo-SEARCH_TERM-bar (or match other columns)

- `nps PACKAGE_NAME` searches the cache file for packages matching the `PACKAGE_NAME` search string, see image above.
- The cache is created on the first call. Be patient, it might take a while. This is done under the hood by calling `nix-env -qaP` (or `nix search SYSTEM_REGISTRY` for experimental mode) and writing the output to a cache file. Subsequent queries are much faster.

### Configuration

Settings are configured via environment variables. Override them when calling `nps`, or in the configuration file of your shell, e.g. `.bashrc` or `.zshrc`.

#### `NIX_PACKAGE_SEARCH_EXPERIMENTAL`
Use the experimental \"nix search\" command. It pulls information from the nix flake registries instead of nix channels. This is useful if no channels are in use, or channels are not updated regularly.

value: `"true"` | `"false"`

default: `"false"`

#### `NIX_PACKAGE_SEARCH_FLIP`
Flip the order of matches? By default most relevant matches appear first. Flipping the order makes them appear last and is thus easier to read with long output.

value: `"true"` | `"false"`

default: `"false"`

#### `NIX_PACKAGE_SEARCH_SHOW_PACKAGE_VERSION`
Show the `VERSION` column.

value: `"true"` | `"false"`

default: `"true"`

#### `NIX_PACKAGE_SEARCH_SHOW_PACKAGE_DESCRIPTION`
Show the `DESCRIPTION` column.

value: `"true"` | `"false"`

default: `"true"`

#### `NIX_PACKAGE_SEARCH_PRINT_SEPARATOR`
Separate matches with a newline?

value: `"true"` | `"false"`

default: `"true"`

#### `NIX_PACKAGE_SEARCH_FOLDER`
In which folder is the cache located?

value: path

default: `"${HOME}/.nix-package-search"`

#### `NIX_PACKAGE_SEARCH_EXACT_COLOR`
Color of EXACT matches, e.g.

    SEARCH_TERM
    registry#SEARCH_TERM
    channel.SEARCH_TERM

value: `"black"` `"blue"` `"green"` `"red"` `"cyan"` `"magenta"` `"yellow"` `"white"`<br>for advanced color options, see https://github.com/BurntSushi/ripgrep/blob/master/FAQ.md#how-do-i-configure-ripgreps-colors

default: `"purple"`

#### `NIX_PACKAGE_SEARCH_DIRECT_COLOR`
Color of DIRECT matches, e.g.

    SEARCH_TERM-bar
    registry#SEARCH_TERM-bar
    channel.SEARCH_TERM-bar

value: `"black"` `"blue"` `"green"` `"red"` `"cyan"` `"magenta"` `"yellow"` `"white"`<br>for advanced color options, see https://github.com/BurntSushi/ripgrep/blob/master/FAQ.md#how-do-i-configure-ripgreps-colors

default: `"blue"`

#### `NIX_PACKAGE_SEARCH_INDIRECT_COLOR`
Color of INDIRECT matches, e.g.

    foo-SEARCH_TERM-bar (or match other columns)
    registry#foo-SEARCH_TERM-bar (or match other columns)
    channel.foo-SEARCH_TERM-bar (or match other columns)

value: `"black"` `"blue"` `"green"` `"red"` `"cyan"` `"magenta"` `"yellow"` `"white"`<br>for advanced color options, see https://github.com/BurntSushi/ripgrep/blob/master/FAQ.md#how-do-i-configure-ripgreps-colors

default: `"green"`

#### `NIX_PACKAGE_SEARCH_COLOR_MODE`
`grep` color mode, show search matches in color

|value|effect|
|--|--|
| never | Never show color |
| always |  Always show color |
| auto | Only show color if stdout is in terminal, suppress if e.g. piped |

default: `"auto"`

## Acknowledgements
Bash argument parsing by [Robert Siemer](https://stackoverflow.com/a/29754866/996961).

## Contributing

1. Check existing issues or open a new one to suggest a feature or report a bug
1. Fork the repository and make your changes
1. Open a pull request
