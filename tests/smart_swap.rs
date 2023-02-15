use serial_test::serial;
mod common;
use crate::common::run_script_against;

#[tokio::test]
#[serial]
async fn locate_text_input_by_preceding_label() { 
    run_script_against(
        "locate \"test input element\" and type \"Some Text\" and chill \"1\"",
        "<label>test input element</label><input type=\"text\" />"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_text_area_by_preceding_label() { 
    run_script_against(
        "locate \"test textarea element\" and type \"Some Text\" and chill \"1\"",
        "<label>test textarea element</label><textarea></textarea>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_select_by_preceding_label() { 
    run_script_against(
        "locate \"test select element\" and select \"Option 2\" and chill \"1\"",
        "<label>test select element</label><select><option>Option 1</option><option>Option 2</option></select>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_text_input_by_label_with_for_attribute_to_id() { 
    run_script_against(
        "locate \"test input element\" and type \"Some Text\" and chill \"1\"",
        "<label for=\"test-input\">test input element</label><input id=\"test-input\" type=\"text\" />"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_text_input_by_label_with_for_attribute_to_name() { 
    run_script_against(
        "locate \"test input element\" and type \"Some Text\" and chill \"1\"",
        "<label for=\"test-input\">test input element</label><input name=\"test-input\" type=\"text\" />"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_text_input_by_containing_label() { 
    run_script_against(
        "locate \"test input element\" and type \"Some Text\" and chill \"1\"",
        "<label>test input element <input type=\"text\" /></label>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_text_area_by_containing_label() { 
    run_script_against(
        "locate \"test textarea element\" and type \"Some Text\" and chill \"1\"",
        "<label>test textarea element <textarea></textarea></label>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_select_by_containing_label() { 
    run_script_against(
        "locate \"test select element\" and select \"Option 2\" and chill \"1\"",
        "<label>test select element <select><option>Option 1</option><option>Option 2</option></select></label>"
    ).await;
}

#[tokio::test]
#[serial]
async fn locate_text_input_by_nested_label() { 
    run_script_against(
        "locate \"test input element\" and type \"Some Text\" and chill \"1\"",
        "<div><label>test input element</label></div><div><input type=\"text\" /></div>"
    ).await;
}