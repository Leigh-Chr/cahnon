#[cfg(test)]
mod tests {
    use crate::database::Database;
    use crate::models::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.cahnon");
        let db = Database::create(&db_path).expect("Failed to create database");
        (db, temp_dir)
    }

    // ========================================================================
    // Project Tests
    // ========================================================================

    #[test]
    fn test_create_project() {
        let (db, _temp_dir) = create_test_db();

        let req = CreateProjectRequest {
            title: "Test Novel".to_string(),
            author: Some("Test Author".to_string()),
            description: Some("A test description".to_string()),
        };

        let project = db.create_project(&req).expect("Failed to create project");

        assert_eq!(project.title, "Test Novel");
        assert_eq!(project.author, Some("Test Author".to_string()));
        assert_eq!(project.description, Some("A test description".to_string()));
        assert!(!project.id.is_empty());
    }

    #[test]
    fn test_get_project() {
        let (db, _temp_dir) = create_test_db();

        let req = CreateProjectRequest {
            title: "My Project".to_string(),
            author: None,
            description: None,
        };

        let created = db.create_project(&req).expect("Failed to create project");
        let fetched = db.get_project().expect("Failed to get project");

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.title, "My Project");
    }

    #[test]
    fn test_update_project() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Original Title".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let updated = db
            .update_project(&UpdateProjectRequest {
                title: Some("New Title".to_string()),
                author: Some("New Author".to_string()),
                description: None,
                word_target: Some(50000),
                daily_word_target: None,
            })
            .expect("Failed to update project");

        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.author, Some("New Author".to_string()));
        assert_eq!(updated.word_target, Some(50000));
    }

    // ========================================================================
    // Chapter Tests
    // ========================================================================

    #[test]
    fn test_create_chapter() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let req = CreateChapterRequest {
            title: "Chapter 1".to_string(),
            summary: Some("First chapter".to_string()),
            position: None,
        };

        let chapter = db.create_chapter(&req).expect("Failed to create chapter");

        assert_eq!(chapter.title, "Chapter 1");
        assert_eq!(chapter.summary, Some("First chapter".to_string()));
        assert_eq!(chapter.status, "planned");
        assert_eq!(chapter.position, 1);
    }

    #[test]
    fn test_get_chapters() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        db.create_chapter(&CreateChapterRequest {
            title: "Chapter 1".to_string(),
            summary: None,
            position: None,
        })
        .expect("Failed to create chapter");

        db.create_chapter(&CreateChapterRequest {
            title: "Chapter 2".to_string(),
            summary: None,
            position: None,
        })
        .expect("Failed to create chapter");

        let chapters = db.get_chapters().expect("Failed to get chapters");

        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Chapter 1");
        assert_eq!(chapters[1].title, "Chapter 2");
    }

    #[test]
    fn test_update_chapter() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Original".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let updated = db
            .update_chapter(
                &chapter.id,
                &UpdateChapterRequest {
                    title: Some("Updated".to_string()),
                    summary: Some("New summary".to_string()),
                    status: Some("writing".to_string()),
                    notes: Some("Some notes".to_string()),
                    position: None,
                },
            )
            .expect("Failed to update chapter");

        assert_eq!(updated.title, "Updated");
        assert_eq!(updated.summary, Some("New summary".to_string()));
        assert_eq!(updated.status, "writing");
        assert_eq!(updated.notes, Some("Some notes".to_string()));
    }

    #[test]
    fn test_delete_chapter() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "To Delete".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        db.delete_chapter(&chapter.id)
            .expect("Failed to delete chapter");

        let chapters = db.get_chapters().expect("Failed to get chapters");
        assert!(chapters.is_empty());
    }

    #[test]
    fn test_reorder_chapters() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let c1 = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let c2 = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 2".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let c3 = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 3".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        // Reorder: c3, c1, c2
        db.reorder_chapters(&[c3.id.clone(), c1.id.clone(), c2.id.clone()])
            .expect("Failed to reorder");

        let chapters = db.get_chapters().expect("Failed to get chapters");
        assert_eq!(chapters[0].title, "Chapter 3");
        assert_eq!(chapters[1].title, "Chapter 1");
        assert_eq!(chapters[2].title, "Chapter 2");
    }

    // ========================================================================
    // Scene Tests
    // ========================================================================

    #[test]
    fn test_create_scene() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene 1".to_string(),
                summary: Some("First scene".to_string()),
                position: None,
            })
            .expect("Failed to create scene");

        assert_eq!(scene.title, "Scene 1");
        assert_eq!(scene.chapter_id, chapter.id);
        assert_eq!(scene.summary, Some("First scene".to_string()));
        assert_eq!(scene.status, "to write");
        assert_eq!(scene.position, 1);
        assert!(scene.on_timeline);
    }

    #[test]
    fn test_get_scenes() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene 1".to_string(),
            summary: None,
            position: None,
        })
        .expect("Failed to create scene");

        db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene 2".to_string(),
            summary: None,
            position: None,
        })
        .expect("Failed to create scene");

        let scenes = db.get_scenes(&chapter.id).expect("Failed to get scenes");

        assert_eq!(scenes.len(), 2);
        assert_eq!(scenes[0].title, "Scene 1");
        assert_eq!(scenes[1].title, "Scene 2");
    }

    #[test]
    fn test_update_scene() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Original".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create scene");

        let updated = db
            .update_scene(
                &scene.id,
                &UpdateSceneRequest {
                    title: Some("Updated".to_string()),
                    summary: Some("New summary".to_string()),
                    text: Some("Some text content here.".to_string()),
                    status: Some("writing".to_string()),
                    pov: Some("John".to_string()),
                    tags: Some("action,drama".to_string()),
                    notes: Some("Important scene".to_string()),
                    todos: Some("Add more detail".to_string()),
                    word_target: Some(1000),
                    time_point: None,
                    time_start: None,
                    time_end: None,
                    on_timeline: Some(false),
                    position: None,
                    pov_goal: None,
                    has_conflict: None,
                    has_change: None,
                    tension: None,
                    setup_for_scene_id: None,
                    payoff_of_scene_id: None,
                    revision_notes: None,
                    revision_checklist: None,
                },
            )
            .expect("Failed to update scene");

        assert_eq!(updated.title, "Updated");
        assert_eq!(updated.summary, Some("New summary".to_string()));
        assert_eq!(updated.text, "Some text content here.");
        assert_eq!(updated.status, "writing");
        assert_eq!(updated.pov, Some("John".to_string()));
        assert_eq!(updated.tags, Some("action,drama".to_string()));
        assert!(!updated.on_timeline);
    }

    #[test]
    fn test_delete_scene() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "To Delete".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create scene");

        db.delete_scene(&scene.id).expect("Failed to delete scene");

        let scenes = db.get_scenes(&chapter.id).expect("Failed to get scenes");
        assert!(scenes.is_empty());
    }

    #[test]
    fn test_move_scene_to_chapter() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter1 = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let chapter2 = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 2".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter1.id.clone(),
                title: "Scene 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create scene");

        let moved = db
            .move_scene_to_chapter(&scene.id, &chapter2.id, 0)
            .expect("Failed to move scene");

        assert_eq!(moved.chapter_id, chapter2.id);

        let scenes_c1 = db.get_scenes(&chapter1.id).expect("Failed to get scenes");
        let scenes_c2 = db.get_scenes(&chapter2.id).expect("Failed to get scenes");

        assert!(scenes_c1.is_empty());
        assert_eq!(scenes_c2.len(), 1);
    }

    // ========================================================================
    // Bible Entry Tests
    // ========================================================================

    #[test]
    fn test_create_bible_entry() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "John Doe".to_string(),
                aliases: Some("Johnny, JD".to_string()),
                short_description: Some("The protagonist".to_string()),
                full_description: Some("A detailed description of John Doe.".to_string()),
                status: None,
                tags: Some("hero,main".to_string()),
                color: Some("#FF5733".to_string()),
            })
            .expect("Failed to create bible entry");

        assert_eq!(entry.name, "John Doe");
        assert_eq!(entry.entry_type, "character");
        assert_eq!(entry.aliases, Some("Johnny, JD".to_string()));
        assert_eq!(entry.status, "draft");
        assert_eq!(entry.color, Some("#FF5733".to_string()));
    }

    #[test]
    fn test_get_bible_entries() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "John".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        })
        .expect("Failed to create");

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "location".to_string(),
            name: "Castle".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        })
        .expect("Failed to create");

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Jane".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        })
        .expect("Failed to create");

        // Get all entries
        let all = db.get_bible_entries(None).expect("Failed to get");
        assert_eq!(all.len(), 3);

        // Get only characters
        let characters = db
            .get_bible_entries(Some("character"))
            .expect("Failed to get");
        assert_eq!(characters.len(), 2);

        // Get only locations
        let locations = db
            .get_bible_entries(Some("location"))
            .expect("Failed to get");
        assert_eq!(locations.len(), 1);
    }

    #[test]
    fn test_update_bible_entry() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Original".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed to create");

        let updated = db
            .update_bible_entry(
                &entry.id,
                &UpdateBibleEntryRequest {
                    name: Some("Updated".to_string()),
                    aliases: Some("Alias1".to_string()),
                    short_description: Some("Short desc".to_string()),
                    full_description: None,
                    status: Some("confirmed".to_string()),
                    tags: None,
                    image_path: None,
                    notes: Some("Some notes".to_string()),
                    todos: None,
                    color: Some("#00FF00".to_string()),
                    custom_fields: None,
                },
            )
            .expect("Failed to update");

        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.aliases, Some("Alias1".to_string()));
        assert_eq!(updated.status, "confirmed");
        assert_eq!(updated.color, Some("#00FF00".to_string()));
    }

    #[test]
    fn test_delete_bible_entry() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "To Delete".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed to create");

        db.delete_bible_entry(&entry.id).expect("Failed to delete");

        let entries = db.get_bible_entries(None).expect("Failed to get");
        assert!(entries.is_empty());
    }

    // ========================================================================
    // Association Tests
    // ========================================================================

    #[test]
    fn test_create_association() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create scene");

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "John".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed to create");

        let association = db
            .create_association(&CreateAssociationRequest {
                scene_id: scene.id.clone(),
                bible_entry_id: entry.id.clone(),
            })
            .expect("Failed to create association");

        assert_eq!(association.scene_id, scene.id);
        assert_eq!(association.bible_entry_id, entry.id);
    }

    #[test]
    fn test_get_scene_associations() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create scene");

        let char1 = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "John".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed to create");

        let char2 = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Jane".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed to create");

        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: char1.id.clone(),
        })
        .expect("Failed");

        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: char2.id.clone(),
        })
        .expect("Failed");

        let associations = db.get_scene_associations(&scene.id).expect("Failed");
        assert_eq!(associations.len(), 2);
    }

    #[test]
    fn test_delete_association() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create scene");

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "John".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed to create");

        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        })
        .expect("Failed");

        db.delete_association(&scene.id, &entry.id)
            .expect("Failed to delete");

        let associations = db.get_scene_associations(&scene.id).expect("Failed");
        assert!(associations.is_empty());
    }

    // ========================================================================
    // Word Count Tests
    // ========================================================================

    #[test]
    fn test_get_word_counts() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create scene");

        // Update scene with some text
        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                title: None,
                summary: None,
                text: Some("This is a test with ten words in total here now.".to_string()),
                status: None,
                pov: None,
                tags: None,
                notes: None,
                todos: None,
                word_target: None,
                time_point: None,
                time_start: None,
                time_end: None,
                on_timeline: None,
                position: None,
                pov_goal: None,
                has_conflict: None,
                has_change: None,
                tension: None,
                setup_for_scene_id: None,
                payoff_of_scene_id: None,
                revision_notes: None,
                revision_checklist: None,
            },
        )
        .expect("Failed to update");

        let counts = db.get_word_counts().expect("Failed to get word counts");

        assert!(counts.total > 0);
        assert!(!counts.by_chapter.is_empty());
        assert!(!counts.by_status.is_empty());
    }

    // ========================================================================
    // Database File Tests
    // ========================================================================

    #[test]
    fn test_open_nonexistent_database() {
        let result = Database::open(&PathBuf::from("/nonexistent/path/test.cahnon"));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_and_reopen_database() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.cahnon");

        // Create database and add data
        {
            let db = Database::create(&db_path).expect("Failed to create");
            db.create_project(&CreateProjectRequest {
                title: "Persistent Project".to_string(),
                author: Some("Author".to_string()),
                description: None,
            })
            .expect("Failed to create project");
        }

        // Reopen and verify data
        {
            let db = Database::open(&db_path).expect("Failed to open");
            let project = db.get_project().expect("Failed to get project");
            assert_eq!(project.title, "Persistent Project");
            assert_eq!(project.author, Some("Author".to_string()));
        }
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_empty_chapters_list() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapters = db.get_chapters().expect("Failed to get chapters");
        assert!(chapters.is_empty());
    }

    #[test]
    fn test_chapter_cascade_delete() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene 1".to_string(),
            summary: None,
            position: None,
        })
        .expect("Failed to create scene");

        db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene 2".to_string(),
            summary: None,
            position: None,
        })
        .expect("Failed to create scene");

        // Delete chapter should also delete scenes
        db.delete_chapter(&chapter.id).expect("Failed to delete");

        let scenes = db.get_scenes(&chapter.id).expect("Failed");
        assert!(scenes.is_empty());
    }

    #[test]
    fn test_duplicate_association_is_ignored() {
        let (db, _temp_dir) = create_test_db();

        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create chapter");

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed to create scene");

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "John".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed to create");

        // Create association twice - should not fail
        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        })
        .expect("Failed");

        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        })
        .expect("Failed");

        // Should still only have one association
        let associations = db.get_scene_associations(&scene.id).expect("Failed");
        assert_eq!(associations.len(), 1);
    }
}
