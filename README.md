# FileCLassed

Efficient, lightweight, configurable file organizer. This project is very simple : it takes a file in certains directories (that can be configured), then organize them, and sends them to one directory (that can be configured, too).

## How it works

The way this program organizes files could be conflicting with some normal software behaviour. Note that this doesn't keep references between files ; LaTeX code would thus need to be a little changed. FileClassed should only be used for non-IT classification.

This program takes care of the creation date of this file. Thanks to Rust `creation` function, it does

A file that was created on September 2021 named `mt.Assignments.euler.pdf` would be move by the program to `Mathematics/Assignments/euler.pdf`. mt means Mathematics, when time info is disabled. Otherwise, it would be `2021/Mathematics/Assignments/September/euler.pdf`.

A file named `mt.hst.euler.pdf` would be moved to `Mathematics/History/euler.pdf`. hst means History, as you guessed.

Note that "meanings" can easily and fully be configured. What I just said isn't the absolute truth since shortcuts are in the defaults written in french, including months.

It is primarily intended to class scolar works, that's why it uses years and months.
Note that months and years are optional, since v1.1.1.

## Installing

Here are the instructions for installation :
- Clone this repo
- `cd` in the directory
- Run `cargo build --release`. The binary should be placed in `target/release/fcs`
- Move it to somewhere in PATH, and running `fcs --help` should get you started.

Then, you should have a new program named fcs that works as stated before.

## Configuring

See `fcs --help` and fcs.yml in the repo.

This program is free software (as stated in LICENSE), and published under the GPLv3 license.

