use super::*;

#[test]
fn test_parse_numbered_list_basic() {
    let text = "1. First concept here.\n2. Second concept here.\n3. Third concept.";
    let result = parse_numbered_list(text);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], "First concept here.");
    assert_eq!(result[1], "Second concept here.");
    assert_eq!(result[2], "Third concept.");
}

#[test]
fn test_parse_numbered_list_multiline() {
    let text =
        "1. First concept starts here\nand continues on next line.\n2. Second concept.";
    let result = parse_numbered_list(text);
    assert_eq!(result.len(), 2);
    assert!(result[0].contains("continues on next line"));
}

#[test]
fn test_parse_numbered_list_parenthesis_format() {
    let text = "1) First concept.\n2) Second concept.\n3) Third concept.";
    let result = parse_numbered_list(text);
    assert_eq!(result.len(), 3);
}

#[test]
fn test_parse_numbered_list_empty() {
    let result = parse_numbered_list("");
    assert!(result.is_empty());
}

#[test]
fn test_parse_judge_rankings_valid() {
    let json = r#"[
        {"rank": 1, "concept_index": 3, "score": 92, "reasoning": "Best composition"},
        {"rank": 2, "concept_index": 0, "score": 87, "reasoning": "Good lighting"}
    ]"#;
    let result = parse_judge_rankings(json).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].rank, 1);
    assert_eq!(result[0].concept_index, 3);
    assert_eq!(result[0].score, 92);
    assert_eq!(result[0].reasoning, "Best composition");
}

#[test]
fn test_parse_judge_rankings_with_surrounding_text() {
    let text = "Here are my rankings:\n[{\"rank\":1,\"concept_index\":0,\"score\":90,\"reasoning\":\"Good\"}]\nThats my assessment.";
    let result = parse_judge_rankings(text).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].rank, 1);
}

#[test]
fn test_parse_judge_rankings_invalid() {
    let result = parse_judge_rankings("This is not JSON at all");
    assert!(result.is_err());
}

#[test]
fn test_parse_prompt_pair_valid() {
    let json =
        r#"{"positive": "masterpiece, best quality, cat", "negative": "lowres, blurry"}"#;
    let result = parse_prompt_pair(json).unwrap();
    assert_eq!(result.positive, "masterpiece, best quality, cat");
    assert_eq!(result.negative, "lowres, blurry");
}

#[test]
fn test_parse_prompt_pair_with_surrounding_text() {
    let text = "Here is the prompt:\n{\"positive\": \"a cat\", \"negative\": \"bad\"}\nDone.";
    let result = parse_prompt_pair(text).unwrap();
    assert_eq!(result.positive, "a cat");
    assert_eq!(result.negative, "bad");
}

#[test]
fn test_parse_prompt_pair_missing_field() {
    let json = r#"{"positive": "a cat"}"#;
    let result = parse_prompt_pair(json);
    assert!(result.is_err());
}

#[test]
fn test_parse_reviewer_approved() {
    let json = r#"{"approved": true}"#;
    let result = parse_reviewer_output(json).unwrap();
    assert!(result.approved);
    assert!(result.issues.is_none());
}

#[test]
fn test_parse_reviewer_not_approved() {
    let json = r#"{
        "approved": false,
        "issues": ["prompt drift", "token bloat"],
        "suggested_positive": "better prompt",
        "suggested_negative": "better neg"
    }"#;
    let result = parse_reviewer_output(json).unwrap();
    assert!(!result.approved);
    assert_eq!(result.issues.as_ref().unwrap().len(), 2);
    assert_eq!(result.suggested_positive.as_deref(), Some("better prompt"));
}

#[test]
fn test_extract_json_direct() {
    let json = r#"{"key": "value"}"#;
    let result = extract_json_from_text(json).unwrap();
    assert_eq!(result["key"], "value");
}

#[test]
fn test_extract_json_with_surrounding_text() {
    let text = "Here is the result:\n{\"key\": \"value\"}\nEnd of response.";
    let result = extract_json_from_text(text).unwrap();
    assert_eq!(result["key"], "value");
}

#[test]
fn test_extract_json_array() {
    let text = "Rankings: [{\"rank\": 1}]";
    let result = extract_json_from_text(text).unwrap();
    assert!(result.is_array());
}

#[test]
fn test_extract_json_no_json() {
    let result = extract_json_from_text("No JSON here at all");
    assert!(result.is_err());
}

#[test]
fn test_extract_json_with_think_tags() {
    let text = r#"<think>
Let me analyze concept [0] and concept [1] carefully.
I think [concept 0] is better because it has clearer composition.
</think>
[{"rank": 1, "concept_index": 0, "score": 90, "reasoning": "Clear focal point"}]"#;
    let result = extract_json_from_text(text).unwrap();
    assert!(result.is_array());
    assert_eq!(result[0]["rank"], 1);
}

#[test]
fn test_extract_json_with_markdown_code_block() {
    let text = "Here are the rankings:\n```json\n[{\"rank\": 1, \"concept_index\": 0, \"score\": 85, \"reasoning\": \"Great\"}]\n```";
    let result = extract_json_from_text(text).unwrap();
    assert!(result.is_array());
    assert_eq!(result[0]["score"], 85);
}

#[test]
fn test_extract_json_think_tags_with_code_block() {
    let text = r#"<think>
The user wants me to rank [these concepts]. Let me evaluate each one.
Concept [0] has strong visual clarity. Concept [1] is weaker.
</think>

```json
[{"rank": 1, "concept_index": 0, "score": 92, "reasoning": "Best"}]
```"#;
    let result = extract_json_from_text(text).unwrap();
    assert!(result.is_array());
    assert_eq!(result[0]["concept_index"], 0);
}

#[test]
fn test_parse_judge_with_think_tags() {
    let text = r#"<think>
Looking at the concepts, I need to evaluate [concept 0] vs [concept 1].
</think>
[{"rank": 1, "concept_index": 0, "score": 88, "reasoning": "Strong composition"}]"#;
    let result = parse_judge_rankings(text).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].concept_index, 0);
    assert_eq!(result[0].score, 88);
}

#[test]
fn test_parse_judge_wrapped_in_object() {
    let text = r#"{
        "ranked_concepts": [
            {"rank": 1, "concept_index": 0, "score": 85, "reasoning": "Best composition"},
            {"rank": 2, "concept_index": 1, "score": 75, "reasoning": "Good but complex"}
        ]
    }"#;
    let result = parse_judge_rankings(text).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].rank, 1);
    assert_eq!(result[0].concept_index, 0);
    assert_eq!(result[0].score, 85);
    assert_eq!(result[1].rank, 2);
    assert_eq!(result[1].concept_index, 1);
}

#[test]
fn test_parse_judge_wrapped_with_think_tags() {
    let text = r#"<think>
I need to evaluate these concepts carefully. [concept 0] looks strong.
</think>
{
    "ranked_concepts": [
        {"rank": 1, "concept_index": 0, "score": 90, "reasoning": "Clear focal point"}
    ]
}"#;
    let result = parse_judge_rankings(text).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].score, 90);
}

#[test]
fn test_parse_judge_single_object() {
    let text = r#"{"rank": 1, "concept_index": 0, "score": 85, "reasoning": "This concept checks all the boxes for visual clarity."}"#;
    let result = parse_judge_rankings(text).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].rank, 1);
    assert_eq!(result[0].concept_index, 0);
    assert_eq!(result[0].score, 85);
}
