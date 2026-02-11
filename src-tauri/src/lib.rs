pub mod ai;
pub mod comfyui;
pub mod commands;
pub mod config;
pub mod db;
pub mod gallery;
pub mod pipeline;
pub mod queue;
pub mod state;
pub mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = config::manager::load_or_create_default().expect("Failed to load configuration");

    let data_dir = config::manager::data_dir();
    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    // Create image directories (respects custom image_directory config)
    let image_base = config::manager::image_dir(&config);
    std::fs::create_dir_all(image_base.join("originals"))
        .expect("Failed to create originals directory");
    std::fs::create_dir_all(image_base.join("thumbnails"))
        .expect("Failed to create thumbnails directory");

    let db_path = data_dir.join("gallery.db");
    let conn = db::open_database(&db_path).expect("Failed to initialize database");

    // Requeue any jobs interrupted by previous shutdown
    let requeued = queue::manager::requeue_interrupted(&conn).unwrap_or(0);
    if requeued > 0 {
        eprintln!("[startup] Requeued {} interrupted jobs", requeued);
    }

    let app_state = state::AppState::new(conn, config);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .setup(|app| {
            queue::executor::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Config
            commands::config_cmds::get_config,
            commands::config_cmds::save_config,
            // Pipeline
            commands::pipeline_cmds::run_full_pipeline,
            commands::pipeline_cmds::run_pipeline_stage,
            commands::pipeline_cmds::cancel_pipeline,
            commands::pipeline_cmds::get_available_models,
            commands::pipeline_cmds::check_ollama_health,
            // ComfyUI
            commands::comfyui_cmds::check_comfyui_health,
            commands::comfyui_cmds::get_comfyui_checkpoints,
            commands::comfyui_cmds::get_comfyui_samplers,
            commands::comfyui_cmds::get_comfyui_schedulers,
            commands::comfyui_cmds::queue_generation,
            commands::comfyui_cmds::get_generation_status,
            commands::comfyui_cmds::get_comfyui_queue_status,
            commands::comfyui_cmds::free_comfyui_memory,
            commands::comfyui_cmds::interrupt_comfyui,
            // Queue
            commands::queue_cmds::add_to_queue,
            commands::queue_cmds::get_queue,
            commands::queue_cmds::reorder_queue,
            commands::queue_cmds::cancel_queue_job,
            commands::queue_cmds::pause_queue,
            commands::queue_cmds::resume_queue,
            commands::queue_cmds::is_queue_paused,
            // Gallery
            commands::gallery_cmds::get_gallery_images,
            commands::gallery_cmds::get_image,
            commands::gallery_cmds::delete_image,
            commands::gallery_cmds::restore_image,
            commands::gallery_cmds::permanently_delete_image,
            commands::gallery_cmds::update_image_rating,
            commands::gallery_cmds::update_image_favorite,
            commands::gallery_cmds::update_caption,
            commands::gallery_cmds::update_image_note,
            commands::gallery_cmds::add_tag,
            commands::gallery_cmds::remove_tag,
            commands::gallery_cmds::get_image_lineage,
            commands::gallery_cmds::get_image_file_path,
            commands::gallery_cmds::get_thumbnail_file_path,
            // AI
            commands::ai_cmds::tag_image,
            commands::ai_cmds::caption_image,
            // Seeds
            commands::seed_cmds::create_seed,
            commands::seed_cmds::get_seed,
            commands::seed_cmds::list_seeds,
            commands::seed_cmds::delete_seed,
            commands::seed_cmds::add_seed_tag,
            commands::seed_cmds::remove_seed_tag,
            commands::seed_cmds::add_seed_checkpoint_note,
            commands::seed_cmds::get_seed_checkpoint_notes,
            // Checkpoints
            commands::checkpoint_cmds::upsert_checkpoint,
            commands::checkpoint_cmds::get_checkpoint,
            commands::checkpoint_cmds::list_checkpoint_profiles,
            commands::checkpoint_cmds::add_prompt_term,
            commands::checkpoint_cmds::get_prompt_terms,
            commands::checkpoint_cmds::add_checkpoint_observation,
            commands::checkpoint_cmds::get_checkpoint_observations,
            commands::checkpoint_cmds::get_checkpoint_context,
            // Comparisons
            commands::comparison_cmds::create_comparison,
            commands::comparison_cmds::get_comparison,
            commands::comparison_cmds::list_comparisons,
            commands::comparison_cmds::list_comparisons_for_checkpoint,
            commands::comparison_cmds::update_comparison_note,
            commands::comparison_cmds::delete_comparison,
            // Export
            commands::export_cmds::export_images,
            commands::export_cmds::export_gallery,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
