use serial_test::serial;
mod common;
use crate::common::run_script_against;


#[tokio::test]
#[serial]
async fn locate_by_text() { 
    run_script_against(
        "locate \"Text\"",
        "<h1>Text</h1>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_by_partial_text() { 
    run_script_against(
        "locate \"Te\"",
        "<h1>Text</h1>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_by_placeholder() { 
    run_script_against(
        "locate \"A Placeholder\"",
        "<input type=\"text\" placeholder=\"A Placeholder\"></input>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_by_partial_placeholder() { 
    run_script_against(
        "locate \"A Place\"",
        "<input type=\"text\" placeholder=\"A Placeholder\"></input>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_by_title() { 
    run_script_against(
        "locate \"test title\"",
        "<h1 title=\"test title\">Text</h1>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_by_aria_label() { 
    run_script_against(
        "locate \"test label\"",
        "<h1 aria-label=\"test label\">Text</h1>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_by_id() { 
    run_script_against(
        "locate \"test-id\"",
        "<h1 id=\"test-id\">Text</h1>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_by_name() { 
    run_script_against(
        "locate \"test-name\"",
        "<h1 name=\"test-name\">Text</h1>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_by_class() { 
    run_script_against(
        "locate \"test-class\"",
        "<h1 class=\"test-class some-other-stuff\">Text</h1>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_by_tag_name() { 
    run_script_against(
        "locate \"h1\"",
        "<h1>Text</h1>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_by_xpath() { 
    run_script_against(
        "locate \"//h1[@name='test-name']\"",
        "<h1 name=\"test-name\">Text</h1>"
    ).await;
}