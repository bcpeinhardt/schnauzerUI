# Statements

*Don't see functionality you were looking for? File an issue on our [github](https://github.com/bcpeinhardt/schnauzerUI/issues/new) and we'll consider implementing it for you. Or feel free to file a pull request.*

### Comment
A comment statement is the simplest statement in Schnauzer UI. Create a comment by typing `#` and then the text you'd like.
It doesn't do anything except help keep scripts organized. Comments are automatically added to the test run log for reference. It's good practice to separate chuncks of a Schnauzer UI script with comments to keep it readable.

Ex. Commenting a chunk of code.
`# This part of the script performs x action ...`

For programmers: Schnauzer UI doesn't have units of code that can be extracted and reused like "functions".
This makes good comments really important. You can maintain large SchnauzerUI projects and achieve code reusability
with things like inlining, but it's a priority that each script be a straightforward linear description
of a browser based process, which can be easily passed around via tools like Jira, Teams, Slack, etc.

### Save As
The save as command will save some text as a variable for use later in the script. 

Ex. Begin a script with some important info

`save "test@test.com" as username`

### Command Statement
A command statement consists of a one or more commands connected by the `and` keyword. This is 
the bread and butter of your scripts.

Ex. Locate and click a button
`locate "Submit" and click`

### If Statement
An if statement is used for conditional actions. The statement takes a command as a predicate, and
executes the body only if the command succeeds without error.

Ex. Dismissing a popup.
`if locate "Confirm" then click`

### Catch Error
Schnauzer UI provides the `catch-error` statement for simple error handling. Whenever a command
produces an error (except in an if condition), the script will jump ahead to the nearest `catch-error:`
token. This lets you handle an error how you want. Some commands to key in mind are `screenshot`, `refresh`, and `try-again`.
If you don't make use of `catch-error`, scripts will simply exit when they encounter errors (which is good for a lot
of testing use cases but not good for RPA use cases).

Ex. Taking a screenshot from a script that reproduces a bug.
`catch-error: screenshot`

# Commands

### url
The `url` command navigates to a spcific url. It can take either a full url or a slug as an argument

Ex. Navigate to complete url

`url "https://youtube.com"`

Ex. Navigate to a slug /posts

`url "posts"`

### locate
The `locate` command finds a web element and scrolls it to the top of the viewport to interact with.

Ex. Locate a submit button

`locate "Submit"`

The locate command uses precedence to determine how to use the provided locator.

- If there is an input element whose placeholder matches the provided value, will locate the input
- If there is a label which precedes an input whose text matches the provided value, will locate the input
- If there is an element whose text matches the provided value, will locate the element
- If there is an element whose text contains the provided value, will locate the element
- If there is an element whose title matches the provided value, will locate the element 
- If there is an element whose id matches the provided value, will locate the element
- If there is an element whose name matches the provided value, will locate the element
- If there is an element which has a class that matches the provided value, will locate the element
- If none of these locate the element, the provided value will be used as an xpath to locate the element.

### locate-no-scroll
The `locate-no-scroll` command is the same as the locate command, but does not scroll the element from where
it is. Useful for when scrolling an element to the top of the viewport causes it to be covered by a navbar or 
some other element.

### click
The `click` command __performs a click at the location of the located element__. This helps to avoid 
click intercept issues with complex components. 

Ex. Locate a button and click it

`locate "login-btn" and click`

### type
The `type` command will send text to the located element.

Ex. Type in a username

`locate "Username" and type "test@test.com"`

### refresh
The `refresh` command simply refreshes the page

### screenshot
The `screenshot` command will capture a screeshot of the current window

### read-to
The `read-to` command will save the text of a web element to a variable. Useful for things
like dynamically generated names etc.

Ex. Read the number of search results to a variable

`locate "result-stats" and read-to mySearchResults`
 
### press
The `press` command is used to perform keyboard actions. The kepresses are registered against 
the currently selected web element, so it's mainly useful for things like hitting Enter from a search box.

Ex. Press enter when logging in.

`locate "password" and type myPassword and press "Enter"`

### chill
The `chill` command causes the script to pause for the provided number of seconds. Useful for waiting
for some process to finish. 
(Note: Commands by default have a one second wait between execution. Explicitly managing waits is complicated,
and we opted for a simpler approach. Generally this command will not be necessary. If you are waiting for some transition
on the page to take place, consider using the `locate` command to automatically wait for an element to signal the page is ready.
For example, after logging into a website, rather than using the `chill` command, use `locate` to find some element of the loaded dashboard to verify that the page has loaded.)

Ex. Wait 10 seconds.

`chill "10"`