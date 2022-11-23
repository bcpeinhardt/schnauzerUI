# Commands

*Don't see a command you were looking for? File an issue on our [github](https://github.com/bcpeinhardt/schnauzerUI/issues/new)
and we'll try to implement it for you!*

### url
The `url` command navigates to a spcific url. It can take either a full url or a slug as an argument

Ex. Navigate to complete url

`url "https://youtube.com"`

Ex. Navigate to a slug /posts

`url "posts"`

### locate
The `locate` command finds a web element and makes it available to interact with.

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

### save as
The save as command will save some text as a variable for use later in the script. 

Ex. Begin a script with some important info

`save "test@test.com" as username`