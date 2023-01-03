# Say goodbye to maintaining huge automated test suites. <br>

 *Maintaining huge automated E2E test suites comes with enormous complexity.
 Eventually, engineers spend more time maintaining the tests and the framework
 than they do testing new functionality. If you work on an automated
 test suite that provides more complexity than value, SchnauzerUI may be a good
 choice for you.* <br><br>

 *SchnauzerUI takes a radically different approach to automated UI testing than 
 other frameworks. It is designed to empower manual testers to begin doing automated
 testing without learning to code or overhauling their existing
 testing processes. It achieves this in a few ways:* <br><br>

  - *SchnauzerUI is, first and foremost, a DSL (Domain Specific Language) for web automation.
  Interacting with websites programmatically is a very specific task, and testers shouldn't
  have to know Java or Python to perform it. The DSL is as simple as possible, and lets users
  opt-in to complexity when required. Smart locators and automatic logging are examples of this philosophy.*
  - *SchnauzerUI is also a testing framework. Things like webdriver management, report generation,
  and csv datatable support are built into the CLI. The only SchnauzerUI code a user ever writes is 
  for a test.*
  - *SchnauzerUI scripts are completely standalone. Manual testers can paste them in testpad,
    slack, jira, teams, etc. Changing a script will never break another script. This point is
    fairly contentious, and counter to the instincts of software engineers. But the reality is 
    that most automated test suites are too large and too brittle to provide an effective feedback
    loop for development. SchnauzerUI doesn't attempt to turn automated testing into a 
    software project, because it's complexity averse. SchnauzerUI empowers manual testers to automate 
    existing processes in place. Using testpad? Provide a SchnauzerUI script for each test case, and 
    only inspect the ones that break. Perform manual testing on new features as part of sprints?
    Use SchanuzerUI to create bug reproductions, so anyone with the ticket and SchnauzerUI installed
    can instantly understand the issue.*

 *What makes SchnauzerUI so productive?*
-  *REPL driven development brings exploratory and automated testing together*
-  *Easily inline existing scripts to achieve reusability while creating a script that runs by itself*
-  *Smart locators empower you to think in terms of the UI instead of the HTML*
-  *Scripts are based on information visible to the user, so they're likely to survive a framework migration*
-  *Launch a browser and start automating with a single command*<br><br>

 *To learn more, check out the [narrative_docs](https://bcpeinhardt.github.io/schnauzerUI/)*

