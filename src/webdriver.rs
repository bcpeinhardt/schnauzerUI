use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::Deserialize;
///! This module contains code for working with `thirtyfour::WebDriver`s
use std::{collections::HashMap, fmt::Display};
use thirtyfour::{DesiredCapabilities, WebDriver};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, ValueEnum)]
pub enum SupportedBrowser {
    Firefox,
    Chrome,
}

impl Display for SupportedBrowser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedBrowser::Firefox => write!(f, "firefox"),
            SupportedBrowser::Chrome => write!(f, "chrome"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebDriverConfig {
    pub port: usize,
    pub headless: bool,
    pub browser: SupportedBrowser,
}

impl Default for WebDriverConfig {
    fn default() -> Self {
        Self {
            port: 4444,
            headless: false,
            browser: SupportedBrowser::Firefox,
        }
    }
}

pub async fn new_driver(
    WebDriverConfig {
        port,
        headless,
        browser,
    }: WebDriverConfig,
) -> Result<WebDriver> {
    let localhost = format!("http://localhost:{}", port);
    match browser {
        SupportedBrowser::Firefox => {
            let mut caps = DesiredCapabilities::firefox();
            if headless {
                caps.set_headless()?;
            }
            WebDriver::new(&localhost, caps)
                .await
                .context("Could not launch WebDriver")
        }
        SupportedBrowser::Chrome => {
            let mut caps = DesiredCapabilities::chrome();
            if headless {
                caps.set_headless()?;
            }
            caps.add_arg("--disable-infobars")?;
            caps.add_arg("start-maximized")?;
            caps.add_arg("--disable-extensions")?;
            let mut prefs = HashMap::new();
            prefs.insert("profile.default_content_setting_values.notifications", 1);
            caps.add_experimental_option("prefs", prefs)?;
            WebDriver::new(&localhost, caps)
                .await
                .context("Could not launch WebDriver")
        }
    }
}
