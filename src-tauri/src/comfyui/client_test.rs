use super::*;
use serde_json::Value;

#[test]
fn test_parse_history_response() {
    let json: Value = serde_json::from_str(r#"{
        "abc123": {
            "status": {"status_str": "success", "completed": true},
            "outputs": {
                "9": {
                    "images": [
                        {"filename": "ComfyUI_00001_.png", "subfolder": "", "type": "output"}
                    ]
                }
            }
        }
    }"#).unwrap();

    let entry = json.get("abc123").unwrap();
    let status = entry.pointer("/status/status_str").and_then(|v| v.as_str());
    assert_eq!(status, Some("success"));

    let completed = entry.pointer("/status/completed").and_then(|v| v.as_bool());
    assert_eq!(completed, Some(true));

    let images = entry.pointer("/outputs/9/images").and_then(|v| v.as_array());
    assert!(images.is_some());
    assert_eq!(images.unwrap()[0]["filename"], "ComfyUI_00001_.png");
}

#[test]
fn test_parse_queue_response() {
    let json: Value = serde_json::from_str(r#"{
        "queue_running": [["item1"]],
        "queue_pending": [["item2"], ["item3"]]
    }"#).unwrap();

    let running = json.get("queue_running").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let pending = json.get("queue_pending").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    assert_eq!(running, 1);
    assert_eq!(pending, 2);
}

#[test]
fn test_parse_prompt_response() {
    let json: Value = serde_json::from_str(r#"{
        "prompt_id": "abc-123-def",
        "number": 1,
        "node_errors": {}
    }"#).unwrap();

    let prompt_id = json.get("prompt_id").and_then(|v| v.as_str());
    assert_eq!(prompt_id, Some("abc-123-def"));

    let errors = json.get("node_errors").and_then(|v| v.as_object());
    assert!(errors.unwrap().is_empty());
}

#[test]
fn test_image_ref_struct() {
    let img = ImageRef {
        filename: "test.png".to_string(),
        subfolder: "".to_string(),
        img_type: "output".to_string(),
    };
    assert_eq!(img.filename, "test.png");
}

#[test]
fn test_queue_status_serialization() {
    let status = QueueStatus { running: 1, pending: 3 };
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"running\":1"));
    assert!(json.contains("\"pending\":3"));
}
