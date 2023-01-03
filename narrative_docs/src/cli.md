# CLI

The easiest way to get an overview of what the CLI can do is to run the help command! In a terminal,
run `sui --help`

The output should look something like this:

```
SchnauzerUI is a DSL for automated web UI testing

Usage: sui [OPTIONS] <--input-filepath <INPUT_FILEPATH>|--repl>

Options:
  -f, --input-filepath <INPUT_FILEPATH>
          Path to a SchnauzerUI .sui file to run
  -i, --repl
          Run SchnauzerUI in a REPL
  -o, --output-dir <OUTPUT_DIR>
          When --filepath passed, path to a directory for logs and screenshots. When --repl passed, path to a directory for the script to record the repl interactions
  -z, --headless
          Whether or not to display the browsers while the tests are running
  -b, --browser <BROWSER>
          Which browser to use. Supports "firefox" or "chrome". Defaults to chrome [default: chrome]
  -x, --datatable <DATATABLE>
          Path to an excel file which holds variable values for test runs
      --demo
          Highlight elements which are located to more clearly demonstrate process
  -h, --help
          Print help information
  -V, --version
          Print version information

```

## REPL/Iteractive Mode

The SchnauzerUI cli supports something called "REPL driven development". Running `sui -i` or `sui --repl` will launch a browser and prompt you for the name of
a test. Give your test a name then it will ask for a command.

With most UI testing tools, making a change requires rerunning at least the entire test. If you have a more complex setup,
you may find yourself rerunning 15 minutes worth of tests to validate one change. SchnauzerUI tries to be much 
more productive than that.

Type the command `url "https://youtube.com"`. The browser will navigate to youtube. You will then be asked whether 
you want to save the command. If the command produced no error, the default is yes and you can simply press Enter to
save it. SchnauzerUI lets you work one command at a time. If a command doesn't do what you expected it to do, don't save it and try something else. If you accidentally performed an action you didnt mean to, just manually set things back the way they were in the browser and keep going. 

When you're done writing your script, simply type the `exit` command. You will be asked whether you want to save the test.
The default is yes, so simply hit Enter and the script will be saved as a sui file. Now you can rerun it any time you 
like with `sui -f <path-to-test>.sui`.

## Demo Mode

Running or writing a script in demo mode makes a very simple change. Whenever an element is located using one 
of the locate commands (`locate`, `locate-no-scroll` etc.), that element is given a colored border. 
This makes understanding what a script is doing just by watching it execute a lot easier. 

## Datatables

If you have experience in automated UI testing, you've likely used datatables in some capacity. Most UI
testing tools provide support for specifying multiple test cases via a CSV file. SchnauzerUI is no 
different. 

To create a "Datatable", create a CSV file with a header for each variable you need in the test.
Then, create a row for each test case. In your SchnauzerUI script, anytime you want to use a variable,
use `<VariableName>`. Then, run the script with the flag `-x <path-to-datatable>.csv"`.

Most will want to use Excel, which is fine! Just save the file as a CSV when you're done.