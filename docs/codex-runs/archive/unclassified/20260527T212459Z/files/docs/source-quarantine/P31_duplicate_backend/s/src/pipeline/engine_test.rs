use super::*;
use crate::types::pipeline::{
    ComposerOutput, IdeatorOutput, JudgeOutput, JudgeRanking, ModelsUsed, PipelineConfig,
    PipelineResult, PipelineStages, PromptEngineerOutput, PromptPair, ReviewerOutput,
};

fn make_test_result() -> PipelineResult {
    PipelineResult {
        original_idea: "a cat on a throne".to_string(),
        pipeline_config: PipelineConfig {
            stages_enabled: [true, true, true, true, false],
            models_used: ModelsUsed {
                ideator: Some("mistral:7b".to_string()),
                composer: Some("llama3.1:8b".to_string()),
                judge: Some("qwen2.5:7b".to_string()),
                prompt_engineer: Some("mistral:7b".to_string()),
                reviewer: None,
            },
        },
        stages: PipelineStages {
            ideator: Some(IdeatorOutput {
                input: "a cat on a throne".to_string(),
                output: vec!["Concept A".to_string(), "Concept B".to_string()],
                duration_ms: 1000,
                model: "mistral:7b".to_string(),
                tokens_in: Some(50),
                tokens_out: Some(200),
            }),
            composer: Some(ComposerOutput {
                input_concept_index: 1,
                input: "Concept B".to_string(),
                output: "Rich description of concept B".to_string(),
                duration_ms: 1500,
                model: "llama3.1:8b".to_string(),
                tokens_in: Some(80),
                tokens_out: Some(150),
            }),
            judge: Some(JudgeOutput {
                input: vec!["Desc A".to_string(), "Desc B".to_string()],
                output: vec![
                    JudgeRanking {
                        rank: 1,
                        concept_index: 1,
                        score: 92,
                        reasoning: "Better composition".to_string(),
                    },
                    JudgeRanking {
                        rank: 2,
                        concept_index: 0,
                        score: 85,
                        reasoning: "Good but less focused".to_string(),
                    },
                ],
                duration_ms: 2000,
                model: "qwen2.5:7b".to_string(),
            }),
            prompt_engineer: Some(PromptEngineerOutput {
                input: "Rich description".to_string(),
                checkpoint_context: None,
                output: PromptPair {
                    positive: "masterpiece, cat on throne".to_string(),
                    negative: "lowres, blurry".to_string(),
                },
                duration_ms: 1000,
                model: "mistral:7b".to_string(),
                tokens_in: Some(100),
                tokens_out: Some(60),
            }),
            reviewer: None,
        },
        user_edits: None,
        auto_approved: false,
        generation_settings: None,
    }
}

#[test]
fn test_get_final_prompts() {
    let result = make_test_result();
    let prompts = get_final_prompts(&result).unwrap();
    assert_eq!(prompts.positive, "masterpiece, cat on throne");
    assert_eq!(prompts.negative, "lowres, blurry");
}

#[test]
fn test_get_final_prompts_no_pe() {
    let mut result = make_test_result();
    result.stages.prompt_engineer = None;
    assert!(get_final_prompts(&result).is_none());
}

#[test]
fn test_get_selected_concept() {
    let result = make_test_result();
    assert_eq!(get_selected_concept(&result), 1);
}

#[test]
fn test_get_selected_concept_no_judge() {
    let mut result = make_test_result();
    result.stages.judge = None;
    assert_eq!(get_selected_concept(&result), 0);
}

#[test]
fn test_pipeline_result_serialization() {
    let result = make_test_result();
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("originalIdea"));
    assert!(json.contains("masterpiece"));
    let parsed: PipelineResult = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.original_idea, "a cat on a throne");
}

#[test]
fn test_reviewer_overrides_prompts() {
    let mut result = make_test_result();
    result.stages.reviewer = Some(ReviewerOutput {
        approved: false,
        issues: Some(vec!["prompt drift".to_string()]),
        suggested_positive: Some("better positive".to_string()),
        suggested_negative: Some("better negative".to_string()),
        duration_ms: 500,
        model: "qwen2.5:7b".to_string(),
    });

    // Simulate the engine's reviewer override logic
    if let Some(ref reviewer) = result.stages.reviewer {
        if !reviewer.approved {
            if let Some(ref mut pe) = result.stages.prompt_engineer {
                if let Some(ref sp) = reviewer.suggested_positive {
                    pe.output.positive = sp.clone();
                }
                if let Some(ref sn) = reviewer.suggested_negative {
                    pe.output.negative = sn.clone();
                }
            }
        }
    }

    let prompts = get_final_prompts(&result).unwrap();
    assert_eq!(prompts.positive, "better positive");
    assert_eq!(prompts.negative, "better negative");
}
