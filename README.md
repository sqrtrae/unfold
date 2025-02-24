# unfold

`unfold` is a command line utility for replacing symbolic links with their targets.

# Table of Contents

* [Table of Contents](#table-of-contents)
* [Installation](#installation)
* [User Guide](#user-guide)
* [CHANGELOG](#changelog)

# Installation

`unfold` is available via cargo:

```sh
cargo install unfold-symlinks
```

# User Guide

For reference, below is the output of `unfold -h`. For a more detailed output, run `unfold --help`.

```text
Unfold symbolic links to their targets.

Usage: unfold [OPTIONS] <SYMLINK>...

Arguments:
  <SYMLINK>...  Symbolic links to unfold

Options:
  -f, --follow-to-source  Follow symbolic links to their source
  -n, --num-layers <NUM>  Follow up to NUM symbolic links
  -h, --help              Print help (see more with '--help')
  -V, --version           Print version
```

## Basic Usage

* Use `unfold` to replace a symbolic link to a file with a copy of said file:

```sh
# setup
echo "Hello World!" > greeting.txt
ln -s greeting.txt second_greeting.txt

# unfold second_greeting.txt to replace w/ copy of greeting.txt
unfold second_greeting.txt

# change contents of the original file
echo "Hello There!" > greeting.txt

# verify second_greeting.txt hasn't changed
cat second_greeting.txt  # output: 'Hello World!'
```

* `unfold` also works on symbolic links to directories. The contents of the new directory will be symbolic links to the contents of the target directory:

```sh
# setup
mkdir secret_stuff
echo "Krabby Patty Formula" > secret_stuff/secret_recipe.txt
ln -s secret_stuff important_stuff

# unfold the important stuff
unfold important_stuff

# after unfolding, important_stuff will be a directory, 
# containing symbolic links to the contents of the secret_stuff directory.
readlink important_stuff/secret_recipe.txt  # output: 'documents/secret_recipe.txt'
```

* If the target of the symbolic link is itself a symbolic link, then `unfold` will replace it
  with a copy of that symbolic link (i.e. a new symbolic link with identical target to that symbolic link):

```sh
# setup
echo "Trans rights are human rights!" > facts.txt
ln -s facts.txt laws_of_physics.txt
ln -s laws_of_physics.txt bridget_says.txt

# unfold
unfold bridget_says.txt

# verify symlink has changed to now target facts.txt
readlink bridget_says.txt  # output: 'facts.txt'
```

* You can `unfold` multiple symbolic links in the same command:

```sh
# setup
touch water earth fire air
ln -s water korra
ln -s earth kyoshi
ln -s fire roku
ln -s air aang

# unfold multiple symbolic links
unfold korra kyoshi roku aang
```

## Advanced Usage

* Use the `-f` option to unfold a symbolic link to the source file/directory (following all intermediate symbolic links).

```sh
# setup
echo "Source of wisdom" > grandparent
ln -s grandparent parent
ln -s parent you
ln -s you child
ln -s child grandchild

# unfold, following symbolic links to their source.
unfold -f grandchild

# change contents of grandchild and verify the other files haven't changed.
echo "Source of hope" > grandchild
cat child  # output: 'Source of wisdom'
cat grandparent  # output: 'Source of wisdom'
```

* Use the `-n <NUM>` option to unfold up to `<NUM>` symbolic links in a chain of symbolic links (if chain length is less than `<NUM>`, then this behaves identically to the option `-f`):

```sh
# setup
touch kanto
ln -s kanto johto
ln -s johto hoenn
ln -s hoenn sinnoh
ln -s sinnoh unova

# unfold up to 3 symbolic links in the chain
unfold -n 3 unova

readlink unova  # output: 'kanto'
```

# CHANGELOG

Please see [CHANGELOG.md](https://github.com/sqrtrae/unfold/blob/main/CHANGELOG.md).