use schnauzer_ui::run_no_log;

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

    let result = run_no_log(script.to_owned()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}
