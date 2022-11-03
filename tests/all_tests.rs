use async_once::AsyncOnce;
use lazy_static::lazy_static;
use schnauzer_ui::{new_driver, run_no_log, SupportedBrowser, WebDriverConfig};
use thirtyfour::WebDriver;

// Set up the DriverConfig
const DRIVER_CONFIG: WebDriverConfig = WebDriverConfig {
    port: 4444,
    headless: false,
    browser: SupportedBrowser::FireFox,
};

async fn get_test_driver() -> WebDriver {
    new_driver(DRIVER_CONFIG).await.unwrap()
}

// Health Check ------------------------------------------------------------------------------------

#[tokio::test]
async fn health_check() {
    let script = r#"
    # Navigate to the test url
    url "http://localhost:1234"
    "#;

    let result = run_no_log(script.to_owned(), get_test_driver().await).await;
    assert!(result.is_ok());
}

// Locate ------------------------------------------------------------------------------------------

#[tokio::test]
pub async fn locate() {
    let script = r#"
    url "http://localhost:1234/locate.html"

    locate "locate-by-placeholder"
    locate "locate-by-preceding-label"
    locate "locate-by-text"
    locate "locate-by-id"
    locate "locate-by-name"
    locate "locate-by-title"
    locate "locate-by-class"
    locate "//input[@name='locate-by-name']/../p"
    "#;

    let result = run_no_log(script.to_owned(), get_test_driver().await).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

// Variables ---------------------------------------------------------------------------------------

#[tokio::test]
async fn manually_set_locator_for_elem() {
    let script = r#"
    url "http://localhost:1234/variables.html"

    save myPlaceholder as "I show you where to type"

    locate myPlaceholder and type "Found ya"
    "#;

    let result = run_no_log(script.to_owned(), get_test_driver().await).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

#[tokio::test]
async fn read_locator_from_elem() {
    let script = r#"
    url "http://localhost:1234/variables.html"

    locate "locate-me" and read-to myPlaceholder

    locate myPlaceholder and type "Found ya"
    "#;

    let result = run_no_log(script.to_owned(), get_test_driver().await).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

// Error handling tests ----------------------------------------------------------------------------

#[tokio::test]
async fn catch_error_does_not_get_stuck_in_loop() {
    let script = r#"# Navigate to the test url
    url "http://localhost:1234/login.html"
    
    # Type in username (located by labels)
    locate "Username" and type "test@test.com"
    
    catch-error: screenshot
    
    # Type in password (located by placeholder)
    locate "Passwodr" and type "Password123!"
    
    # Click the submit button (located by element text)
    locate "Submit" and click 
    
    # Handle errors
    catch-error: screenshot and refresh and try-again"#;

    let result = run_no_log(script.to_owned(), get_test_driver().await).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[tokio::test]
async fn good_test_does_not_error() {
    let script = r#"
    # Navigate to the test url
    url "http://localhost:1234/login.html"
    
    # Type in username (located by labels)
    locate "Username" and type "test@test.com"
    "#;

    let result = run_no_log(script.to_owned(), get_test_driver().await).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

#[tokio::test]
async fn exit_early_no_catch_error_stmt_correctly_indicates_early_return() {
    let script = r#"
    # Navigate to the test url
    url "http://localhost:1234/login.html"
    
    # Type in username (located by labels)
    locate "Im not here" and type "test@test.com"
    "#;

    let result = run_no_log(script.to_owned(), get_test_driver().await).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

// If Stmt -------------------------------------------------------------------------------------------------

#[tokio::test]
pub async fn if_stmt() {
    let script = r#"
    url "http://localhost:1234/if_stmt.html"

    # Try typing into a non-existant element
    if locate "I dont exist" then type "I shouldnt be typing this"

    # Now type into an existing element
    if locate "Type Here" then type "Woohoo"
    "#;

    let result = run_no_log(script.to_owned(), get_test_driver().await).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

// Read Me tests (example code from readme) -----------------------------------------------------------------

#[tokio::test]
async fn basic_example() {
    let script = r#"
    url "http://localhost:1234/login.html"

    # Type in username (located by labels)
    locate "Username" and type "test@test.com"

    # Type in password (located by placeholder)
    locate "Password" and type "Password123!"

    # Click the submit button (located by element text)
    locate "Submit" and click
    "#;

    let result = run_no_log(script.to_owned(), get_test_driver().await).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

#[tokio::test]
async fn error_handling_example() {
    let script = r#"
    url "http://localhost:1234/login.html"

    # Type in username (located by labels)
    locate "Username" and type "test@test.com"

    # Type in password (located by placeholder)
    locate "Password" and type "Password123!"

    # Click the submit button (located by element text)
    locate "Submit" and click

    # This page is quite slow to load, so we'll try again if something goes wrong
    catch-error: screenshot and refresh and try-again
    "#;

    let result = run_no_log(script.to_owned(), get_test_driver().await).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}
