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

    let content = json.get("response").and_then(|c| c.as_str()).unwrap_or("");
    assert!(content.contains("Concept one"));
    assert!(content.contains("Concept two"));
}

#[test]
fn test_parse_error_response() {
    let json: Value = serde_json::from_str(r#"{"error": "model not found"}"#).unwrap();

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

// ========== Thinking model detection tests ==========

#[test]
fn test_known_thinking_models() {
    assert!(is_known_thinking_model("qwen3:8b"));
    assert!(is_known_thinking_model("qwen3:32b-q4_K_M"));
    assert!(is_known_thinking_model("deepseek-r1:7b"));
    assert!(is_known_thinking_model("deepseek-r1:1.5b"));
    assert!(is_known_thinking_model("qwq:latest"));
    assert!(is_known_thinking_model("phi4-reasoning:14b"));
    assert!(is_known_thinking_model("phi-4-reasoning:14b"));
    assert!(is_known_thinking_model("gpt-oss:latest"));
    assert!(is_known_thinking_model("marco-o1:7b"));
}

#[test]
fn test_non_thinking_models() {
    assert!(!is_known_thinking_model("mistral:7b"));
    assert!(!is_known_thinking_model("llama3.1:8b"));
    assert!(!is_known_thinking_model("qwen2.5:7b"));
    assert!(!is_known_thinking_model("llava:7b"));
    assert!(!is_known_thinking_model("codellama:13b"));
    assert!(!is_known_thinking_model("gemma2:9b"));
}

#[test]
fn test_thinking_model_case_insensitive() {
    assert!(is_known_thinking_model("Qwen3:8b"));
    assert!(is_known_thinking_model("DEEPSEEK-R1:7b"));
    assert!(is_known_thinking_model("QwQ:latest"));
}

#[test]
fn test_stage_options_with_thinking() {
    let opts = stage_options_with_thinking(1024, Some(false));
    assert_eq!(opts.think, Some(false));
    assert_eq!(opts.num_predict, Some(1024));

    let opts_default = stage_options_with_thinking(512, None);
    assert_eq!(opts_default.think, None);

    let opts_on = stage_options_with_thinking(2048, Some(true));
    assert_eq!(opts_on.think, Some(true));
    assert_eq!(opts_on.num_predict, Some(2048));
}

#[test]
fn test_think_param_not_in_build_options() {
    // think is a top-level param, not in "options" sub-object
    let opts = OllamaOptions {
        think: Some(false),
        ..Default::default()
    };
    let options = build_options(&opts);
    assert!(!options.contains_key("think"));
}

#[test]
fn test_stage_options_default_has_no_think() {
    let opts = stage_options(1024);
    assert_eq!(opts.think, None);
}
