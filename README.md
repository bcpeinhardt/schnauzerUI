# schnauzerUI

SchnauzerUI is a human readable DSL for performing automated UI testing in the browser.
The main goal of SchnauzerUI is to increase stakeholder visibility and participation in
automated Quality Assurance testing. Rather than providing a shim to underling code written by
a QA engineer (see [Cucumber](https://cucumber.io/)), SchnauzerUI is the only source of truth for a
test's execution. In this way, SchnauzerUI aims to provide a test report you can trust.

If you would like to try it out, you can start with the [narrative docs](https://bcpeinhardt.github.io/schnauzerUI/)
or watch this intro youtube video (not yet filmed, sorry).

SchnauzerUI is under active development, the progress of which is being recorded as a
[youtube video series](https://www.youtube.com/playlist?list=PLK0mRy_gymKMLPlQ-ZAYfpBzXWjK7W9ER).

## Running the tests
Before running the tests you will need firefox and geckodriver installed and in your path.
Then

1. Start selenium. SchnauzerUI is executed against a standalone selenium grid (support for configuring
SchnauzerUI to run against an existing selenium infrastructure is on the todo list). To run the provided
selenium grid, `cd` into the selenium directory in a new terminal and run
```bash
java -jar .\selenium-server-<version>.jar standalone --override-max-sessions true --max-sessions 1000 --port 4444
```
No, this will not launch 1000 browsers. There is another setting, max-instances which controls the number of browsers
running at a time (defaults to 8 for firefox and chrome). Its just that now we can run as many tests as we like (up to 1000),
provided we only do 8 at a time.

2. The tests come with accompanying HTML files. The easiest way to serve the files to localhost
is probably to use python. In another new terminal, run the command
```python
python -m http.server 1234
```

From there, it should be a simple `cargo test`. The tests will take a moment to execute,
as they will launch browsers to run in.
