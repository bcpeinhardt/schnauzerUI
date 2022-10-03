use schnauzer_ui::run_no_log;

#[tokio::test]
async fn health_check() {
    let script = r#"
    # Navigate to the test url
    url "http://localhost:1234"
    "#;

    let result = run_no_log(script.to_owned()).await;
    assert!(result.is_ok());
}
