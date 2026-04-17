# fntldr

`fntldr` can temporarily install (or "load") fonts into the system. It is also an easier-to-use replacement of `FontLoader`, `FontLoaderSub` and `ListAssFonts`.

The program supports GNU/Linux (using Fontconfig) and Windows.

## Usage

Notice: Do not force kill the process, or fonts and temporary files won't be properly cleaned.

Default cache location is `{user cache dir}/fntldr/fntldr.bin`.

`fntldr index` will update the current database, unless you use `-b`.

## Drag-and-drop Compatibility

Rename or link the executable to `fontloader` / `fontloadersub` / `listassfonts` (case insensitive) to use DnD compatible mode.

### FontLoader mode

Also scans directories.

If running without any parameters, it tries to load all font files in current directory.

Equivalent to `fntldr load ...` or `fntldr load .`.

### FontLoaderSub mode

Equivalent to `fntldr load-by -r ...`.

If the database is not found at `./fntldr.bin`, the program will scan the current directory and build it, equivalent to running `fntldr index -r . -c . -p`.

### ListAssFonts mode

Equivalent to `fntldr list -r ...`.

## Note for `Fontconfig` aliases

The program does not follow your custom aliases, it only checks for the original names. Some aliased fonts may be treated as not installed.
