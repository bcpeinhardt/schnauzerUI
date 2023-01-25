# Installation

Schnauzer UI is not yet on a regular release schedule or producing pre-built binaries. To install
it, you'll need the rust toolchain installed.

To install Rust, go to [the install section of the rust website](https://www.rust-lang.org/tools/install)
and follow the download instructions for your operating system. 

Once you have the rust toolchain installed, you can build SchnauzerUI from source by running
`cargo install --git https://github.com/bcpeinhardt/schnauzerUI` from the terminal.

To verify proper installation, try running `sui -i` from a terminal. 
Hopefully, a chrome browser opened up on your computer. In the terminal, follow the prompts
until you're asked for a command. Try `url "https://youtube.com"`, and the open chrome browser
should navigate to youtube. Hit enter to save the command after it executes.

Congrats! You're all set up. To quit, type `exit` in the terminal and hit enter. It should close the browser for you,
but not before asking if you want to save your script. Type "N" to discard the script.

# Updates

The best way to keep Schnauzer UI up to date for now is using a tool called cargo-update. To install it,
type `cargo install cargo-update` in the terminal and hit enter.

Once installed, you can update all installed rust binaries (including Schnauzer UI) by running
`cargo install-update -g -a` in the terminal.
