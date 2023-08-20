# Quickstart

## Installation

To install Schnauzer UI, you'll need the [rust toolchain](https://www.rust-lang.org/tools/install) installed.
Then run `cargo install schnauzer_ui`. After installing, type `sui --help` and take a quick look
at the options.

You will also need a webdriver compliant process running to run your tests against. Schnauzer UI currently supports
firefox and chrome. `geckodriver`, `chromedriver`, or a `selenium standalone` should work just fine.
Make a note of what port you're running the process on.

If you've never worked with any of these before:
 - Try installing either `geckodriver` or `chromedriver` however you normally install packages (brew, apt, etc.) depending on whether you want to test in Firefox or Chrome respectively.
 - If that's not an option, you can manually download any of these as binaries, put them somwhere in your path,
 and run them.
 - If that all sounds like too much, and you're okay using Firefox, you can build geckodriver from source with
 `cargo install geckodriver` and then run it.

To start a REPL, first make sure your webdriver process is running.
Then make yourself a folder called `sui_tutorial` (or whatever you like) and open a terminal there. 
Type `sui -b <the browser> -p <the port>`. For example, `sui -b chrome -p 9515`.
(Note: Schnauzer UI defaults to using firefox on port 4444, so the commands `sui` and `sui -b firefox -p 4444` are equivalent).

You should see 1. a browser launch and 2. a prompt appear on the terminal asking for the name of your test.
Go through the start up options until you are prompted for a command. Then type `url "https://youtube.com"`
and hit enter. The browser should navigate to YouTube.
You will then be prompted for whether you want to save the command. The default is yes, so simply hit enter.

Congrats! You're up an running with Schnauzer UI. Feel free to try commands from the [reference](reference/statements_and_commands.md).

When you're ready to quit, type `exit` as a command and hit enter. You will be asked if you want to save your script. 
Simply type `yes` and hit enter.

There should now be a Schnauzer UI script saved as a file ending in `.sui` in your current directory. 
`cat` the file or open it up in your favorite text editor to take a look inside. You should see your saved commands
there. This is your brand new Schnauzer UI test script! To run the script, type `sui -f <path-to-the-file>` along 
with the port and browser arguments if they are not the default. The browser should launch again and run the entire script, then generate some test results. There should now also be an HTML file, a JSON file, and a screenshots folder
with any screenshots taken during the test run. 
- The HTML file is the standard formatting of the test report. You can open it in a browser to see the report.
- The JSON file is the same information in JSON format, in case you want to create custom styled reports or use the 
test result data programatically.





