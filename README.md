# fntldr

`fntldr` can temporarily install (or "load") fonts in the system. It is also an easier-to-use replacement of `FontLoader`, `FontLoaderSub` and `ListAssFonts`.

The program supports GNU/Linux (using Fontconfig) and Windows.

## Usage

Notice: Do not force kill the process, or fonts and temporary files won't be properly cleaned.

Default cache location is `{user cache dir}/fntldr/fntldr_cache.bin`.

Add `--help` after a subcommand for more information.

### Load font files

```
fntldr load [--dir DIRECTORY]... [--recurse DIRECTORY]... [FONT_FILE]...
```

### Load used fonts in (A)SSA subtitles

```
fntldr load-by [--dir DIRECTORY]... [--recurse DIRECTORY]... [--cache CACHE] [--font-list]
```

When `--cache` is not specified, it first try to load `./fntldr_cache.bin`, if not present, then try default location.

### Build index cache

```
fntldr index [--dir DIRECTORY]... [--recurse DIRECTORY]... [--cache CACHE] [--portable]
```

By default, it tries to load cache from default location and update it, or you can specify `--cache` to operate on given cache file.

### List used fonts in (A)SSA subtitles

```
fntldr list [--dir DIRECTORY]... [--recurse DIRECTORY]... [--cache [CACHE]] [--font-list] [--export DIRECTORY]
```

Font reexporting is not yet available on Windows.

## Drag-and-drop Compatibility

Rename the executable to `fontloader` / `fontloadersub` / `listassfonts` (case insensitive) to use DnD compatibility modes.

### FontLoader mode

```
fontloader [FONT_FILE]...
```

Equivalent to `fntldr load ...` or `fntldr load --dir .` (without parameters).
If running without any parameters, it loads all font files in the current working directory.

### FontLoaderSub mode

```
fontloadersub [SUBTITLES_DIR]...
```

Equivalent to `fntldr load-by --recurse ...` after cache is built.
If the cache is not found, it recursively scans the current working directory for font files, and build the cache, which is the same as running:

```
fntldr index --recurse . --cache . --portable
```

### ListAssFonts mode

```
listassfonts [SUBTITLES_DIR]...
```

Equivalent to `fntldr list --recurse ...`.

The program currently does not have all the features as in `ListAssFonts`, and they will be added at `fntldr list`, this mode is just for simple listing.

## Font installation check using `Fontconfig`

The program may load unnecassary fonts if you have aliases in your config, and `fntldr list` may mark them as not installed. So sad!

For a detailed reason, see the comments in `src/system/linux.rs`, above `impl FindFont ...`.
