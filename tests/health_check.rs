use schnauzer_ui::run;

#[tokio::test]
async fn health_check() { 
    let script = r#"
    # Navigate to the test url
    url "http://localhost:1234"
    "#;

    let result = run(script.to_owned()).await;
    assert!(result.is_ok());
}
