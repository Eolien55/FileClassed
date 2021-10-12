# ScolarSorted

Efficient, lightweight, configurable file organizer. This project is very simple : it takes a file in certains directories (that can be configured), then organize them, and sends them to one directory (that can be configured, too).

## How it works

The way this program organizes files could be conflicting with some normal app behaviour. Note that it is chronolagically based ; it is thus only applicable to non-IT works.

This program takes care of the creation date of this file. The way it currently does it is specific to Linux ; however, it's quite a simple feature to adapt.

A file that was created on September 2021 named `mt.Assignments.euler.pdf` would be move by the program to `2021/Mathematics/Assignments/September/euler.pdf`. mt means Mathematics.

A file created on the same month but named `mt.hst.euler.pdf` would be moved to `2021/Mathematics/History/September/euler.pdf`. hst means History, as you guessed.

Note that "meanings" can easily be configured. What I just said isn't the absolute truth since shortcuts are in the defaults written in french, including months.

It is primarily intended to class scolar works, that's why it uses years and months.

## Installing

Here are the instructions for installation :
- Clone this repo
- `cd` in the directory
- Run `cargo build --release`. The binary should be placed in `target/release/fcs`
- Move it to somewhere in PATH, and running `fcs --help` should get you started.

Then, you should have a new program named fcs that works as stated before.

## Configuring

See `fcs --help` and fcs.yml in the repo.

(c) Eolien55, 2021

This program is free software (as stated in LICENSE), and published under the GPLv3 license.

