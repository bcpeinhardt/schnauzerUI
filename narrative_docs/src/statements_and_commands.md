# Statements

*Don't see functionality you were looking for? File an issue on our [github](https://github.com/bcpeinhardt/schnauzerUI/issues/new) and we'll consider implementing it for you. Or feel free to file a pull request.*

### Comment
A comment statement is the simplest statement in Schnauzer UI. Create a comment by typing `#` and then the text you'd like.
It doesn't do anything except help keep scripts organized. Comments are automatically added to the test run log for reference. It's good practice to separate chunks of a Schnauzer UI script with comments to keep it readable.

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
token. This lets you handle an error how you want. Some commands to keep in mind are `screenshot`, `refresh`, and `try-again`.
If you don't make use of `catch-error`, scripts will simply exit when they encounter errors (which is good for a lot
of testing use cases but not good for RPA use cases).

Ex. Taking a screenshot from a script that reproduces a bug.

`catch-error: screenshot`

### Under
An under statement changes the way locators work for a single line of code. It lets the 
locator start searching for html by radiating out from a given element rather than starting
at the top of the HTML document. It's especially useful for locating elements by visual text when
that text is present multiple places on the page. 
The scope of the search is not restricted to elements contained by the located element. Locators
will still search the whole page, but they will start searching where you tell them. Consider the
following HTML
```
<h1>Desired Text</h1>
<div class="container">
    <div class="nav-title">
        <h3>Navigation</h3>
    </div>
    <div class="nav-body">
        <a href="https://somesite.com">Desired Text</a>
        ... more links
    </div>
</div>
```
the command
```
under "Navigation" locate "Desired Text" and click
```
will click the navigation link, not the level one heading. This is not because the link is "under" the h3 element,
but because they are close together. The locate command searches children of the h3, then
children of the div with class nav-title, then children of the div with class container before succeeding.

### Under Active Element
Works the same as the Under statement, but begins searching under the "active" element (the last
element interacted with).

Ex. Locate an element near the last element interacted with.

`under-active-element locate subElement and click`

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

- Match a placeholder or partial placeholder
- Match text or partial text
- Match title attribute
- Match aria-label
- Match id attribute
- Match name attribute
- Match class attribute or partial class attribute
- Match an XPath

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
In general, the `type` command will send text to the located element.
In reality, the `type` command will click a located element then begin typing
in the active element! This is usually the same thing, but better for complex UI components
which switch the active element on their own.

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

### drag-to
The `drag-to` command uses javascript to simulate a drag and drop event, dragging the currently located
element to the element matching the provided locator.

Ex. Imagine mapping headers to the correct column of an uploaded file.

`locate "email" and drag-to "@"`

### select
The `select` command will select (by text) one of the options in a select element.
Note. This command will also work if the currently located element is an option in the given
select element. This makes it much easier to locate and use select elements using only the displayed option
text. 

Ex. Login as an admin user.

`locate "Select Role" and select "Admin User"`

### upload
The `upload` command performs a basic file upload on an html input element of type file.
The located element must be an `input` element. Very often, custom component is used which performs 
the javascript to do the upload, which means you'll have to look in the html for the hidden input you 
actually want to upload to.

Ex. Upload file

`locate "//input[@id='file-input']" and upload "./screenshots/main_screenshot_2.png"`

Note: We are locating the input by xpath because it's not actually displayed on the page.
Smart locators only return elements which are currently displayed.

### accept-alert and dismiss-alert
The `accept-alert` and `dismiss-alert` commands accept and dismiss alerts. 

Ex. Accept cookie alert
```
# Accept cookie alert
accept-alert
```
