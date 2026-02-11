use super::*;
use crate::db;

fn setup() -> Connection {
    db::open_memory_database().unwrap()
}

pub fn make_test_image(id: &str) -> ImageEntry {
    ImageEntry {
        id: id.to_string(),
        filename: format!("{}.png", id),
        created_at: "2026-01-15T10:00:00".to_string(),
        positive_prompt: Some("a cat on a throne".to_string()),
        negative_prompt: Some("lowres, bad anatomy".to_string()),
        original_idea: Some("cat throne".to_string()),
        checkpoint: Some("dreamshaper_8.safetensors".to_string()),
        width: Some(512),
        height: Some(768),
        steps: Some(25),
        cfg_scale: Some(7.5),
        sampler: Some("dpmpp_2m".to_string()),
        scheduler: Some("karras".to_string()),
        seed: Some(12345),
        pipeline_log: None,
        selected_concept: Some(2),
        auto_approved: false,
        caption: None,
        caption_edited: false,
        rating: None,
        favorite: false,
        deleted: false,
        user_note: None,
        tags: None,
    }
}

#[test]
fn test_insert_and_get() {
    let conn = setup();
    let img = make_test_image("img-001");
    insert_image(&conn, &img).unwrap();

    let retrieved = get_image(&conn, "img-001").unwrap().unwrap();
    assert_eq!(retrieved.id, "img-001");
    assert_eq!(retrieved.filename, "img-001.png");
    assert_eq!(retrieved.positive_prompt.unwrap(), "a cat on a throne");
    assert_eq!(retrieved.seed, Some(12345));
}

#[test]
fn test_get_nonexistent() {
    let conn = setup();
    assert!(get_image(&conn, "nope").unwrap().is_none());
}

#[test]
fn test_list_default_filter() {
    let conn = setup();
    for i in 0..5 {
        insert_image(&conn, &make_test_image(&format!("img-{:03}", i))).unwrap();
    }
    let images = list_images(&conn, &GalleryFilter::default()).unwrap();
    assert_eq!(images.len(), 5);
}

#[test]
fn test_list_with_checkpoint_filter() {
    let conn = setup();
    let mut img1 = make_test_image("img-001");
    img1.checkpoint = Some("dreamshaper.safetensors".to_string());
    let mut img2 = make_test_image("img-002");
    img2.checkpoint = Some("deliberate.safetensors".to_string());
    insert_image(&conn, &img1).unwrap();
    insert_image(&conn, &img2).unwrap();

    let filter = GalleryFilter {
        checkpoint: Some("dreamshaper.safetensors".to_string()),
        ..Default::default()
    };
    let images = list_images(&conn, &filter).unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].id, "img-001");
}

#[test]
fn test_list_with_search() {
    let conn = setup();
    let mut img1 = make_test_image("img-001");
    img1.positive_prompt = Some("beautiful sunset over ocean".to_string());
    let mut img2 = make_test_image("img-002");
    img2.positive_prompt = Some("dark forest at night".to_string());
    insert_image(&conn, &img1).unwrap();
    insert_image(&conn, &img2).unwrap();

    let filter = GalleryFilter {
        search: Some("sunset".to_string()),
        ..Default::default()
    };
    let images = list_images(&conn, &filter).unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].id, "img-001");
}

#[test]
fn test_pagination() {
    let conn = setup();
    for i in 0..10 {
        insert_image(&conn, &make_test_image(&format!("img-{:03}", i))).unwrap();
    }

    let page1 = list_images(
        &conn,
        &GalleryFilter {
            limit: Some(3),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page1.len(), 3);

    let page2 = list_images(
        &conn,
        &GalleryFilter {
            limit: Some(3),
            offset: Some(3),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page2.len(), 3);
    assert_ne!(page1[0].id, page2[0].id);
}

#[test]
fn test_soft_delete_and_restore() {
    let conn = setup();
    insert_image(&conn, &make_test_image("img-001")).unwrap();

    soft_delete_image(&conn, "img-001").unwrap();
    assert_eq!(
        list_images(&conn, &GalleryFilter::default()).unwrap().len(),
        0
    );

    let deleted = list_images(
        &conn,
        &GalleryFilter {
            show_deleted: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(deleted.len(), 1);

    restore_image(&conn, "img-001").unwrap();
    assert_eq!(
        list_images(&conn, &GalleryFilter::default()).unwrap().len(),
        1
    );
}

#[test]
fn test_update_rating_and_favorite() {
    let conn = setup();
    insert_image(&conn, &make_test_image("img-001")).unwrap();
    update_image_rating(&conn, "img-001", Some(5)).unwrap();
    update_image_favorite(&conn, "img-001", true).unwrap();

    let img = get_image(&conn, "img-001").unwrap().unwrap();
    assert_eq!(img.rating, Some(5));
    assert!(img.favorite);

    let found = list_images(
        &conn,
        &GalleryFilter {
            min_rating: Some(4),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(found.len(), 1);
    let empty = list_images(
        &conn,
        &GalleryFilter {
            min_rating: Some(6),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_update_caption() {
    let conn = setup();
    insert_image(&conn, &make_test_image("img-001")).unwrap();
    update_image_caption(&conn, "img-001", "A beautiful cat", true).unwrap();

    let img = get_image(&conn, "img-001").unwrap().unwrap();
    assert_eq!(img.caption.unwrap(), "A beautiful cat");
    assert!(img.caption_edited);
}

#[test]
fn test_favorite_only_filter() {
    let conn = setup();
    insert_image(&conn, &make_test_image("img-001")).unwrap();
    insert_image(&conn, &make_test_image("img-002")).unwrap();
    update_image_favorite(&conn, "img-001", true).unwrap();

    let results = list_images(
        &conn,
        &GalleryFilter {
            favorite_only: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "img-001");
}

#[test]
fn test_permanent_delete() {
    let conn = setup();
    insert_image(&conn, &make_test_image("img-001")).unwrap();
    permanently_delete_image(&conn, "img-001").unwrap();
    assert!(get_image(&conn, "img-001").unwrap().is_none());
}
