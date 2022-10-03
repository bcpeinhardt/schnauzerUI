use schnauzer_ui::run;

#[tokio::test]
pub async fn if_stmt() {
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

    let result = run(script.to_owned()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}