//! Tests that correspond to example SchnauzerUI code provided in the readme.

use schnauzer_ui::run;

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

    let result = run(script.to_owned()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}