use schnauzer_ui::run_no_log;

#[tokio::test]
async fn manually_set_locator_for_elem() {
    let script = r#"
    url "http://localhost:1234/variables.html"

    save myPlaceholder as "I show you where to type"

    locate myPlaceholder and type "Found ya"
    "#;

    let result = run_no_log(script.to_owned()).await;
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

    let result = run_no_log(script.to_owned()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}
