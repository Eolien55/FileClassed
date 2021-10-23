# FileCLassed

Efficient, lightweight, configurable file organizer. This project is very simple : it takes a file in certains directories (that can be configured), then organize them, and finally moves them to one directory (that can be configured, too).

Feel free to open pull requests or issues.

## How it works

The way this program organizes files could be conflicting with some normal software behaviour. Note that this doesn't keep references between files ; LaTeX code with inclusion of other files could thus need to be a little changed. FileClassed should only be used for non-IT classification. Since codes can be very easily changed, this program can be applied to a variety of domains.

This program takes care of the creation date of this file. This does it this way :
`<YEAR>/<MEANING>/<MONTH>/<NAME>`.

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

See `fcs --help` and default.yml in the repo. For generating a config file, you can add `-g` flag at the very end of the command.
You can easily configure this program, by creating the according configuration file. In Windows, this would be `C:\Users\<User>\AppData\Roaming\fcs\init.yml`. This would be `/home/<user>/.config/fcs/init.yml` on Linux or BSD. Note that there is no such thing as a system-wide configuration file, for portability reasons (MacOS and Windows people couldn't use it). Note finally that editing the configuration file is a lot of pain on MacOS and Windows because of the ugly paths, so you may want to create a link to it in your home directory.

Note that FileClassed updates automatically when config file changes, except if you use the `-S` flag or set the `static_mode` field  to `true` in the config file.

This program is free software (as stated in LICENSE), and published under the GPLv3 license.
