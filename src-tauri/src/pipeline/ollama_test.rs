use super::*;

#[test]
fn test_chat_message_serialization() {
    let msg = ChatMessage {
        role: "system".to_string(),
        content: "You are a helper.".to_string(),
    };
    let json = serde_json::to_value(&msg).unwrap();
    assert_eq!(json["role"], "system");
    assert_eq!(json["content"], "You are a helper.");
}

#[test]
fn test_parse_chat_response() {
    let json: Value = serde_json::from_str(
        r#"{
            "model": "mistral:7b",
            "message": {"role": "assistant", "content": "Hello world"},
            "done": true,
            "total_duration": 5000000000,
            "prompt_eval_count": 42,
            "eval_count": 10
        }"#,
    )
    .unwrap();

    let content = json
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("");
    assert_eq!(content, "Hello world");
    assert_eq!(
        json.get("total_duration").and_then(|v| v.as_u64()),
        Some(5000000000)
    );
}

#[test]
fn test_parse_generate_response() {
    let json: Value = serde_json::from_str(
        r#"{
            "model": "mistral:7b",
            "response": "1. Concept one\n2. Concept two",
            "done": true,
            "total_duration": 3000000000,
            "prompt_eval_count": 50,
            "eval_count": 200
        }"#,
    )
    .unwrap();

    let content = json
        .get("response")
        .and_then(|c| c.as_str())
        .unwrap_or("");
    assert!(content.contains("Concept one"));
    assert!(content.contains("Concept two"));
}

#[test]
fn test_parse_error_response() {
    let json: Value =
        serde_json::from_str(r#"{"error": "model not found"}"#).unwrap();

    let error = json.get("error").and_then(|v| v.as_str());
    assert_eq!(error, Some("model not found"));
}

#[test]
fn test_parse_models_response() {
    let json: Value = serde_json::from_str(
        r#"{
            "models": [
                {"name": "mistral:7b", "size": 4000000000, "digest": "abc123"},
                {"name": "llama3.1:8b", "size": 5000000000, "digest": "def456"}
            ]
        }"#,
    )
    .unwrap();

    let models: Vec<OllamaModel> = json
        .get("models")
        .and_then(|m| m.as_array())
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|m| {
            let name = m.get("name")?.as_str()?.to_string();
            let size = m.get("size").and_then(|s| s.as_u64());
            let digest = m.get("digest").and_then(|d| d.as_str().map(String::from));
            Some(OllamaModel { name, size, digest })
        })
        .collect();

    assert_eq!(models.len(), 2);
    assert_eq!(models[0].name, "mistral:7b");
    assert_eq!(models[1].name, "llama3.1:8b");
}
