# Introduction

SchnauzerUI is a human readable DSL for performing automated UI testing in the browser. 
The main goal of SchnauzerUI is to increase stakeholder visibility and participation in automated Quality Assurance testing. 
Rather than providing a shim to underling code written by a QA engineer (see Cucumber), SchnauzerUI is the only source of truth for a test's execution. 
In this way, SchnauzerUI aims to provide a test report you can trust.

Feel free to skip ahead to [The Basics](basics.md), or keep reading to learn the why.

## Motivation

QA Departments today tend to operate as two separate teams with two separate missions.

There are the manual testers, who generally test features that are actively coming out (i.e. work in the current dev sprint). 
They usually have deep knowledge of the system under test and good working relationships with the development team. 
They probably have pretty solid technical chops, or at least are comfortable using API and Database testing tools, and are excellent written communicators. 
The really good ones will also have an interest in security. 
They have a personal process that works for them, and there's a good chance they have processes and domain knowledge 
that are not documented in the fancy team wiki. In general, they are overworked and under valued.

Then there are the automation testers. 
They typically work in sprints like the development team, incorporating a backlog of smoke and regression tests into automated test frameworks, 
which are (theoretically) used as part of automated testing and deployment processes. 
The automated testing suite is generally a real software project, 
sometimes just as complex as one of the companies products, maintained by real(ly expensive) engineers, 
and reviewed by no one because the rest of the engineers are busy building products. 
There's a good chance they're working in a feature factory style, 
in a project that probably includes API and Database testing that doesn't play nice with the companies CI/CD pipeline, 
and is plagued by scope creep. Bonus points if the project was vendored and the vendor hardly communicates with in house employees.

Our thesis is basically this:

1. Complicated whitebox E2E testing is an engineering task, so do it during the development process and 
    get real buy in and participation from the development team. You might even be using a web framework with built in support for E2E testing.

2. Automated black box "functional" E2E testing is a straight forward enough task to be carried out by "manual" QA testers, 
    with the right tools. There is absolutely no reason a person should need to know Java to verify that a button works. 
    These tools should be open source and sensitive test data (for example, logins) should live on prem or in private repos, 
    not $900/month and run on some other companies infrastructure so that you can be overcharged for compute and sued when they get hacked.

SchnauzerUI aims to be the right tool for point number two. 
It's a human readable DSL (Domain Specific Language) for writing UI tests, that any manual QA tester can quickly pick up 
(without having to become a programmer).
