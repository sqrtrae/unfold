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
cargo install unfold
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

TODO

# CHANGELOG

Please see [CHANGELOG.md](https://github.com/sqrtrae/unfold/CHANGELOG.md).