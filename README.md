This project is archived. I have made a new lsdiff program in rust in my [dotfiles](https://github.com/REALERvolker1/homescripts/tree/main/.config/rustcfg/lsdiff-rs)

# LSDIFF

A simple rust "script" that shows you what files were added to or removed from a directory

I wrote this because I occasionally see random folders popping up in my home directory when I `ls ~` and I wrote this so I don't have to rely on noticing something and reading each line.

## Installation

Run the following commands:

```
# cd to whatever dir you install stuff from
cd "$HOME/.local/librs"
git clone "https://github.com/REALERvolker1/lsdiff" && cd lsdiff
cargo build --release
# use cp to copy, or ln -s to symlink (if you want to update it in-place)
ln -s "$PWD/target/release/lsdiff" "$HOME/.local/bin/"
```

Or you can download the file at `https://github.com/REALERvolker1/lsdiff/releases` 😅

## Configuration

You can configure this with environment variables. I will likely make it file-based if someone wants that.

If you change these, **delete the cache file and run the program again**.
`$LSDIFF_DIR`: The directory to diff. (Default: `$HOME`)
`$LSDIFF_CACHE`: The cache file. (Default: `$XDG_CACHE_HOME/lsdiff.list`)

These ones are purely for display. You don't have to worry about deleting cache for them.
`$LSDIFF_ICON_FOLDER`: The folder icon. (Default: ``)
`$LSDIFF_ICON_FILE`: The file icon. (Default: ``)

Run `lsdiff -u` to update the cache file manually

You can run `lsdiff -h` or `lsdiff --help` to get help.

## Notes

-   Some of this code was generated with ChatGPT
-   I might change the cache file format in the future. This will not affect users very much.
-   I might add a way to conveniently refresh the cache idk tho

Pull requests welcome!
