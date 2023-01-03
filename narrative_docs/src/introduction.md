# Introduction

SchnauzerUI is a human readable DSL for performing automated UI testing in the browser.
The main goal of SchnauzerUI is to increase stakeholder visibility and participation in automated Quality Assurance testing. 

SchnauzerUI lets non-programmers create automated tests from start to finish, without
out ever needing to involve a developer. To demonstrate, lets look at the SchnauzerUI
code necessary to search for cats on youtube.

```
url "https://youtube.com"
locate "Search" and type "cats" and press "Enter"
```

This is the only code necessary to write in order to run the test in a live
browser and generate a test report. You can place this code in a file called `yt.sui`
and run the test with `sui -f ./yt.sui`. It will launch chrome, perform 
the search, and produce an html test report as well as json output. 

This works
because SchnauzerUI is also a "test framework", meaning it handles downloading, updating, launching,
and stopping webdrivers and generating test reports on it's own, so you don't have to.
The only code a user has to write is the actual tests, which are incredibly straightforward.


