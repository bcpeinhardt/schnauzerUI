use serial_test::serial;
mod common;
use crate::common::run_script_against;

#[tokio::test]
#[serial]
async fn if_stmt_should_execute() { 
    run_script_against(
        "if locate \"some-elm\" then type \"Some Text\" and chill \"1\"",
        "<input id=\"some-elm\" type=\"text\" />"
    ).await;
}

#[tokio::test]
#[serial]
async fn if_stmt_should_not_execute() { 
    run_script_against(
        "if locate \"some-elm\" then type \"Some Text\" and chill \"1\"",
        "<input type=\"text\" />"
    ).await;
}

#[tokio::test]
#[serial]
async fn comment() { 
    run_script_against(
        "# I'm just a comment baby\nlocate \"some-elm\"",
        "<input id=\"some-elm\" type=\"text\" />"
    ).await;
}

#[tokio::test]
#[serial]
async fn save_as() { 
    run_script_against(
        "save \"some-elm\" as myLocator\nlocate myLocator and type \"Some Text\" and chill \"1\"",
        "<input id=\"some-elm\" type=\"text\" />"
    ).await;
}

#[tokio::test]
#[serial]
async fn under() { 
    run_script_against(
        "under \"haystack\" locate \"some-elm\" and type \"Some Text\" and chill \"1\"",
        "<p id='some-elm'>No type here</p><div id='haystack'><input class=\"some-elm\" type=\"text\" /></div>"
    ).await;
}

#[tokio::test]
#[serial]
async fn under_active_element() { 
    run_script_against(
        "locate \"Click me\" and click\nunder-active-element locate \"some-elm\" and type \"Some Text\" and chill \"1\"",
        "<p id='some-elm'>No type here</p><div id='haystack'><input class=\"some-elm\" type=\"text\" /><button>Click me</button></div>"
    ).await;
}