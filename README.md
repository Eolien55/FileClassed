# FileClassed
[![forthebadge](https://forthebadge.com/images/badges/built-with-love.svg)](https://forthebadge.com) [![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)


Efficient, lightweight, configurable file organizer. This project is very simple : it takes a file in certains directories (that can be configured), then organize them, and finally moves them to one directory (that can be configured, too).

Feel free to open pull requests or issues.

## Example
See `before-after.txt`.

## How it works

The way this program organizes files could be conflicting with some normal software behaviour. Note that this doesn't keep references between files ; LaTeX code with inclusion of other files could thus need to be a little changed. FileClassed should only be used for non-IT classification. Since codes can be very easily changed, this program can be applied to a variety of domains.

Through the following document, the word shortcuts will be used several times. It isn't very clear, so I prefer explain here. The program is configured with "shortcuts", which is a pair Short string / Meaning of the string.

For exmple, `fr / Français` is a shortcut ; everytime the string `fr` is encountered alone, it will be expanded into `Français`.

### Basic

Here's a first example of the behavior of this organizer.

<img src="https://raw.githubusercontent.com/Eolien55/FileClassed/master/schema-basic.svg">

Here, the program expands each part seperated by commas. `mt` is expanded to `Mathematics`, and `asg` to `Assignments`. The last two parts are never expanded : they constitute the file name.

Note that the way the parts are expanded can be configured, via the `-c` option when running FileClassed or the `codes` field in the configuration file.

### No expanding

Here's a second example of the program's behavior.

<img src="https://raw.githubusercontent.com/Eolien55/FileClassed/master/schema-multiple.svg">

Here, we suppose that the program is configured to expand `emp` into `Empire` and `hst` into `History`. We also suppose that `France` isn't a registered shortcut.

This is almost the same case as in the first example, but this time, a part isn't expanded. `France` keeps being `France` in the final path since it's not configured as a shortcut.

When the program cannot expand a shortcut, it uses it plain.

### Variable replacement

In this example, we'll show how include shortcuts inside plain strings, or even combine shortcuts.

<img src="https://raw.githubusercontent.com/Eolien55/FileClassed/master/schema-variable-replacement.svg">

We suppose that `fr` means `French`, that `hst` means `History` and that `emp` means `Empire`.

Here the program tries to expand `fr` then `hst`. It succeeds, and replace `{fr}` by `Français` and `{hst}` by `History`. Like in the previous example, if FileClassed fails to expand a part, it replaces it plain. So `{France}` would be replaced by `France`, but this isn't useful, at all.

Note that it expands those variables recursively. Say, hypothetically, that we configured `1` as `one` and `fone` as `Fossil number One`. The file name `{f{1}}.image.jpg` would be expanded to `{fone}.image.jpg` and then `Fossil number One/image.jpeg`.

## Installing

Here are the instructions for installation :
- Clone this repo
- `cd` in the directory
- Run `cargo build --release`. The binary should be placed in `target/release/fcs`
- Move it to somewhere in PATH, and running `fcs --help` should get you started.

Then, you should have a new program named fcs that works as stated before.

## Configuring

Note that you can generate a config file by running fcs with the `-g` flag.

The configuration file is located to `C:\Users\<User>\AppData\Roaming\fcs\init.yml` in Windows, `/home/<user>/.config/fcs/init.yml` for Linux, \*BSD and other Unix-like operating systems, and `/Users/<User>/Library/Application Support/fcs/init.yml` for MacOS.

There are multiple fields, and (almost) each of them corresponds to an option or a flag of this program.
Refer to `fcs --help` for more information about each of the options.

The `dirs` field / CLI option sets which directories to look for files to organize.

The `dest` field / CLI option sets which directory to move the files, once expanded. All organized files are moved to it.

The `once` field / CLI flag makes the program organize the files only once. By default, it organizes them, then sleep and organizes them again.

The `sleep` field / CLI option sets the sleep time between each loop, in milliseconds.

The `static_mode` field / CLI flag disables the program looking for configuration changes.

The `timeinfo` field / CLI flag enables file info in the path. With `mt / Mathematics`, the file `mt.exponentiation.txt` wouldn't be expanded to `Mathematics/exponentiation.txt` but to `<year>/Mathematics/<month>/exponentiation.txt`.

The `codes` field / CLI option sets the "shortcuts".

The `completion` CLI option generates shell specific completion script and print it to stdout.

The `begin_var` field / CLI option sets the character to detect a variable 'lookup' ('{' by default).

The `end_var` field / CLI option sets the character to detect the end of a variable 'lookup' ('}' by default).

The `separator` field / CLI option sets the separator to separate each filename part ('.' by default).

The `filename-separators` field / CLI option sets the number of characters that are the separator in the filename (1 by default).

Note that the default values are in french, so you really should write your configuration file.

This program is free software (as stated in LICENSE), and published under the MIT license.
