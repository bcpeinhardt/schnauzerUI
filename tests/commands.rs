use serial_test::serial;
mod common;
use crate::common::{run_script_against, run_script_against_fails};

#[tokio::test]
#[serial]
async fn type_into_input() {
    run_script_against(
        "locate \"input\" and type \"Some Text\" and chill \"1\"",
        "<input type=\"text\" />",
    )
    .await;
}

/// Temporarily ignoring this test because we need to find a compromise for typing even when elements
/// don't clear but erroring for elements that are obviously wrong, like p tags. This gets even more
/// complicated when you consider things like web components.
#[tokio::test]
#[serial]
#[ignore]
async fn type_into_input_errors_properly() {
    run_script_against_fails(
        "locate \"not-an-input\" and type \"Some Text\" and chill \"1\"",
        "<p id='not-an-input'>Cant type in me</p>",
    )
    .await;
}

#[tokio::test]
#[serial]
async fn click_button() {
    // Clicks a button causing it's text to change and then locates
    // the button element by its new text
    run_script_against(
        "locate \"Click Me\" and click and locate \"Clicked\"",
        "<button id='btn' onclick=\"function changeTxt(){
            document.querySelector('#btn').textContent = 'Clicked'
        };changeTxt();\">Click Me</button>",
    )
    .await;
}

#[tokio::test]
#[serial]
async fn read_to() {
    // Reads the text from a paragraph and uses it
    // to locate the same paragraph
    run_script_against(
        "locate \"the-answer\" and read-to theAnswer and locate theAnswer",
        "<p id='the-answer'>42</p>",
    )
    .await;
}

#[tokio::test]
#[serial]
async fn locate_no_scroll() {
    // Just making sure the command doesn't error. Haven't
    // thought of a good way to check the scroll with the current
    // testing setup yet
    run_script_against("locate \"input\"", "<input type=\"text\" />").await;
}

#[tokio::test]
#[serial]
async fn refresh() {
    // Clicks a button causing it's text to change, then refreshes the page
    // and verifies the refresh by locating the buttons old text
    run_script_against(
        "locate \"Click Me\" and click and locate \"Clicked\" and refresh and locate \"Click Me\"",
        "<button id='btn' onclick=\"function changeTxt(){
            document.querySelector('#btn').textContent = 'Clicked'
        };changeTxt();\">Click Me</button>",
    )
    .await;
}

#[tokio::test]
#[serial]
// #[ignore = "Test which rely on timeouts are ignored because they take so long. Feel free to run manually"]
async fn try_again() {
    run_script_against(
        "locate \"Clicked\"\n catch-error: locate \"Click Me\" and click and try-again",
        "<button id='btn' onclick=\"function changeTxt(){
            document.querySelector('#btn').textContent = 'Clicked'
        };changeTxt();\">Click Me</button>",
    )
    .await;

    run_script_against_fails(
        "locate \"Clicked\"\n catch-error: locate \"Click Me\" and click and try-again",
        "<button id='btn' onclick=\"function changeTxt(){
            document.querySelector('#btn').textContent = 'Wrong Content'
        };changeTxt();\">Click Me</button>",
    )
    .await;
}

#[tokio::test]
#[serial]
async fn accept_alert() {
    // Clicks a button causing it's text to change, then refreshes the page
    // and verifies the refresh by locating the buttons old text
    run_script_against(
        "locate \"Click Me\" and click and accept-alert",
        "<button id='btn' onclick=\"function doAlert(){
            alert('I am an alert');
        };doAlert();\">Click Me</button>",
    )
    .await;
}

#[tokio::test]
#[serial]
async fn dismiss_alert() {
    // Clicks a button causing it's text to change, then refreshes the page
    // and verifies the refresh by locating the buttons old text
    run_script_against(
        "locate \"Click Me\" and click and dismiss-alert",
        "<button id='btn' onclick=\"function doAlert(){
            alert('I am an alert');
        };doAlert();\">Click Me</button>",
    )
    .await;
}

#[tokio::test]
#[serial]
async fn upload() {
    // Clicks a button causing it's text to change, then refreshes the page
    // and verifies the refresh by locating the buttons old text
    run_script_against(
        "locate \"myfile\" and upload \"./tests/assets/test_file_for_upload.txt\"",
        "<input type=\"file\" id=\"myfile\" name=\"myfile\">",
    )
    .await;
}
