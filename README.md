# mlpretty

Prettifier for [Ocaml](https://ocaml.org/) error messages.

*mlpretty* changes the appearance of the error (and warning) messages,
by showing the offending line, underlining the span and adding some nice colors.
The message format is inspired by [Elm](http://elm-lang.org/)
(see [Compilers as Assistants](http://elm-lang.org/blog/compilers-as-assistants)).

## Example

Before

![error message before](before.png?raw=true)

After

![error message after](after.png?raw=true)

## Installation

*mlpretty* is written in [Rust](https://www.rust-lang.org/),
so you need to install it beforehand.

The simplest way to install is to use Rust's package manager, cargo:

```sh
cargo install --git https://github.com/krdln/mlpretty
```

Alternatively, you can clone the repository, and then run

```sh
cargo build --release
```

This will create binary `target/release/mlpretty`.
The binary won't require Rust to run. You can then copy
it or install with:

```sh
cargo install --path .
```

## Usage

When given no arguments, *mlpretty* acts as a pipe filter:

```sh
ocamlbuild foo.native | mlpretty
```

`ocamlc` writes on stderr, you need a redirect here:

```sh
ocamlc foo.ml 2>&1 | mlpretty # bash
ocamlc foo.ml ^| mlpretty     # fish
```

When given arguments, *mlpetty* treats them as command to run.
This can be used to prettify repl:

```sh
rlwrap mlpretty ocaml
```

Note that this converts only stdout.
Older versions of `ocaml` print to stderr. In such
a case you can create a bash script with a redirect.
