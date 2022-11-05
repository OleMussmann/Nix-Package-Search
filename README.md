# Nix-Package-Search
Cache the nix package list, query and sort by relevance.

Searching for installable packages in NixOS is a pain. `nps` to the rescue! Find packages at lightning speed and sort the result by relevance, splitting by ...

- exact hits, the package _matching your exact string_
- direct hits, packages that _start with the search string_
- indirect hits, packages that _contain the search string_

... in configurable individual colors, optionally separated by a newline. Have a look, this query took less than 0.1s:

![Color output of nps neovim](https://i.imgur.com/XpSo8qW.png "nps neovim")

## Installation
- Clone this repository.
- Copy or symlink the `nps` script to a folder in your `PATH`, or include the `Nix-Package-Search` folder in your `PATH`.

## Automate package scanning (optional)
- Set up a cron job or a systemd timer for `nps -s` at regular intervals. Make sure to do so with your local user environment.

## Usage
    Usage: nps [OPTION...] PACKAGE_NAME
    Find PACKAGE_NAME in available nix packages and sort results by relevance.
    
      -c, --color          force color preservation
      -h, --help           display this help message and exit
      -n, --no-separator   don't separate matches with a newline
      -s, --scan           query packages and cache results

- `nps PACKAGE_NAME` searches the cache file for packages matching the `PACKAGE_NAME` search string, see image above.
- The cache is created on the first call. Be patient, it might take a while. This is done under the hood by calling `nix-env -qaP` and writing the output to a cache file. Subsequent queries are much faster.

### Options

- `-c` or `--color` forces color preservation when piping the output to other commands. `nps -c FOO | head` would still show colored output. But be aware that other commands like `grep` might trip over the color codes.
- `-h` or `--help` displays the above usage message.
- `-n` or `--no-separator` omits the new line separating search result types.
- `-s` or `--scan` forces a fresh `nix-env -qaP` query, getting the latest packages. This might take a while and would be best automated with `cron` or another scheduling utility.

### Configuration

For now certain settings are hard-coded in the script. Set color codes to `0` to remove highlighting and see [this excellent askubuntu post](https://askubuntu.com/questions/1042234/modifying-the-color-of-grep) for more color options.

#### `NIX_PACKAGE_SEARCH_FOLDER`
Folder where the cache is stored. Default: `${HOME}/.nix-package-search`

#### `NIX_PACKAGE_SEARCH_CACHE_FILE`
File name of the cache. Default: `nps.cache`

#### `EXACT_COLOR`
Color highlight of exact matches, matches `nixos.PACKAGE_NAME`. Default: `01;35` (purple)

#### `DIRECT_COLOR`
Color highlight of direct matches, matches `nixos.PACKAGE_NAME-foo`. Default: `01;34` (blue)

#### `INDIRECT_COLOR`
Color highlight of indirect matches, matches `nixos.foo-PACKAGE_NAME-bar`. Default: `01;32` (green)

#### `PRINT_SEPARATOR`
Whether to separate matching types by newlines. Default: `true`

## Contributing

1. Check existing issues or open a new one to suggest a feature or report a bug
1. Fork the repository and make your changes
1. Open a pull request
