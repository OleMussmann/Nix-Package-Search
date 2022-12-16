# Nix-Package-Search
Cache the nix package list, query and sort by relevance.

Searching for installable packages in NixOS can be painful. `nps` to the rescue! Find packages at lightning speed and sort the result by relevance, split by ...

- exact hits, the package _matching your exact string_
- direct hits, packages that _start with the search string_
- indirect hits, packages that _contain the search string_

... in configurable individual colors, optionally separated by a newline. Have a look:

![Color output of nps neovim](https://i.imgur.com/XpSo8qW.png "nps neovim")

## Installation
### Try It Out Without Installing
    nix run github:OleMussmann/Nix-Package-Search

### Local Installation
    nix profile install github:OleMussmann/Nix-Package-Search

### Declarative Installation (preferred)
Add to your inputs:

    nps.url = "github:OleMussmann/Nix-Package-Search";
    nps.inputs.nixpkgs.follows = "nixpkgs";

And add nps it to your `systemPackages`:

      environment.systemPackages = with pkgs; [
          git
          inputs.nps
          ...
      ];

### By Hand
- Clone this repository.
- Copy or symlink the `nps` script to a folder in your `PATH`, or include the `Nix-Package-Search` folder in your `PATH`.
- Dependencies: `ripgrep` and GNU `getopt`

## Automate package scanning (optional)
- Set up a cron job or a systemd timer for `nps -r` at regular intervals. Make sure to do so with your local user environment.

## Usage
    Usage: nps [OPTION]... SEARCH_TERM
    Find SEARCH_TERM in available nix packages and sort results by relevance.

    List up to three columns, the latter two being optional:
    channel.PACKAGE_NAME  [PACKAGE_VERSION]  [PACKAGE_DESCRIPTION]

    Mandatory arguments to long options are mandatory for short options too.

      -c, --color=WHEN            highlight search matches in color,
          --colour=WHEN             WHEN=
                                    {always} always emit color codes
                                     never   never emit color codes
                                     auto    only emit color codes when stdout
                                             is a terminal
      -C, --columns=COLUMNS       choose columns to show,
                                    COLUMNS=
                                    {all}         show all columns
                                     none         show only PACKAGE_NAME
                                     version      also show PACKAGE_VERSION
                                     description  also show PACKAGE_DESCRIPTION
      -h, --help                  display a short help message and exit
      -l, --long-help             display a long help message and exit
      -r, --refresh               refresh package cache
      -s, --separator=true|false  separate match types with a newline {true}
      -v, --version               print `nps` version and exit"

    The `nps --color=WHEN` option follows the `grep` color option, except that
    here the WHEN option is mandatory. Be aware that color codes can trip up
    subsequent commands like `grep`, if they occur within a match string.

    Matches are sorted by type. Show 'exact' matches first, then 'direct' matches,
    and finally 'indirect' matches.
      exact     channel.SEARCH_TERM
      direct    channel.SEARCH_TERM-bar
      indirect  channel.foo-SEARCH_TERM-bar (or match other columns)

- `nps PACKAGE_NAME` searches the cache file for packages matching the `PACKAGE_NAME` search string, see image above.
- The cache is created on the first call. Be patient, it might take a while. This is done under the hood by calling `nix-env -qaP` and writing the output to a cache file. Subsequent queries are much faster.

### Configuration

Settings are configured via environment variables. Override them when calling `nps`, or in your `*rc` file.

#### `NIX_PACKAGE_SEARCH_FOLDER`
In which folder is the cache located?

value: path

default: `"${HOME}/.nix-package-search"`

#### `NIX_PACKAGE_SEARCH_CACHE_FILE`
Name of the cache file

value: filename

default: `"nps.cache"`

#### `NIX_PACKAGE_SEARCH_SHOW_PACKAGE_VERSION`
Show the `PACKAGE_VERSION` column

value: `"true"` | `"false"`

default: `"true"`

#### `NIX_PACKAGE_SEARCH_SHOW_PACKAGE_DESCRIPTION`
Show the `PACKAGE_DESCRIPTION` column

value: `"true"` | `"false"`

default: `"true"`

#### `NIX_PACKAGE_SEARCH_EXACT_COLOR`
Color of EXACT matches `channel.MATCH`

value: `"black"` `"blue"` `"green"` `"red"` `"cyan"` `"magenta"` `"yellow"` `"white"`<br>for advanced color options, see https://github.com/BurntSushi/ripgrep/blob/master/FAQ.md#how-do-i-configure-ripgreps-colors

default: `"purple"`

#### `NIX_PACKAGE_SEARCH_DIRECT_COLOR`
Color of DIRECT matches `channel.MATCH-bar`

value: `"black"` `"blue"` `"green"` `"red"` `"cyan"` `"magenta"` `"yellow"` `"white"`<br>for advanced color options, see https://github.com/BurntSushi/ripgrep/blob/master/FAQ.md#how-do-i-configure-ripgreps-colors

default: `"blue"`

#### `NIX_PACKAGE_SEARCH_INDIRECT_COLOR`
Color of INDIRECT matches `channel.foo-MATCH-bar` `channel.foo     description MATCH more description`

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

#### `NIX_PACKAGE_SEARCH_PRINT_SEPARATOR`
Separate matches with a newline?

value: `"true"` | `"false"`

default: `"true"`

## Acknowledgements
Bash argument parsing by [Robert Siemer](https://stackoverflow.com/a/29754866/996961).

## Contributing

1. Check existing issues or open a new one to suggest a feature or report a bug
1. Fork the repository and make your changes
1. Open a pull request
