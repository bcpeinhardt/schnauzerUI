use schnauzer_ui::run;

#[tokio::test]
pub async fn if_stmt() {
    let script = r#"
    url "http://localhost:1234/if_stmt.html"

    # Try typing into a non-existant element
    if locate "I dont exist" then type "I shouldnt be typing this"

    # Now type into an existing element
    if locate "Type Here" then type "Woohoo"
    "#;

    let result = run(script.to_owned()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}