//! Schnauzer UI is a DSL for performing browser automation with Selenium.
//! Rather than focusing on large automated test suites, Schnauzer UI helps
//! you write small, easy to understand test scripts.
//!
//! ```schnauzer_ui
//! # Navigate to the site
//! url "https://youtube.com"
//!
//! # Search for Cats
//! locate "Search" and type "cats" and press "Enter"
//! ```
//!
//! #### When to use Schnauzer UI vs. other options (Selenium, Cypress, Playwright, etc.)
//! Schnauzer UI is a bad choice if:
//! - You want to maintain a large automated test suite with reusable components.
//! - You have a team of full time software engineers dedicated to QA.
//! - Your QA and Engineering teams are able to collaborate closely to build software
//!   that is easy to E2E test (you have dedicated test environments, a system for test IDs for elements, etc.).
//! - You want to build your E2E test suites into CI/CD processes and want good tooling around doing it.
//!
//! Schnauzer UI is a good choice if:
//! - You want to be able to easily run stand alone scripts and produce a test report.
//! - You want to quickly draft tests as needed instead of maintaining a large test suite.
//! - You want a more robust way to codify Acceptance Criteria / Bug Reproduction Steps on your tickets.
//! - You want to empower non-programmer QA team members to take advantage of automation.
//! - You want to build some automation into existing manual process.
//!
//! To get started, check out the [narrative documentation](https://bcpeinhardt.github.io/schnauzerUI/)

pub mod datatable;
pub mod interpreter;
pub mod parser;
pub mod scanner;
pub mod test_report;
pub mod webdriver;

mod environment;
mod js;
