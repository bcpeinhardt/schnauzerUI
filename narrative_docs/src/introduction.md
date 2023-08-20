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
browser and generate a test report.

## About this document

We are subscribing to the Divio documentation system, so this document is composed of four sections:
1. **Tutorials**: Focused on learning SchnauzerUI. Hands on and closely guided.
2. **How-To Guides**: Focused on solving specific problems with SchnauzerUI. We hope to accumulate a lot of these, so don't be afraid to submit a Github issue asking for help, it helps us as much as it does you.
3. **Reference**: A complete list of all available functionality of SchnauzerUI.
4. **Explanation**: Covers how SchanzuerUI works under the hood and why it was designed that way. Great place to start if you're considering forking or contributing to SchnauzerUI.

To get started, checkout the [quickstart tutorial](tutorials/quickstart.md)


