//! Tests that correspond to example SchnauzerUI code provided in the readme.

use schnauzer_ui::run_no_log;

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

    let result = run_no_log(script.to_owned()).await;
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

    let result = run_no_log(script.to_owned()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}
