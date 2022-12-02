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
A SchnauzerUI script is composed of "statements" made up of "commands" that execute on top of running Selenium webdrivers.
A `#` creates a comment statement. Comments in SchnauzerUI are automatically added to test reports.
The `locate` command locates a WebElement in the most straightforward way possible. It begins with
aspects of the element that are __visible to the user__ (placeholder, adjacent label, text). This is important for a few reasons:

1. QA testers rarely need to go digging around in HTML to write tests, which greatly improves productivity.
2. Tests are more likely to survive a change in technology (for example, migrating JavaScript frameworks).
3. Tests are more representative of user experience (The user doesn't care about test_ids, they do care about placeholders).
Then, the `locate` command can default to more technology specific locators, in order to allow flexibility in
test authoring (id, name, title, class, xpath)

Once an element is in focus (i.e. located), any subsequent commands will be executed against it. Commands relating
to web elements include `click`, `type`, and `read-to` (a command for storing the text of a web element as a variable).
The complete lis of statements and commands lives [here](statements_and_commands.md)

## Error Handling
UI tests can be brittle. Sometimes you simply want to write a long
test flow (even when testing gurus tell you not too) without it bailing at the first slow page load. For this, SchnauzerUI
provides the `catch-error:` command for gracefully recovering from errors and resuming test runs. We can improve the
previous test example like so
```SchnauzerUI
# Type in username (located by labels)
locate "Username" and type "test@test.com"

# Type in password (located by placeholder)
locate "Password" and type "Password123!"

# Click the submit button (located by element text)
locate "Submit" and click

# This page is quite slow to load, so we'll try again if something goes wrong
catch-error: screenshot and refresh and try-again
................
```
Here, the `catch-error:` command gives us the chance to reset the page by refreshing
and try the previous commands again without the test simply failing. The test "failure"
is still reported (and a screenshot is taken), but the rest of the test executes.
(Note: This does not risk getting caught in a loop. The `try-again` command will only re-execute
the same code once.)