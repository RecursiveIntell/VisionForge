use super::*;
use crate::types::queue::{QueueJob, QueueJobStatus, QueuePriority};

fn make_job_with_settings(settings_json: &str) -> QueueJob {
    QueueJob {
        id: "test-job".to_string(),
        priority: QueuePriority::Normal,
        status: QueueJobStatus::Pending,
        positive_prompt: "a cat".to_string(),
        negative_prompt: "lowres".to_string(),
        settings_json: settings_json.to_string(),
        pipeline_log: None,
        original_idea: Some("cat".to_string()),
        linked_comparison_id: None,
        created_at: None,
        started_at: None,
        completed_at: None,
        result_image_id: None,
    }
}

#[test]
fn test_build_generation_request_full() {
    let job = make_job_with_settings(
        r#"{"checkpoint":"sd_xl_base.safetensors","width":1024,"height":1024,"steps":30,"cfgScale":8.0,"sampler":"euler","scheduler":"normal","seed":42,"batchSize":2}"#,
    );
    let req = build_generation_request(&job).unwrap();
    assert_eq!(req.checkpoint, "sd_xl_base.safetensors");
    assert_eq!(req.width, 1024);
    assert_eq!(req.height, 1024);
    assert_eq!(req.steps, 30);
    assert_eq!(req.cfg_scale, 8.0);
    assert_eq!(req.sampler, "euler");
    assert_eq!(req.scheduler, "normal");
    assert_eq!(req.seed, 42);
    assert_eq!(req.batch_size, 2);
    assert_eq!(req.positive_prompt, "a cat");
    assert_eq!(req.negative_prompt, "lowres");
}

#[test]
fn test_build_generation_request_defaults() {
    let job = make_job_with_settings(r#"{}"#);
    let req = build_generation_request(&job).unwrap();
    assert_eq!(req.checkpoint, "dreamshaper_8.safetensors");
    assert_eq!(req.width, 512);
    assert_eq!(req.height, 768);
    assert_eq!(req.steps, 25);
    assert_eq!(req.cfg_scale, 7.5);
    assert_eq!(req.sampler, "dpmpp_2m");
    assert_eq!(req.scheduler, "karras");
    assert_eq!(req.seed, -1);
    assert_eq!(req.batch_size, 1);
}

#[test]
fn test_build_generation_request_snake_case_keys() {
    let job = make_job_with_settings(
        r#"{"checkpoint":"test.safetensors","cfg_scale":6.0,"batch_size":3}"#,
    );
    let req = build_generation_request(&job).unwrap();
    assert_eq!(req.cfg_scale, 6.0);
    assert_eq!(req.batch_size, 3);
}

#[test]
fn test_build_generation_request_invalid_json() {
    let job = make_job_with_settings("not json");
    let result = build_generation_request(&job);
    assert!(result.is_err());
}

#[test]
fn test_event_structs_serialize() {
    let started = JobStartedEvent { job_id: "j1".to_string() };
    let json = serde_json::to_string(&started).unwrap();
    assert!(json.contains("jobId"));

    let completed = JobCompletedEvent {
        job_id: "j1".to_string(),
        image_id: "img1".to_string(),
    };
    let json = serde_json::to_string(&completed).unwrap();
    assert!(json.contains("jobId"));
    assert!(json.contains("imageId"));

    let failed = JobFailedEvent {
        job_id: "j1".to_string(),
        error: "something broke".to_string(),
    };
    let json = serde_json::to_string(&failed).unwrap();
    assert!(json.contains("jobId"));
    assert!(json.contains("something broke"));
}
