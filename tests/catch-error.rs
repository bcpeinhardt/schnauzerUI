use schnauzer_ui::run;

#[tokio::test]
async fn bad_test_errors() {
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

    let result = run(script.to_owned()).await;
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

    let result = run(script.to_owned()).await;
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

    let result = run(script.to_owned()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}