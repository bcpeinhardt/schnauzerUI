# Tutorial

This tutorial will serve as an introduction to the basics of using SchnauzerUI to test Web Applications.
It assumes you have followed the [installation instructions](installation.md).

# Basics of the Language

Let's look at an example:
```SchnauzerUI
# Type in username
locate "Username" and type "test@test.com"

# Type in password
locate "Password" and type "Password123!"

# Click the submit button
locate "Submit" and click
```
A SchnauzerUI script is composed of "statements" made up of "commands".

A `#` creates a comment statement. Comments in SchnauzerUI are automatically added to test reports.
The `locate` command locates a WebElement in the most straightforward way possible. It begins with
aspects of the element that are __visible to the user__ (placeholder, text). This is important for a few reasons:

1. QA testers rarely need to go digging around in HTML to write tests, which greatly improves productivity.
2. Tests are more likely to survive a change in technology (for example, migrating JavaScript frameworks).
3. Tests are more representative of user experience (The user doesn't care about test_ids, they do care about placeholders).

Then, the `locate` command can default to more technology specific locators, in order to allow flexibility in
test authoring (id, name, title, class, xpath)

Once an element is in focus (i.e. located), any subsequent commands will be executed against it. Commands relating
to web elements include `click`, `type`, and `read-to` (a command for storing the text of a web element as a variable). The complete list of statements and commands lives [here](statements_and_commands.md)

### Smart Swap

To ensure point number 1, SchnauzerUI smart swaps elements for given commands. Locate select elements by just the visible text of the default option. Locate form inputs and textareas by their labels. This makes it dead simple perform complex UI interactions. 

## Error Handling
UI tests can be brittle. Sometimes you simply want to write a long
test flow (even when testing gurus tell you not too) without it bailing at the first slow page load. For this, SchnauzerUI
provides the `catch-error:` command for gracefully recovering from errors and resuming test runs:
```SchnauzerUI
...

# This page is quite slow to load, so we'll try again if something goes wrong
catch-error: screenshot and refresh and try-again

...
................
```
Here, the `catch-error:` command gives us the chance to reset the page by refreshing
and try the previous commands again without the test simply failing. The test "failure"
is still reported (and a screenshot is taken), but the rest of the test executes.
(Note: This does not risk getting caught in a loop. The `try-again` command will only re-execute
the same code once.)

You can write SchnauzerUI scripts directly in a .sui file, but the best way to write the script is using
the REPL. To learn the different ways to develop tests, see [the cli guide](cli.md)