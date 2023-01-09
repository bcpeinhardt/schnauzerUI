# Installation

To install SchnauzerUI, check [github](https://github.com/bcpeinhardt/schnauzerUI) for a 
download link for your operating system. Be sure to add it to your PATH.
Alternatively, if you have the rust toolchain installed, you can build it from source by running
`cargo install --git https://github.com/bcpeinhardt/schnauzerUI`

To verify proper installation, try running `sui -i` from a terminal. 
Hopefully, a chrome browser opened up on your computer. In the terminal, follow the prompts
until you're asked for a command. Try `url "https://youtube.com"`, and the open chrome browser
should navigate to youtube. Hit enter to save the command after it executes.

Congrats! You're all set up. To quit, type `exit` in the terminal and hit enter. It should close the browser for you,
but not before asking if you want to save your script. Type "N" to discard the script.
