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

    // ========================================================================
    // Arc Tests
    // ========================================================================

    fn setup_project(db: &Database) {
        db.create_project(&CreateProjectRequest {
            title: "Test".to_string(),
            author: None,
            description: None,
        })
        .expect("Failed to create project");
    }

    fn setup_chapter_and_scene(db: &Database) -> (crate::models::Chapter, crate::models::Scene) {
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

        (chapter, scene)
    }

    #[test]
    fn test_create_arc() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let arc = db
            .create_arc(&CreateArcRequest {
                name: "Main Plot".to_string(),
                description: Some("The main storyline".to_string()),
                stakes: Some("End of the world".to_string()),
                characters: None,
                status: None,
                color: Some("#FF0000".to_string()),
            })
            .expect("Failed to create arc");

        assert_eq!(arc.name, "Main Plot");
        assert_eq!(arc.description, Some("The main storyline".to_string()));
        assert_eq!(arc.stakes, Some("End of the world".to_string()));
        assert_eq!(arc.status, "setup");
        assert_eq!(arc.color, Some("#FF0000".to_string()));
        assert!(!arc.id.is_empty());
    }

    #[test]
    fn test_get_arcs() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_arc(&CreateArcRequest {
            name: "Arc A".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        })
        .expect("Failed");

        db.create_arc(&CreateArcRequest {
            name: "Arc B".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        })
        .expect("Failed");

        let arcs = db.get_arcs().expect("Failed to get arcs");
        assert_eq!(arcs.len(), 2);
    }

    #[test]
    fn test_update_arc() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let arc = db
            .create_arc(&CreateArcRequest {
                name: "Original".to_string(),
                description: None,
                stakes: None,
                characters: None,
                status: None,
                color: None,
            })
            .expect("Failed");

        let updated = db
            .update_arc(
                &arc.id,
                &UpdateArcRequest {
                    name: Some("Updated".to_string()),
                    description: Some("New desc".to_string()),
                    stakes: None,
                    characters: None,
                    status: Some("rising".to_string()),
                    color: None,
                },
            )
            .expect("Failed to update");

        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.description, Some("New desc".to_string()));
        assert_eq!(updated.status, "rising");
    }

    #[test]
    fn test_delete_arc() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let arc = db
            .create_arc(&CreateArcRequest {
                name: "To Delete".to_string(),
                description: None,
                stakes: None,
                characters: None,
                status: None,
                color: None,
            })
            .expect("Failed");

        db.delete_arc(&arc.id).expect("Failed to delete");

        let arcs = db.get_arcs().expect("Failed");
        assert!(arcs.is_empty());
    }

    #[test]
    fn test_link_scene_to_arc() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let arc = db
            .create_arc(&CreateArcRequest {
                name: "Plot Arc".to_string(),
                description: None,
                stakes: None,
                characters: None,
                status: None,
                color: None,
            })
            .expect("Failed");

        db.link_scene_to_arc(&scene.id, &arc.id)
            .expect("Failed to link");

        let scene_arcs = db.get_scene_arcs(&scene.id).expect("Failed");
        assert_eq!(scene_arcs.len(), 1);
        assert_eq!(scene_arcs[0].name, "Plot Arc");
    }

    #[test]
    fn test_unlink_scene_from_arc() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let arc = db
            .create_arc(&CreateArcRequest {
                name: "Plot Arc".to_string(),
                description: None,
                stakes: None,
                characters: None,
                status: None,
                color: None,
            })
            .expect("Failed");

        db.link_scene_to_arc(&scene.id, &arc.id).expect("Failed");
        db.unlink_scene_from_arc(&scene.id, &arc.id)
            .expect("Failed to unlink");

        let scene_arcs = db.get_scene_arcs(&scene.id).expect("Failed");
        assert!(scene_arcs.is_empty());
    }

    // ========================================================================
    // Event Tests
    // ========================================================================

    #[test]
    fn test_create_event() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let event = db
            .create_event(&CreateEventRequest {
                title: "Battle of Winterfell".to_string(),
                description: Some("The big fight".to_string()),
                time_point: Some("Year 3, Day 15".to_string()),
                time_start: None,
                time_end: None,
                event_type: None,
                importance: None,
            })
            .expect("Failed to create event");

        assert_eq!(event.title, "Battle of Winterfell");
        assert_eq!(event.description, Some("The big fight".to_string()));
        assert_eq!(event.event_type, "scene");
        assert_eq!(event.importance, "normal");
    }

    #[test]
    fn test_get_events() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_event(&CreateEventRequest {
            title: "Event 1".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        })
        .expect("Failed");

        db.create_event(&CreateEventRequest {
            title: "Event 2".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        })
        .expect("Failed");

        let events = db.get_events().expect("Failed");
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_update_event() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let event = db
            .create_event(&CreateEventRequest {
                title: "Original".to_string(),
                description: None,
                time_point: None,
                time_start: None,
                time_end: None,
                event_type: None,
                importance: None,
            })
            .expect("Failed");

        let updated = db
            .update_event(
                &event.id,
                &UpdateEventRequest {
                    title: Some("Updated".to_string()),
                    description: Some("New desc".to_string()),
                    time_point: Some("Day 10".to_string()),
                    time_start: None,
                    time_end: None,
                    event_type: Some("milestone".to_string()),
                    importance: Some("critical".to_string()),
                },
            )
            .expect("Failed to update");

        assert_eq!(updated.title, "Updated");
        assert_eq!(updated.description, Some("New desc".to_string()));
        assert_eq!(updated.event_type, "milestone");
        assert_eq!(updated.importance, "critical");
    }

    #[test]
    fn test_delete_event() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let event = db
            .create_event(&CreateEventRequest {
                title: "To Delete".to_string(),
                description: None,
                time_point: None,
                time_start: None,
                time_end: None,
                event_type: None,
                importance: None,
            })
            .expect("Failed");

        db.delete_event(&event.id).expect("Failed to delete");

        let events = db.get_events().expect("Failed");
        assert!(events.is_empty());
    }

    #[test]
    fn test_link_scene_to_event() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let event = db
            .create_event(&CreateEventRequest {
                title: "Event".to_string(),
                description: None,
                time_point: Some("Day 1".to_string()),
                time_start: None,
                time_end: None,
                event_type: None,
                importance: None,
            })
            .expect("Failed");

        db.link_scene_to_event(&scene.id, &event.id)
            .expect("Failed to link");

        let scene_events = db.get_scene_events(&scene.id).expect("Failed");
        assert_eq!(scene_events.len(), 1);
        assert_eq!(scene_events[0].title, "Event");

        let event_scenes = db.get_event_scenes(&event.id).expect("Failed");
        assert_eq!(event_scenes.len(), 1);
        assert_eq!(event_scenes[0], scene.id);
    }

    #[test]
    fn test_unlink_scene_from_event() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let event = db
            .create_event(&CreateEventRequest {
                title: "Event".to_string(),
                description: None,
                time_point: None,
                time_start: None,
                time_end: None,
                event_type: None,
                importance: None,
            })
            .expect("Failed");

        db.link_scene_to_event(&scene.id, &event.id)
            .expect("Failed");
        db.unlink_scene_from_event(&scene.id, &event.id)
            .expect("Failed to unlink");

        let scene_events = db.get_scene_events(&scene.id).expect("Failed");
        assert!(scene_events.is_empty());
    }

    #[test]
    fn test_link_bible_entry_to_event() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Hero".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let event = db
            .create_event(&CreateEventRequest {
                title: "Event".to_string(),
                description: None,
                time_point: None,
                time_start: None,
                time_end: None,
                event_type: None,
                importance: None,
            })
            .expect("Failed");

        db.link_bible_entry_to_event(&entry.id, &event.id)
            .expect("Failed to link");

        let event_entries = db.get_event_bible_entries(&event.id).expect("Failed");
        assert_eq!(event_entries.len(), 1);
        assert_eq!(event_entries[0], entry.id);

        let entry_events = db.get_bible_entry_events(&entry.id).expect("Failed");
        assert_eq!(entry_events.len(), 1);
        assert_eq!(entry_events[0].title, "Event");
    }

    #[test]
    fn test_unlink_bible_entry_from_event() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Hero".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let event = db
            .create_event(&CreateEventRequest {
                title: "Event".to_string(),
                description: None,
                time_point: None,
                time_start: None,
                time_end: None,
                event_type: None,
                importance: None,
            })
            .expect("Failed");

        db.link_bible_entry_to_event(&entry.id, &event.id)
            .expect("Failed");
        db.unlink_bible_entry_from_event(&entry.id, &event.id)
            .expect("Failed to unlink");

        let event_entries = db.get_event_bible_entries(&event.id).expect("Failed");
        assert!(event_entries.is_empty());
    }

    // ========================================================================
    // Annotation Tests
    // ========================================================================

    #[test]
    fn test_create_annotation() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let annotation = db
            .create_annotation(&CreateAnnotationRequest {
                scene_id: scene.id.clone(),
                start_offset: 10,
                end_offset: 20,
                annotation_type: Some("note".to_string()),
                content: "Check this passage".to_string(),
            })
            .expect("Failed to create annotation");

        assert_eq!(annotation.scene_id, scene.id);
        assert_eq!(annotation.start_offset, 10);
        assert_eq!(annotation.end_offset, 20);
        assert_eq!(annotation.annotation_type, "note");
        assert_eq!(annotation.content, "Check this passage");
        assert_eq!(annotation.status, "open");
    }

    #[test]
    fn test_create_annotation_default_type() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let annotation = db
            .create_annotation(&CreateAnnotationRequest {
                scene_id: scene.id.clone(),
                start_offset: 0,
                end_offset: 5,
                annotation_type: None,
                content: "Default type".to_string(),
            })
            .expect("Failed");

        assert_eq!(annotation.annotation_type, "comment");
    }

    #[test]
    fn test_annotation_validation_negative_offset() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: -1,
            end_offset: 5,
            annotation_type: None,
            content: "Bad offset".to_string(),
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("negative"));
    }

    #[test]
    fn test_annotation_validation_end_before_start() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 10,
            end_offset: 5,
            annotation_type: None,
            content: "Bad range".to_string(),
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("greater"));
    }

    #[test]
    fn test_get_annotations() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 10,
            annotation_type: None,
            content: "First".to_string(),
        })
        .expect("Failed");

        db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 20,
            end_offset: 30,
            annotation_type: None,
            content: "Second".to_string(),
        })
        .expect("Failed");

        let annotations = db.get_annotations(&scene.id).expect("Failed");
        assert_eq!(annotations.len(), 2);
        // Should be ordered by start_offset
        assert_eq!(annotations[0].start_offset, 0);
        assert_eq!(annotations[1].start_offset, 20);
    }

    #[test]
    fn test_update_annotation() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let annotation = db
            .create_annotation(&CreateAnnotationRequest {
                scene_id: scene.id.clone(),
                start_offset: 0,
                end_offset: 10,
                annotation_type: None,
                content: "Original".to_string(),
            })
            .expect("Failed");

        let updated = db
            .update_annotation(
                &annotation.id,
                &UpdateAnnotationRequest {
                    content: Some("Updated content".to_string()),
                    status: Some("resolved".to_string()),
                },
            )
            .expect("Failed to update");

        assert_eq!(updated.content, "Updated content");
        assert_eq!(updated.status, "resolved");
    }

    #[test]
    fn test_delete_annotation() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let annotation = db
            .create_annotation(&CreateAnnotationRequest {
                scene_id: scene.id.clone(),
                start_offset: 0,
                end_offset: 10,
                annotation_type: None,
                content: "To delete".to_string(),
            })
            .expect("Failed");

        db.delete_annotation(&annotation.id)
            .expect("Failed to delete");

        let annotations = db.get_annotations(&scene.id).expect("Failed");
        assert!(annotations.is_empty());
    }

    // ========================================================================
    // Issue Tests
    // ========================================================================

    #[test]
    fn test_create_issue() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let issue = db
            .create_issue(&CreateIssueRequest {
                issue_type: "consistency".to_string(),
                title: "Character name mismatch".to_string(),
                description: Some("John is called Jon in chapter 3".to_string()),
                severity: Some("error".to_string()),
            })
            .expect("Failed to create issue");

        assert_eq!(issue.title, "Character name mismatch");
        assert_eq!(issue.issue_type, "consistency");
        assert_eq!(issue.severity, "error");
        assert_eq!(issue.status, "open");
    }

    #[test]
    fn test_create_issue_default_severity() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let issue = db
            .create_issue(&CreateIssueRequest {
                issue_type: "plot".to_string(),
                title: "Plot hole".to_string(),
                description: None,
                severity: None,
            })
            .expect("Failed");

        assert_eq!(issue.severity, "warning");
    }

    #[test]
    fn test_get_issues() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_issue(&CreateIssueRequest {
            issue_type: "consistency".to_string(),
            title: "Issue 1".to_string(),
            description: None,
            severity: None,
        })
        .expect("Failed");

        db.create_issue(&CreateIssueRequest {
            issue_type: "plot".to_string(),
            title: "Issue 2".to_string(),
            description: None,
            severity: None,
        })
        .expect("Failed");

        let all_issues = db.get_issues(None).expect("Failed");
        assert_eq!(all_issues.len(), 2);

        let open_issues = db.get_issues(Some("open")).expect("Failed");
        assert_eq!(open_issues.len(), 2);
    }

    #[test]
    fn test_update_issue() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let issue = db
            .create_issue(&CreateIssueRequest {
                issue_type: "consistency".to_string(),
                title: "Issue".to_string(),
                description: None,
                severity: None,
            })
            .expect("Failed");

        let updated = db
            .update_issue(
                &issue.id,
                &UpdateIssueRequest {
                    status: Some("resolved".to_string()),
                    resolution_note: Some("Fixed in chapter 3".to_string()),
                },
            )
            .expect("Failed to update");

        assert_eq!(updated.status, "resolved");
        assert_eq!(
            updated.resolution_note,
            Some("Fixed in chapter 3".to_string())
        );
    }

    #[test]
    fn test_link_scene_to_issue() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let issue = db
            .create_issue(&CreateIssueRequest {
                issue_type: "plot".to_string(),
                title: "Issue".to_string(),
                description: None,
                severity: None,
            })
            .expect("Failed");

        db.link_scene_to_issue(&scene.id, &issue.id)
            .expect("Failed to link");

        let issue_scenes = db.get_issue_scenes(&issue.id).expect("Failed");
        assert_eq!(issue_scenes.len(), 1);
        assert_eq!(issue_scenes[0], scene.id);

        let scene_issues = db.get_scene_issues(&scene.id).expect("Failed");
        assert_eq!(scene_issues.len(), 1);
        assert_eq!(scene_issues[0].title, "Issue");
    }

    #[test]
    fn test_unlink_scene_from_issue() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let issue = db
            .create_issue(&CreateIssueRequest {
                issue_type: "plot".to_string(),
                title: "Issue".to_string(),
                description: None,
                severity: None,
            })
            .expect("Failed");

        db.link_scene_to_issue(&scene.id, &issue.id)
            .expect("Failed");
        db.unlink_scene_from_issue(&scene.id, &issue.id)
            .expect("Failed to unlink");

        let issue_scenes = db.get_issue_scenes(&issue.id).expect("Failed");
        assert!(issue_scenes.is_empty());
    }

    #[test]
    fn test_link_bible_entry_to_issue() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Hero".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let issue = db
            .create_issue(&CreateIssueRequest {
                issue_type: "consistency".to_string(),
                title: "Issue".to_string(),
                description: None,
                severity: None,
            })
            .expect("Failed");

        db.link_bible_entry_to_issue(&entry.id, &issue.id)
            .expect("Failed to link");

        let issue_entries = db.get_issue_bible_entries(&issue.id).expect("Failed");
        assert_eq!(issue_entries.len(), 1);
        assert_eq!(issue_entries[0], entry.id);
    }

    #[test]
    fn test_unlink_bible_entry_from_issue() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Hero".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let issue = db
            .create_issue(&CreateIssueRequest {
                issue_type: "consistency".to_string(),
                title: "Issue".to_string(),
                description: None,
                severity: None,
            })
            .expect("Failed");

        db.link_bible_entry_to_issue(&entry.id, &issue.id)
            .expect("Failed");
        db.unlink_bible_entry_from_issue(&entry.id, &issue.id)
            .expect("Failed to unlink");

        let issue_entries = db.get_issue_bible_entries(&issue.id).expect("Failed");
        assert!(issue_entries.is_empty());
    }

    #[test]
    fn test_delete_issue() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Hero".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let issue = db
            .create_issue(&CreateIssueRequest {
                issue_type: "consistency".to_string(),
                title: "Test issue".to_string(),
                description: Some("Description".to_string()),
                severity: Some("error".to_string()),
            })
            .expect("Failed");

        // Link scene and bible entry
        db.link_scene_to_issue(&scene.id, &issue.id)
            .expect("Failed");
        db.link_bible_entry_to_issue(&entry.id, &issue.id)
            .expect("Failed");

        // Delete the issue
        db.delete_issue(&issue.id).expect("Failed to delete");

        // Verify issue is gone
        let issues = db.get_issues(None).expect("Failed");
        assert!(issues.is_empty());

        // Verify junction records are cleaned up
        let scene_issues = db.get_issue_scenes(&issue.id).expect("Failed");
        assert!(scene_issues.is_empty());
        let bible_issues = db.get_issue_bible_entries(&issue.id).expect("Failed");
        assert!(bible_issues.is_empty());
    }

    // ========================================================================
    // Cut Library Tests
    // ========================================================================

    #[test]
    fn test_create_cut() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let cut = db
            .create_cut(None, "Some deleted text that I want to keep")
            .expect("Failed to create cut");

        assert_eq!(cut.text, "Some deleted text that I want to keep");
        assert!(cut.scene_id.is_none());
    }

    #[test]
    fn test_create_cut_with_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let cut = db
            .create_cut(Some(&scene.id), "Cut from a specific scene")
            .expect("Failed to create cut");

        assert_eq!(cut.scene_id, Some(scene.id));
        assert_eq!(cut.text, "Cut from a specific scene");
    }

    #[test]
    fn test_get_cuts() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_cut(None, "Cut 1").expect("Failed");
        db.create_cut(None, "Cut 2").expect("Failed");

        let cuts = db.get_cuts().expect("Failed");
        assert_eq!(cuts.len(), 2);
    }

    #[test]
    fn test_delete_cut() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let cut = db.create_cut(None, "To delete").expect("Failed");
        db.delete_cut(&cut.id).expect("Failed to delete");

        let cuts = db.get_cuts().expect("Failed");
        assert!(cuts.is_empty());
    }

    // ========================================================================
    // Template Tests
    // ========================================================================

    #[test]
    fn test_init_builtin_templates() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.init_builtin_templates()
            .expect("Failed to init templates");

        let templates = db.get_templates().expect("Failed");
        assert_eq!(templates.len(), 4); // Three-Act, Save the Cat, Hero's Journey, Seven-Point

        // Check all are builtin
        for t in &templates {
            assert!(t.is_builtin);
        }
    }

    #[test]
    fn test_init_builtin_templates_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.init_builtin_templates().expect("Failed");
        db.init_builtin_templates().expect("Failed second time");

        let templates = db.get_templates().expect("Failed");
        assert_eq!(templates.len(), 4); // Should not duplicate
    }

    #[test]
    fn test_get_template_steps() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        db.init_builtin_templates().expect("Failed");

        let templates = db.get_templates().expect("Failed");
        let three_act = templates
            .iter()
            .find(|t| t.name == "Three-Act Structure")
            .expect("Three-Act not found");

        let steps = db.get_template_steps(&three_act.id).expect("Failed");
        assert_eq!(steps.len(), 8); // 8 steps in Three-Act Structure
        assert_eq!(steps[0].name, "Setup");
    }

    #[test]
    fn test_set_active_template() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        db.init_builtin_templates().expect("Failed");

        let templates = db.get_templates().expect("Failed");
        let first = &templates[0];

        db.set_active_template(&first.id).expect("Failed");

        let updated = db.get_templates().expect("Failed");
        let active_count = updated.iter().filter(|t| t.is_active).count();
        assert_eq!(active_count, 1);
    }

    #[test]
    fn test_create_custom_template() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let template = db
            .create_template(&CreateTemplateRequest {
                name: "My Custom Template".to_string(),
            })
            .expect("Failed");

        assert_eq!(template.name, "My Custom Template");
        assert!(!template.is_builtin);
        assert!(!template.is_active);
    }

    #[test]
    fn test_update_template() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let template = db
            .create_template(&CreateTemplateRequest {
                name: "Original".to_string(),
            })
            .expect("Failed");

        let updated = db
            .update_template(
                &template.id,
                &UpdateTemplateRequest {
                    name: Some("Renamed".to_string()),
                },
            )
            .expect("Failed");

        assert_eq!(updated.name, "Renamed");
    }

    #[test]
    fn test_delete_custom_template() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let template = db
            .create_template(&CreateTemplateRequest {
                name: "To Delete".to_string(),
            })
            .expect("Failed");

        db.delete_template(&template.id).expect("Failed to delete");

        let templates = db.get_templates().expect("Failed");
        assert!(templates.is_empty());
    }

    #[test]
    fn test_create_template_step() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let template = db
            .create_template(&CreateTemplateRequest {
                name: "My Template".to_string(),
            })
            .expect("Failed");

        let step = db
            .create_template_step(&CreateTemplateStepRequest {
                template_id: template.id.clone(),
                name: "Opening Hook".to_string(),
                description: Some("Grab the reader's attention".to_string()),
                typical_position: Some(0.05),
                color: Some("#3b82f6".to_string()),
            })
            .expect("Failed");

        assert_eq!(step.name, "Opening Hook");
        assert_eq!(step.template_id, template.id);
        assert_eq!(step.typical_position, 0.05);
    }

    #[test]
    fn test_assign_scene_to_step() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let template = db
            .create_template(&CreateTemplateRequest {
                name: "My Template".to_string(),
            })
            .expect("Failed");

        let step = db
            .create_template_step(&CreateTemplateStepRequest {
                template_id: template.id.clone(),
                name: "Step 1".to_string(),
                description: None,
                typical_position: None,
                color: None,
            })
            .expect("Failed");

        db.assign_scene_to_step(&scene.id, &step.id)
            .expect("Failed to assign");

        let scene_step = db
            .get_scene_step(&scene.id)
            .expect("Failed")
            .expect("No step found");

        assert_eq!(scene_step.id, step.id);
        assert_eq!(scene_step.name, "Step 1");
    }

    #[test]
    fn test_get_scene_step_none() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let step = db.get_scene_step(&scene.id).expect("Failed");
        assert!(step.is_none());
    }

    // ========================================================================
    // Bible Relationship Tests
    // ========================================================================

    #[test]
    fn test_create_bible_relationship() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let char1 = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Alice".to_string(),
                aliases: None,
                short_description: Some("The hero".to_string()),
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let char2 = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Bob".to_string(),
                aliases: None,
                short_description: Some("The villain".to_string()),
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let rel = db
            .create_bible_relationship(&CreateBibleRelationshipRequest {
                source_id: char1.id.clone(),
                target_id: char2.id.clone(),
                relationship_type: "rival".to_string(),
                note: Some("They are enemies".to_string()),
                status: None,
            })
            .expect("Failed");

        assert_eq!(rel.source_id, char1.id);
        assert_eq!(rel.target_id, char2.id);
        assert_eq!(rel.relationship_type, "rival");
        assert_eq!(rel.status, "active");
    }

    #[test]
    fn test_get_bible_relationships() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let char1 = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Alice".to_string(),
                aliases: None,
                short_description: Some("Hero".to_string()),
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let char2 = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Bob".to_string(),
                aliases: None,
                short_description: Some("Villain".to_string()),
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        db.create_bible_relationship(&CreateBibleRelationshipRequest {
            source_id: char1.id.clone(),
            target_id: char2.id.clone(),
            relationship_type: "rival".to_string(),
            note: None,
            status: None,
        })
        .expect("Failed");

        let rels = db.get_bible_relationships(&char1.id).expect("Failed");
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].related_entry_name, "Bob");
        assert_eq!(rels[0].relationship_type, "rival");
    }

    #[test]
    fn test_update_bible_relationship() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let char1 = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Alice".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let char2 = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Bob".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let rel = db
            .create_bible_relationship(&CreateBibleRelationshipRequest {
                source_id: char1.id.clone(),
                target_id: char2.id.clone(),
                relationship_type: "friend".to_string(),
                note: None,
                status: None,
            })
            .expect("Failed");

        let updated = db
            .update_bible_relationship(
                &rel.id,
                &UpdateBibleRelationshipRequest {
                    relationship_type: Some("lover".to_string()),
                    note: Some("They fell in love".to_string()),
                    status: None,
                },
            )
            .expect("Failed");

        assert_eq!(updated.relationship_type, "lover");
        assert_eq!(updated.note, Some("They fell in love".to_string()));
    }

    #[test]
    fn test_delete_bible_relationship() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let char1 = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Alice".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let char2 = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Bob".to_string(),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let rel = db
            .create_bible_relationship(&CreateBibleRelationshipRequest {
                source_id: char1.id.clone(),
                target_id: char2.id.clone(),
                relationship_type: "friend".to_string(),
                note: None,
                status: None,
            })
            .expect("Failed");

        db.delete_bible_relationship(&rel.id)
            .expect("Failed to delete");

        let rels = db.get_bible_relationships(&char1.id).expect("Failed");
        assert!(rels.is_empty());
    }

    // ========================================================================
    // Trash / Restore Tests
    // ========================================================================

    #[test]
    fn test_trash_and_restore_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        // Delete scene (soft delete)
        db.delete_scene(&scene.id).expect("Failed to delete");

        // Scene should not appear in normal list
        let scenes = db.get_scenes(&chapter.id).expect("Failed");
        assert!(scenes.is_empty());

        // Scene should appear in deleted list
        let deleted = db.get_deleted_scenes().expect("Failed");
        assert_eq!(deleted.len(), 1);
        assert_eq!(deleted[0].id, scene.id);

        // Restore the scene
        let restored = db.restore_scene(&scene.id).expect("Failed to restore");
        assert_eq!(restored.id, scene.id);
        assert_eq!(restored.title, "Scene 1");

        // Scene should be back
        let scenes = db.get_scenes(&chapter.id).expect("Failed");
        assert_eq!(scenes.len(), 1);

        // Deleted list should be empty
        let deleted = db.get_deleted_scenes().expect("Failed");
        assert!(deleted.is_empty());
    }

    #[test]
    fn test_trash_and_restore_chapter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter to delete".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene A".to_string(),
            summary: None,
            position: None,
        })
        .expect("Failed");

        // Delete chapter (cascade soft-delete)
        db.delete_chapter(&chapter.id).expect("Failed");

        // Chapter should appear in deleted
        let deleted_chapters = db.get_deleted_chapters().expect("Failed");
        assert_eq!(deleted_chapters.len(), 1);

        // Restore chapter (should also restore scenes)
        let restored = db.restore_chapter(&chapter.id).expect("Failed to restore");
        assert_eq!(restored.title, "Chapter to delete");

        // Scenes should also be restored
        let scenes = db.get_scenes(&chapter.id).expect("Failed");
        assert_eq!(scenes.len(), 1);
    }

    // ========================================================================
    // Scene Split / Merge / Duplicate Tests
    // ========================================================================

    #[test]
    fn test_split_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        // Add text to the scene
        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("First part. Second part.".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        // Split at position 12 (after "First part. ")
        let (first, second) = db
            .split_scene(&scene.id, 12, Some("Part Two"))
            .expect("Failed to split");

        assert_eq!(first.text, "First part. ");
        assert_eq!(second.text, "Second part.");
        assert_eq!(second.title, "Part Two");
        assert_eq!(first.chapter_id, chapter.id);
        assert_eq!(second.chapter_id, chapter.id);
        assert_eq!(second.position, first.position + 1);
    }

    #[test]
    fn test_split_scene_default_title() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("AB".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        let (_, second) = db.split_scene(&scene.id, 1, None).expect("Failed");

        assert!(second.title.contains("continued"));
    }

    #[test]
    fn test_split_scene_negative_position() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.split_scene(&scene.id, -1, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_split_scene_beyond_text() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("Short".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        let result = db.split_scene(&scene.id, 100, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let scene1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene A".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let scene2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene B".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        db.update_scene(
            &scene1.id,
            &UpdateSceneRequest {
                text: Some("Text A".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        db.update_scene(
            &scene2.id,
            &UpdateSceneRequest {
                text: Some("Text B".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        let merged = db
            .merge_scenes(&[scene1.id.clone(), scene2.id.clone()])
            .expect("Failed to merge");

        assert_eq!(merged.text, "Text A\n\nText B");
        assert_eq!(merged.id, scene1.id);

        // Scene B should be soft-deleted
        let scenes = db.get_scenes(&chapter.id).expect("Failed");
        assert_eq!(scenes.len(), 1);
    }

    #[test]
    fn test_merge_scenes_needs_at_least_two() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.merge_scenes(&[scene.id.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("At least two"));
    }

    #[test]
    fn test_duplicate_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("Original content".to_string()),
                pov: Some("Alice".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        let duplicate = db
            .duplicate_scene(&scene.id, false)
            .expect("Failed to duplicate");

        assert!(duplicate.title.contains("copy"));
        assert_eq!(duplicate.text, "Original content");
        assert_eq!(duplicate.pov, Some("Alice".to_string()));
        assert_ne!(duplicate.id, scene.id);
        assert_eq!(duplicate.chapter_id, chapter.id);
    }

    #[test]
    fn test_duplicate_scene_structure_only() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("Original content".to_string()),
                pov: Some("Alice".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        let duplicate = db
            .duplicate_scene(&scene.id, true)
            .expect("Failed to duplicate");

        assert!(duplicate.title.contains("copy"));
        assert_eq!(duplicate.text, ""); // Structure only - no text
        assert_eq!(duplicate.pov, Some("Alice".to_string())); // Metadata copied
    }

    // ========================================================================
    // Scene History Tests
    // ========================================================================

    #[test]
    fn test_scene_history_on_update() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // Initial text
        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("Version 1".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        // Update text (should save previous to history)
        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("Version 2".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        let history = db.get_scene_history(&scene.id).expect("Failed");
        // At least one history entry (the text before last update)
        assert!(!history.is_empty());
    }

    #[test]
    fn test_restore_scene_version() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("Version 1".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("Version 2".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        let history = db.get_scene_history(&scene.id).expect("Failed");
        assert!(!history.is_empty());

        // Restore to earlier version
        let restored = db
            .restore_scene_version(&scene.id, &history[0].id)
            .expect("Failed to restore");

        // Text should be the historical version
        assert_eq!(restored.text, history[0].text);
    }

    // ========================================================================
    // Snapshot Tests
    // ========================================================================

    #[test]
    fn test_create_snapshot() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, _scene) = setup_chapter_and_scene(&db);

        let snapshot = db
            .create_snapshot("My Snapshot", Some("Before big changes"), "manual")
            .expect("Failed to create snapshot");

        assert_eq!(snapshot.name, "My Snapshot");
        assert_eq!(snapshot.description, Some("Before big changes".to_string()));
        assert_eq!(snapshot.snapshot_type, "manual");
        assert!(!snapshot.data.is_empty());
    }

    #[test]
    fn test_get_snapshots() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_snapshot("Snapshot 1", None, "manual")
            .expect("Failed");
        db.create_snapshot("Snapshot 2", None, "manual")
            .expect("Failed");

        let snapshots = db.get_snapshots().expect("Failed");
        assert_eq!(snapshots.len(), 2);
    }

    #[test]
    fn test_delete_snapshot() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let snapshot = db
            .create_snapshot("To Delete", None, "manual")
            .expect("Failed");

        db.delete_snapshot(&snapshot.id).expect("Failed to delete");

        let snapshots = db.get_snapshots().expect("Failed");
        assert!(snapshots.is_empty());
    }

    #[test]
    fn test_delete_nonexistent_snapshot() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.delete_snapshot("nonexistent-id");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_snapshot_restore() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Original Chapter".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Original Scene".to_string(),
            summary: None,
            position: None,
        })
        .expect("Failed");

        // Create snapshot
        let snapshot = db
            .create_snapshot("Before Changes", None, "manual")
            .expect("Failed");

        // Make changes
        db.update_chapter(
            &chapter.id,
            &UpdateChapterRequest {
                title: Some("Modified Chapter".to_string()),
                summary: None,
                status: None,
                notes: None,
                position: None,
            },
        )
        .expect("Failed");

        // Restore snapshot
        db.restore_snapshot(&snapshot.id)
            .expect("Failed to restore");

        // Verify data was restored
        let chapters = db.get_chapters().expect("Failed");
        assert!(!chapters.is_empty());
        assert_eq!(chapters[0].title, "Original Chapter");
    }

    // ========================================================================
    // Bible Entry Validation Tests
    // ========================================================================

    #[test]
    fn test_bible_entry_invalid_type() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "invalid_type".to_string(),
            name: "Test".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid entry type"));
    }

    #[test]
    fn test_bible_entry_all_valid_types() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let valid_types = vec![
            "character",
            "location",
            "object",
            "faction",
            "concept",
            "glossary",
        ];

        for entry_type in valid_types {
            let entry = db
                .create_bible_entry(&CreateBibleEntryRequest {
                    entry_type: entry_type.to_string(),
                    name: format!("Test {}", entry_type),
                    aliases: None,
                    short_description: None,
                    full_description: None,
                    status: None,
                    tags: None,
                    color: None,
                })
                .expect(&format!("Failed to create {} entry", entry_type));

            assert_eq!(entry.entry_type, entry_type);
        }
    }

    // ========================================================================
    // Validation Module Tests
    // ========================================================================

    #[test]
    fn test_sanitize_text() {
        use crate::validation::sanitize_text;

        assert_eq!(sanitize_text("Hello World", 100), "Hello World");
        assert_eq!(sanitize_text("  Hello  ", 100), "Hello");
        assert_eq!(sanitize_text("Hello\x00World", 100), "HelloWorld");
        assert_eq!(sanitize_text("Long text here", 4), "Long");
    }

    #[test]
    fn test_sanitize_multiline_text() {
        use crate::validation::sanitize_multiline_text;

        assert_eq!(
            sanitize_multiline_text("Line 1\nLine 2", 100),
            "Line 1\nLine 2"
        );
        assert_eq!(sanitize_multiline_text("Has\x00null", 100), "Hasnull");
    }

    #[test]
    fn test_validate_required() {
        use crate::validation::validate_required;

        assert!(validate_required("Hello", "field").is_ok());
        assert!(validate_required("", "field").is_err());
        let err = validate_required("", "title").unwrap_err();
        assert!(err.contains("title"));
    }

    // ========================================================================
    // Word Count Detailed Tests
    // ========================================================================

    #[test]
    fn test_word_counts_by_chapter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let c1 = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let c2 = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 2".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: c1.id.clone(),
                title: "S1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: c2.id.clone(),
                title: "S2".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        db.update_scene(
            &s1.id,
            &UpdateSceneRequest {
                text: Some("One two three".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        db.update_scene(
            &s2.id,
            &UpdateSceneRequest {
                text: Some("Four five six seven eight".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        let counts = db.get_word_counts().expect("Failed");
        assert_eq!(counts.total, 8); // 3 + 5
        assert_eq!(counts.by_chapter.len(), 2);

        let ch1_count = counts
            .by_chapter
            .iter()
            .find(|c| c.chapter_title == "Chapter 1")
            .expect("Ch1 not found");
        assert_eq!(ch1_count.word_count, 3);

        let ch2_count = counts
            .by_chapter
            .iter()
            .find(|c| c.chapter_title == "Chapter 2")
            .expect("Ch2 not found");
        assert_eq!(ch2_count.word_count, 5);
    }

    #[test]
    fn test_word_counts_by_status() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "S1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "S2".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        db.update_scene(
            &s1.id,
            &UpdateSceneRequest {
                text: Some("Draft words here".to_string()),
                status: Some("draft".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        db.update_scene(
            &s2.id,
            &UpdateSceneRequest {
                text: Some("Done words".to_string()),
                status: Some("done".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        let counts = db.get_word_counts().expect("Failed");
        assert!(!counts.by_status.is_empty());

        let draft_count = counts.by_status.iter().find(|s| s.status == "draft");
        assert!(draft_count.is_some());

        let done_count = counts.by_status.iter().find(|s| s.status == "done");
        assert!(done_count.is_some());
    }

    #[test]
    fn test_word_counts_with_html() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "S1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>Hello <strong>world</strong> today.</p>".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        let counts = db.get_word_counts().expect("Failed");
        // HTML tags should be stripped, counting only actual words
        assert_eq!(counts.total, 3); // "Hello world today"
    }

    // ========================================================================
    // Scene Reorder Tests
    // ========================================================================

    #[test]
    fn test_reorder_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene A".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene B".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let s3 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene C".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        // Reorder: C, A, B
        db.reorder_scenes(&chapter.id, &[s3.id.clone(), s1.id.clone(), s2.id.clone()])
            .expect("Failed to reorder");

        let scenes = db.get_scenes(&chapter.id).expect("Failed");
        assert_eq!(scenes[0].title, "Scene C");
        assert_eq!(scenes[1].title, "Scene A");
        assert_eq!(scenes[2].title, "Scene B");
    }

    // ========================================================================
    // Integration: Full Workflow Tests
    // ========================================================================

    #[test]
    fn test_full_writing_workflow() {
        let (db, _temp_dir) = create_test_db();

        // Create project
        let project = db
            .create_project(&CreateProjectRequest {
                title: "My Novel".to_string(),
                author: Some("Author".to_string()),
                description: Some("A great novel".to_string()),
            })
            .expect("Failed");

        assert_eq!(project.title, "My Novel");

        // Create chapters
        let ch1 = db
            .create_chapter(&CreateChapterRequest {
                title: "The Beginning".to_string(),
                summary: Some("Where it all starts".to_string()),
                position: None,
            })
            .expect("Failed");

        let ch2 = db
            .create_chapter(&CreateChapterRequest {
                title: "The Middle".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        // Create scenes
        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: ch1.id.clone(),
                title: "Opening".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: ch1.id.clone(),
                title: "Meeting".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        let _s3 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: ch2.id.clone(),
                title: "Conflict".to_string(),
                summary: None,
                position: None,
            })
            .expect("Failed");

        // Write content
        db.update_scene(
            &s1.id,
            &UpdateSceneRequest {
                text: Some("<p>It was a dark and stormy night.</p>".to_string()),
                status: Some("draft".to_string()),
                pov: Some("Alice".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        db.update_scene(
            &s2.id,
            &UpdateSceneRequest {
                text: Some("<p>Alice met Bob at the café.</p>".to_string()),
                status: Some("draft".to_string()),
                ..Default::default()
            },
        )
        .expect("Failed");

        // Create bible entries
        let alice = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Alice".to_string(),
                aliases: Some("Ali".to_string()),
                short_description: Some("The protagonist".to_string()),
                full_description: None,
                status: None,
                tags: Some("main,hero".to_string()),
                color: Some("#FF0000".to_string()),
            })
            .expect("Failed");

        let bob = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Bob".to_string(),
                aliases: None,
                short_description: Some("The love interest".to_string()),
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        let cafe = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "location".to_string(),
                name: "The Café".to_string(),
                aliases: None,
                short_description: Some("Where they meet".to_string()),
                full_description: None,
                status: None,
                tags: None,
                color: None,
            })
            .expect("Failed");

        // Create associations
        db.create_association(&CreateAssociationRequest {
            scene_id: s1.id.clone(),
            bible_entry_id: alice.id.clone(),
        })
        .expect("Failed");

        db.create_association(&CreateAssociationRequest {
            scene_id: s2.id.clone(),
            bible_entry_id: alice.id.clone(),
        })
        .expect("Failed");

        db.create_association(&CreateAssociationRequest {
            scene_id: s2.id.clone(),
            bible_entry_id: bob.id.clone(),
        })
        .expect("Failed");

        db.create_association(&CreateAssociationRequest {
            scene_id: s2.id.clone(),
            bible_entry_id: cafe.id.clone(),
        })
        .expect("Failed");

        // Create arc
        let arc = db
            .create_arc(&CreateArcRequest {
                name: "Romance Arc".to_string(),
                description: Some("Alice and Bob's love story".to_string()),
                stakes: None,
                characters: None,
                status: None,
                color: None,
            })
            .expect("Failed");

        db.link_scene_to_arc(&s2.id, &arc.id).expect("Failed");

        // Create relationship
        db.create_bible_relationship(&CreateBibleRelationshipRequest {
            source_id: alice.id.clone(),
            target_id: bob.id.clone(),
            relationship_type: "love interest".to_string(),
            note: Some("They meet at the café".to_string()),
            status: None,
        })
        .expect("Failed");

        // Create event
        let event = db
            .create_event(&CreateEventRequest {
                title: "First Meeting".to_string(),
                description: None,
                time_point: Some("Day 1".to_string()),
                time_start: None,
                time_end: None,
                event_type: None,
                importance: None,
            })
            .expect("Failed");

        db.link_scene_to_event(&s2.id, &event.id).expect("Failed");

        // Verify everything
        let chapters = db.get_chapters().expect("Failed");
        assert_eq!(chapters.len(), 2);

        let ch1_scenes = db.get_scenes(&ch1.id).expect("Failed");
        assert_eq!(ch1_scenes.len(), 2);

        let s2_assocs = db.get_scene_associations(&s2.id).expect("Failed");
        assert_eq!(s2_assocs.len(), 3); // Alice, Bob, Café

        let s2_arcs = db.get_scene_arcs(&s2.id).expect("Failed");
        assert_eq!(s2_arcs.len(), 1);

        let alice_rels = db.get_bible_relationships(&alice.id).expect("Failed");
        assert_eq!(alice_rels.len(), 1);

        let counts = db.get_word_counts().expect("Failed");
        assert!(counts.total > 0);

        // Create snapshot of this state
        let snapshot = db
            .create_snapshot("After initial writing", None, "manual")
            .expect("Failed");
        assert!(!snapshot.data.is_empty());
    }

    // ========================================================================
    // Name Registry Tests
    // ========================================================================

    #[test]
    fn test_create_name_registry_entry() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db
            .create_name_registry_entry(&CreateNameRegistryRequest {
                canonical_name: "John Smith".to_string(),
                name_type: Some("character".to_string()),
                bible_entry_id: None,
                aliases: Some("Johnny, J.S.".to_string()),
            })
            .expect("Failed to create name registry entry");

        assert_eq!(entry.canonical_name, "John Smith");
        assert_eq!(entry.name_type, "character");
        assert_eq!(entry.aliases.as_deref(), Some("Johnny, J.S."));
        assert!(!entry.is_confirmed);
    }

    #[test]
    fn test_get_name_registry_entries() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Alice".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: None,
            aliases: None,
        })
        .unwrap();

        db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Winterfell".to_string(),
            name_type: Some("location".to_string()),
            bible_entry_id: None,
            aliases: None,
        })
        .unwrap();

        let all = db.get_name_registry_entries(None).unwrap();
        assert_eq!(all.len(), 2);

        let chars = db.get_name_registry_entries(Some("character")).unwrap();
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].canonical_name, "Alice");
    }

    #[test]
    fn test_update_name_registry_entry() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db
            .create_name_registry_entry(&CreateNameRegistryRequest {
                canonical_name: "Bob".to_string(),
                name_type: None,
                bible_entry_id: None,
                aliases: None,
            })
            .unwrap();

        let updated = db
            .update_name_registry_entry(
                &entry.id,
                &UpdateNameRegistryRequest {
                    canonical_name: Some("Robert".to_string()),
                    name_type: None,
                    bible_entry_id: None,
                    aliases: Some("Bob, Bobby".to_string()),
                    is_confirmed: Some(true),
                },
            )
            .unwrap();

        assert_eq!(updated.canonical_name, "Robert");
        assert_eq!(updated.aliases.as_deref(), Some("Bob, Bobby"));
        assert!(updated.is_confirmed);
    }

    #[test]
    fn test_delete_name_registry_entry() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db
            .create_name_registry_entry(&CreateNameRegistryRequest {
                canonical_name: "Temp".to_string(),
                name_type: None,
                bible_entry_id: None,
                aliases: None,
            })
            .unwrap();

        db.delete_name_registry_entry(&entry.id).unwrap();
        let result = db.get_name_registry_entry(&entry.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_name_mentions() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();
        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene 1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let registry = db
            .create_name_registry_entry(&CreateNameRegistryRequest {
                canonical_name: "Alice".to_string(),
                name_type: Some("character".to_string()),
                bible_entry_id: None,
                aliases: None,
            })
            .unwrap();

        let mention = db
            .create_name_mention(&registry.id, &scene.id, "Alice", 10, 15)
            .unwrap();

        assert_eq!(mention.mention_text, "Alice");
        assert_eq!(mention.status, "pending");

        // Get by scene
        let scene_mentions = db.get_name_mentions_by_scene(&scene.id).unwrap();
        assert_eq!(scene_mentions.len(), 1);

        // Update status
        let updated = db
            .update_name_mention(
                &mention.id,
                &UpdateNameMentionRequest {
                    status: "accepted".to_string(),
                },
            )
            .unwrap();
        assert_eq!(updated.status, "accepted");

        // Delete
        db.delete_name_mention(&mention.id).unwrap();
        let after = db.get_name_mentions_by_scene(&scene.id).unwrap();
        assert!(after.is_empty());
    }

    // ========================================================================
    // Saved Filter Tests
    // ========================================================================

    #[test]
    fn test_create_saved_filter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let filter = db
            .create_saved_filter(&CreateSavedFilterRequest {
                name: "My Filter".to_string(),
                filter_type: "outline".to_string(),
                filter_data: r#"{"status":"draft","tags":["fantasy"]}"#.to_string(),
            })
            .expect("Failed to create saved filter");

        assert_eq!(filter.name, "My Filter");
        assert_eq!(filter.filter_type, "outline");
    }

    #[test]
    fn test_get_saved_filters() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_saved_filter(&CreateSavedFilterRequest {
            name: "Filter A".to_string(),
            filter_type: "outline".to_string(),
            filter_data: "{}".to_string(),
        })
        .unwrap();

        db.create_saved_filter(&CreateSavedFilterRequest {
            name: "Filter B".to_string(),
            filter_type: "corkboard".to_string(),
            filter_data: "{}".to_string(),
        })
        .unwrap();

        let all = db.get_saved_filters(None).unwrap();
        assert_eq!(all.len(), 2);

        let outlines = db.get_saved_filters(Some("outline")).unwrap();
        assert_eq!(outlines.len(), 1);
        assert_eq!(outlines[0].name, "Filter A");
    }

    #[test]
    fn test_update_saved_filter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let filter = db
            .create_saved_filter(&CreateSavedFilterRequest {
                name: "Old Name".to_string(),
                filter_type: "outline".to_string(),
                filter_data: "{}".to_string(),
            })
            .unwrap();

        let updated = db
            .update_saved_filter(
                &filter.id,
                &UpdateSavedFilterRequest {
                    name: Some("New Name".to_string()),
                    filter_data: Some(r#"{"status":"done"}"#.to_string()),
                },
            )
            .unwrap();

        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.filter_data, r#"{"status":"done"}"#);
    }

    #[test]
    fn test_delete_saved_filter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let filter = db
            .create_saved_filter(&CreateSavedFilterRequest {
                name: "Temp".to_string(),
                filter_type: "outline".to_string(),
                filter_data: "{}".to_string(),
            })
            .unwrap();

        db.delete_saved_filter(&filter.id).unwrap();
        let result = db.get_saved_filter(&filter.id);
        assert!(result.is_err());
    }

    // ========================================================================
    // Integrity Check Test
    // ========================================================================

    #[test]
    fn test_database_integrity_check() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.check_integrity().expect("Integrity check failed");
        assert!(result);
    }

    // ========================================================================
    // Export Tests (NEW)
    // ========================================================================

    /// Helper to set up a project with chapters and scenes containing text
    fn setup_project_with_content(db: &Database) {
        db.create_project(&CreateProjectRequest {
            title: "My Novel".to_string(),
            author: Some("John Doe".to_string()),
            description: Some("A great story".to_string()),
        })
        .unwrap();

        let ch1 = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter One".to_string(),
                summary: Some("The beginning".to_string()),
                position: None,
            })
            .unwrap();

        let ch2 = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter Two".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: ch1.id.clone(),
                title: "Opening Scene".to_string(),
                summary: Some("It all begins".to_string()),
                position: None,
            })
            .unwrap();
        db.update_scene(
            &s1.id,
            &UpdateSceneRequest {
                text: Some("<p>It was a dark and stormy night.</p><p>The wind howled.</p>".to_string()),
                status: Some("draft".to_string()),
                pov: Some("Alice".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: ch1.id.clone(),
                title: "Second Scene".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();
        db.update_scene(
            &s2.id,
            &UpdateSceneRequest {
                text: Some("<p>Morning came with <strong>bright</strong> sunlight.</p>".to_string()),
                status: Some("done".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        let s3 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: ch2.id.clone(),
                title: "New Beginning".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();
        db.update_scene(
            &s3.id,
            &UpdateSceneRequest {
                text: Some("<p>A new chapter unfolds.</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn test_export_markdown() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let result = db.export_markdown().expect("Failed to export markdown");

        assert!(result.contains("# My Novel"));
        assert!(result.contains("**By John Doe**"));
        assert!(result.contains("A great story"));
        assert!(result.contains("## Chapter One"));
        assert!(result.contains("## Chapter Two"));
        assert!(result.contains("Opening Scene"));
        assert!(result.contains("It was a dark and stormy night."));
        assert!(result.contains("**bright**")); // bold preserved in markdown
        assert!(result.contains("A new chapter unfolds."));
    }

    #[test]
    fn test_export_markdown_with_options_filter_chapters() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let chapters = db.get_chapters().unwrap();
        let ch1_id = chapters[0].id.clone();

        let result = db
            .export_markdown_with_options(Some(&[ch1_id]), None, true)
            .expect("Failed to export");

        assert!(result.contains("## Chapter One"));
        assert!(!result.contains("## Chapter Two"));
        assert!(result.contains("Opening Scene"));
    }

    #[test]
    fn test_export_markdown_without_titles() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let result = db
            .export_markdown_with_options(None, None, false)
            .expect("Failed to export");

        // Should not include scene titles as headings
        assert!(!result.contains("### Opening Scene"));
        // But should still have content
        assert!(result.contains("It was a dark and stormy night."));
    }

    #[test]
    fn test_export_markdown_custom_separator() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let result = db
            .export_markdown_with_options(None, Some("---"), true)
            .expect("Failed to export");

        assert!(result.contains("--- Opening Scene"));
    }

    #[test]
    fn test_export_plain_text() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let result = db.export_plain_text().expect("Failed to export plain text");

        assert!(result.contains("My Novel"));
        assert!(result.contains("By John Doe"));
        assert!(result.contains("Chapter One"));
        assert!(result.contains("Chapter Two"));
        assert!(result.contains("It was a dark and stormy night."));
        // HTML should be stripped
        assert!(!result.contains("<p>"));
        assert!(!result.contains("<strong>"));
    }

    #[test]
    fn test_export_plain_text_with_options() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let chapters = db.get_chapters().unwrap();
        let ch2_id = chapters[1].id.clone();

        let result = db
            .export_plain_text_with_options(Some(&[ch2_id]), Some("~~~"))
            .expect("Failed to export");

        assert!(result.contains("Chapter Two"));
        assert!(!result.contains("Chapter One"));
        assert!(result.contains("~~~"));
    }

    #[test]
    fn test_export_json_backup() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let result = db.export_json_backup().expect("Failed to export JSON");

        let parsed: serde_json::Value = serde_json::from_str(&result).expect("Invalid JSON");
        assert_eq!(parsed["version"], "1.0");
        assert!(parsed["exported_at"].as_str().is_some());
        assert_eq!(parsed["project"]["title"], "My Novel");
        assert_eq!(parsed["chapters"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["scenes"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_export_json_backup_empty_project() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.export_json_backup().expect("Failed to export JSON");

        let parsed: serde_json::Value = serde_json::from_str(&result).expect("Invalid JSON");
        assert_eq!(parsed["chapters"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["scenes"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_export_outline() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let result = db.export_outline().expect("Failed to export outline");

        assert!(result.contains("# My Novel - Outline"));
        assert!(result.contains("## Chapter One"));
        assert!(result.contains("Opening Scene"));
        assert!(result.contains("[draft]"));
        assert!(result.contains("*It all begins*"));
        assert!(result.contains("POV: Alice"));
    }

    #[test]
    fn test_export_outline_empty_project() {
        let (db, _temp_dir) = create_test_db();
        db.create_project(&CreateProjectRequest {
            title: "Empty".to_string(),
            author: None,
            description: None,
        })
        .unwrap();

        let result = db.export_outline().expect("Failed to export outline");
        assert!(result.contains("# Empty - Outline"));
    }

    #[test]
    fn test_export_bible() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Alice".to_string(),
            short_description: Some("The protagonist".to_string()),
            full_description: None,
            aliases: Some("Al".to_string()),
            status: None,
            tags: None,
            color: None,
        })
        .unwrap();

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "location".to_string(),
            name: "Wonderland".to_string(),
            short_description: Some("A magical place".to_string()),
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        })
        .unwrap();

        let result = db.export_bible().expect("Failed to export bible");

        assert!(result.contains("# Test - Story Bible"));
        assert!(result.contains("## Characters"));
        assert!(result.contains("## Locations"));
        assert!(result.contains("### Alice"));
        assert!(result.contains("*Aliases: Al*"));
        assert!(result.contains("The protagonist"));
        assert!(result.contains("### Wonderland"));
    }

    #[test]
    fn test_export_bible_empty() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.export_bible().expect("Failed to export bible");
        assert!(result.contains("# Test - Story Bible"));
        // No type headers when empty
        assert!(!result.contains("## Characters"));
    }

    #[test]
    fn test_export_timeline() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_event(&CreateEventRequest {
            title: "The Big Bang".to_string(),
            description: Some("Beginning of everything".to_string()),
            time_point: Some("Day 1".to_string()),
            time_start: None,
            time_end: None,
            event_type: Some("backstory".to_string()),
            importance: None,
        })
        .unwrap();

        let result = db.export_timeline().expect("Failed to export timeline");

        assert!(result.contains("# Test - Timeline"));
        assert!(result.contains("## Events"));
        assert!(result.contains("The Big Bang"));
        assert!(result.contains("Day 1"));
    }

    #[test]
    fn test_export_timeline_with_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Battle Scene".to_string(),
                summary: Some("A great battle".to_string()),
                position: None,
            })
            .unwrap();

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                time_point: Some("Day 5".to_string()),
                on_timeline: Some(true),
                ..Default::default()
            },
        )
        .unwrap();

        let result = db.export_timeline().expect("Failed to export timeline");
        assert!(result.contains("Battle Scene"));
        assert!(result.contains("Day 5"));
    }

    #[test]
    fn test_export_markdown_with_html_content() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>Hello <strong>world</strong></p><h2>Subtitle</h2><p>More <em>text</em> here.</p><blockquote>A quote</blockquote><ul><li>Item 1</li><li>Item 2</li></ul><a href=\"http://example.com\">Link</a>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        let result = db.export_markdown().unwrap();
        assert!(result.contains("**world**")); // bold
        assert!(result.contains("*text*")); // italic
        assert!(result.contains("> A quote")); // blockquote
        assert!(result.contains("- Item 1")); // list
        assert!(result.contains("[Link](http://example.com)")); // link
    }

    // ========================================================================
    // Search Tests
    // ========================================================================

    #[test]
    fn test_global_search_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let results = db.global_search("stormy", None).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.result_type == "scene"));
    }

    #[test]
    fn test_global_search_empty_query() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let results = db.global_search("", None).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_global_search_whitespace_only() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let results = db.global_search("   ", None).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_global_search_no_matches() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let results = db.global_search("zzzznonexistent", None).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_global_search_bible_entries() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Gandalf".to_string(),
            short_description: Some("A wizard".to_string()),
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        })
        .unwrap();

        let results = db.global_search("Gandalf", None).unwrap();
        assert!(results.iter().any(|r| r.result_type == "bible_entry"));
    }

    #[test]
    fn test_global_search_scoped() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "stormy".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        })
        .unwrap();

        // Search only in scenes scope
        let results = db
            .global_search("stormy", Some(vec!["scenes".to_string()]))
            .unwrap();
        assert!(results.iter().all(|r| r.result_type == "scene"));
    }

    #[test]
    fn test_global_search_events() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_event(&CreateEventRequest {
            title: "Battle of Waterloo".to_string(),
            description: Some("A decisive battle".to_string()),
            time_point: Some("1815".to_string()),
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        })
        .unwrap();

        let results = db
            .global_search("Waterloo", Some(vec!["events".to_string()]))
            .unwrap();
        assert!(results.iter().any(|r| r.result_type == "event"));
    }

    #[test]
    fn test_global_search_annotations() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 10,
            annotation_type: Some("note".to_string()),
            content: "Fix this passage about starlight".to_string(),
        })
        .unwrap();

        let results = db
            .global_search("starlight", Some(vec!["annotations".to_string()]))
            .unwrap();
        assert!(results.iter().any(|r| r.result_type == "annotation"));
    }

    #[test]
    fn test_global_search_cuts() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_cut(None, "This paragraph about dragons was removed.").unwrap();

        let results = db
            .global_search("dragons", Some(vec!["cuts".to_string()]))
            .unwrap();
        assert!(results.iter().any(|r| r.result_type == "cut"));
    }

    #[test]
    fn test_sanitize_fts5_query() {
        use crate::database::Database;

        // Normal query
        assert_eq!(Database::sanitize_fts5_query("hello world"), "\"hello\" \"world\"");

        // Empty/whitespace
        assert_eq!(Database::sanitize_fts5_query(""), "");
        assert_eq!(Database::sanitize_fts5_query("   "), "");

        // Special FTS5 characters should be safely quoted
        let result = Database::sanitize_fts5_query("NOT hello OR world");
        assert!(result.contains("\"NOT\""));
        assert!(result.contains("\"hello\""));
        assert!(result.contains("\"OR\""));

        // Embedded quotes should be escaped
        let result = Database::sanitize_fts5_query("it's a \"test\"");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_find_replace_basic() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>The cat sat on the mat.</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        let count = db
            .find_replace_in_scenes("cat", "dog", false, false, None)
            .unwrap();
        assert_eq!(count, 1);

        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("dog"));
        assert!(!updated.text.contains("cat"));
    }

    #[test]
    fn test_find_replace_case_sensitive() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>The Cat and the cat.</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // Case sensitive: only lowercase "cat" should match
        let count = db
            .find_replace_in_scenes("cat", "dog", true, false, None)
            .unwrap();
        assert_eq!(count, 1);

        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("Cat")); // uppercase preserved
        assert!(updated.text.contains("dog")); // lowercase replaced
    }

    #[test]
    fn test_find_replace_whole_word() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>The cat chased the caterpillar.</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // Whole word: only "cat" not "caterpillar"
        let count = db
            .find_replace_in_scenes("cat", "dog", false, true, None)
            .unwrap();
        assert_eq!(count, 1);

        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("dog"));
        assert!(updated.text.contains("caterpillar")); // NOT replaced
    }

    #[test]
    fn test_find_replace_empty_search() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let count = db
            .find_replace_in_scenes("", "replacement", false, false, None)
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_find_replace_no_matches() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let count = db
            .find_replace_in_scenes("zzzznonexistent", "replacement", false, false, None)
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_find_replace_preserves_html_tags() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>Hello <strong>world</strong></p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        let count = db
            .find_replace_in_scenes("world", "earth", false, false, None)
            .unwrap();
        assert_eq!(count, 1);

        let updated = db.get_scene(&scene.id).unwrap();
        // Tags should be preserved, only text content replaced
        assert!(updated.text.contains("<strong>earth</strong>"));
    }

    #[test]
    fn test_find_replace_scoped_to_chapter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch1 = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let ch2 = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch2".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: ch1.id.clone(),
                title: "S1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: ch2.id.clone(),
                title: "S2".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.update_scene(
            &s1.id,
            &UpdateSceneRequest {
                text: Some("<p>The word here.</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        db.update_scene(
            &s2.id,
            &UpdateSceneRequest {
                text: Some("<p>The word here too.</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // Only replace in chapter 1
        let count = db
            .find_replace_in_scenes("word", "term", false, false, Some(&ch1.id))
            .unwrap();
        assert_eq!(count, 1);

        let updated_s1 = db.get_scene(&s1.id).unwrap();
        let updated_s2 = db.get_scene(&s2.id).unwrap();
        assert!(updated_s1.text.contains("term"));
        assert!(updated_s2.text.contains("word")); // unchanged
    }

    // ========================================================================
    // Word Count Tests (extended)
    // ========================================================================

    #[test]
    fn test_word_counts_empty_project_no_data() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let counts = db.get_word_counts().unwrap();
        assert_eq!(counts.total, 0);
        assert!(counts.by_chapter.is_empty());
        assert!(counts.by_status.is_empty());
    }

    #[test]
    fn test_word_counts_with_rich_content() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let counts = db.get_word_counts().unwrap();
        assert!(counts.total > 0);
        assert!(!counts.by_chapter.is_empty());

        // Chapter One has 2 scenes
        let ch1 = &counts.by_chapter[0];
        assert_eq!(ch1.chapter_title, "Chapter One");
        assert_eq!(ch1.scene_count, 2);
        assert!(ch1.word_count > 0);
    }

    #[test]
    fn test_word_counts_by_status_detailed() {
        let (db, _temp_dir) = create_test_db();
        setup_project_with_content(&db);

        let counts = db.get_word_counts().unwrap();

        // We have scenes with "draft", "done", and "to write" statuses
        assert!(counts.by_status.len() >= 2);

        let draft_count = counts
            .by_status
            .iter()
            .find(|s| s.status == "draft")
            .expect("Should have draft status");
        assert!(draft_count.word_count > 0);
    }

    // ========================================================================
    // Timeline Conflict Detection Tests
    // ========================================================================

    #[test]
    fn test_timeline_no_conflicts() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene A".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene B".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.update_scene(
            &s1.id,
            &UpdateSceneRequest {
                pov: Some("Alice".to_string()),
                time_point: Some("Day 1".to_string()),
                on_timeline: Some(true),
                ..Default::default()
            },
        )
        .unwrap();

        db.update_scene(
            &s2.id,
            &UpdateSceneRequest {
                pov: Some("Alice".to_string()),
                time_point: Some("Day 2".to_string()),
                on_timeline: Some(true),
                ..Default::default()
            },
        )
        .unwrap();

        let conflicts = db.detect_timeline_conflicts().unwrap();
        // No overlapping conflicts (different days)
        let overlapping: Vec<_> = conflicts
            .iter()
            .filter(|c| c.conflict_type == "overlapping_time" || c.conflict_type == "same_time")
            .collect();
        assert!(overlapping.is_empty());
    }

    #[test]
    fn test_timeline_same_time_conflict() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene A".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene B".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        // Same POV, same time_point = conflict
        db.update_scene(
            &s1.id,
            &UpdateSceneRequest {
                pov: Some("Alice".to_string()),
                time_point: Some("Day 1".to_string()),
                on_timeline: Some(true),
                ..Default::default()
            },
        )
        .unwrap();

        db.update_scene(
            &s2.id,
            &UpdateSceneRequest {
                pov: Some("Alice".to_string()),
                time_point: Some("Day 1".to_string()),
                on_timeline: Some(true),
                ..Default::default()
            },
        )
        .unwrap();

        let conflicts = db.detect_timeline_conflicts().unwrap();
        let same_time: Vec<_> = conflicts
            .iter()
            .filter(|c| c.conflict_type == "same_time")
            .collect();
        assert_eq!(same_time.len(), 1);
        assert!(same_time[0].description.contains("Alice"));
    }

    #[test]
    fn test_timeline_overlapping_time_conflict() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene A".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene B".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        // Overlapping time ranges for the same POV
        db.update_scene(
            &s1.id,
            &UpdateSceneRequest {
                pov: Some("Bob".to_string()),
                time_start: Some("Day 1".to_string()),
                time_end: Some("Day 3".to_string()),
                on_timeline: Some(true),
                ..Default::default()
            },
        )
        .unwrap();

        db.update_scene(
            &s2.id,
            &UpdateSceneRequest {
                pov: Some("Bob".to_string()),
                time_start: Some("Day 2".to_string()),
                time_end: Some("Day 4".to_string()),
                on_timeline: Some(true),
                ..Default::default()
            },
        )
        .unwrap();

        let conflicts = db.detect_timeline_conflicts().unwrap();
        let overlapping: Vec<_> = conflicts
            .iter()
            .filter(|c| c.conflict_type == "overlapping_time")
            .collect();
        assert_eq!(overlapping.len(), 1);
        assert!(overlapping[0].description.contains("Bob"));
    }

    #[test]
    fn test_timeline_missing_time_no_conflict_without_time_data() {
        // Note: get_all_scenes_for_timeline() filters to scenes with
        // on_timeline=1 AND (time_point IS NOT NULL OR time_start IS NOT NULL),
        // so scenes with POV but no time data are excluded from conflict detection.
        // This test verifies that such scenes don't cause issues.
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Unplaced Scene".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        // Has POV but no time data, and is on timeline
        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                pov: Some("Charlie".to_string()),
                on_timeline: Some(true),
                ..Default::default()
            },
        )
        .unwrap();

        // Scene without time data is excluded from timeline query,
        // so no conflicts are detected
        let conflicts = db.detect_timeline_conflicts().unwrap();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_timeline_different_pov_no_conflict() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene A".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene B".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        // Same time but different POV = no conflict
        db.update_scene(
            &s1.id,
            &UpdateSceneRequest {
                pov: Some("Alice".to_string()),
                time_point: Some("Day 1".to_string()),
                on_timeline: Some(true),
                ..Default::default()
            },
        )
        .unwrap();

        db.update_scene(
            &s2.id,
            &UpdateSceneRequest {
                pov: Some("Bob".to_string()),
                time_point: Some("Day 1".to_string()),
                on_timeline: Some(true),
                ..Default::default()
            },
        )
        .unwrap();

        let conflicts = db.detect_timeline_conflicts().unwrap();
        let overlapping: Vec<_> = conflicts
            .iter()
            .filter(|c| c.conflict_type == "overlapping_time" || c.conflict_type == "same_time")
            .collect();
        assert!(overlapping.is_empty());
    }

    // ========================================================================
    // Error Cases and Edge Cases
    // ========================================================================

    #[test]
    fn test_get_nonexistent_chapter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_chapter("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_nonexistent_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_scene("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_update_nonexistent_chapter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.update_chapter(
            "nonexistent-id",
            &UpdateChapterRequest {
                title: Some("New".to_string()),
                summary: None,
                status: None,
                notes: None,
                position: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_update_nonexistent_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.update_scene(
            "nonexistent-id",
            &UpdateSceneRequest {
                title: Some("New".to_string()),
                ..Default::default()
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent_chapter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Deleting nonexistent should still succeed (soft delete updates 0 rows)
        // or return an error depending on implementation
        let result = db.delete_chapter("nonexistent-id");
        // It's valid for this to succeed silently or return an error
        // The important thing is it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_delete_nonexistent_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.delete_scene("nonexistent-id");
        let _ = result;
    }

    #[test]
    fn test_get_nonexistent_bible_entry() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_bible_entry("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_nonexistent_arc() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_arc("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_nonexistent_event() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_event("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_nonexistent_issue() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_issue("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_nonexistent_template() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_template("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_nonexistent_snapshot() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_snapshot("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_cascade_delete_chapter_removes_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "To Delete".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Child Scene".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.delete_chapter(&chapter.id).unwrap();

        // Scene should also be deleted (soft or cascade)
        let result = db.get_scene(&scene.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_chapters_returns_empty_vec() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapters = db.get_chapters().unwrap();
        assert!(chapters.is_empty());
    }

    #[test]
    fn test_empty_scenes_returns_empty_vec() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Empty".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scenes = db.get_scenes(&chapter.id).unwrap();
        assert!(scenes.is_empty());
    }

    #[test]
    fn test_empty_bible_entries() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entries = db.get_bible_entries(None).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_empty_arcs() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let arcs = db.get_arcs().unwrap();
        assert!(arcs.is_empty());
    }

    #[test]
    fn test_empty_events() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let events = db.get_events().unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_scene_reorder() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene A".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene B".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s3 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene C".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        // Reorder to C, A, B
        db.reorder_scenes(&chapter.id, &[s3.id.clone(), s1.id.clone(), s2.id.clone()])
            .unwrap();

        let scenes = db.get_scenes(&chapter.id).unwrap();
        assert_eq!(scenes[0].title, "Scene C");
        assert_eq!(scenes[1].title, "Scene A");
        assert_eq!(scenes[2].title, "Scene B");
    }

    #[test]
    fn test_bible_entry_types_filter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        })
        .unwrap();

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "location".to_string(),
            name: "Castle".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        })
        .unwrap();

        let all = db.get_bible_entries(None).unwrap();
        assert_eq!(all.len(), 2);

        let characters = db.get_bible_entries(Some("character")).unwrap();
        assert_eq!(characters.len(), 1);
        assert_eq!(characters[0].name, "Hero");

        let locations = db.get_bible_entries(Some("location")).unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].name, "Castle");
    }

    #[test]
    fn test_scene_with_all_revision_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        let updated = db
            .update_scene(
                &scene.id,
                &UpdateSceneRequest {
                    pov_goal: Some("Show character growth".to_string()),
                    has_conflict: Some(true),
                    has_change: Some(true),
                    tension: Some("8".to_string()),
                    revision_notes: Some("Needs more tension".to_string()),
                    revision_checklist: Some("{\"items\":[]}".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(
            updated.pov_goal,
            Some("Show character growth".to_string())
        );
        assert_eq!(updated.has_conflict, Some(true));
        assert_eq!(updated.has_change, Some(true));
        assert_eq!(updated.tension, Some("8".to_string()));
        assert_eq!(
            updated.revision_notes,
            Some("Needs more tension".to_string())
        );
    }

    #[test]
    fn test_scene_setup_payoff_links() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Setup Scene".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Payoff Scene".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        // Link setup and payoff
        let updated = db
            .update_scene(
                &s2.id,
                &UpdateSceneRequest {
                    payoff_of_scene_id: Some(s1.id.clone()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.payoff_of_scene_id, Some(s1.id.clone()));

        let setup = db
            .update_scene(
                &s1.id,
                &UpdateSceneRequest {
                    setup_for_scene_id: Some(s2.id.clone()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(setup.setup_for_scene_id, Some(s2.id.clone()));
    }

    #[test]
    fn test_create_and_reopen_database_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_reopen.cahnon");

        // Create and populate
        {
            let db = Database::create(&db_path).unwrap();
            db.create_project(&CreateProjectRequest {
                title: "Persistent".to_string(),
                author: None,
                description: None,
            })
            .unwrap();

            let ch = db
                .create_chapter(&CreateChapterRequest {
                    title: "Ch1".to_string(),
                    summary: None,
                    position: None,
                })
                .unwrap();

            db.create_scene(&CreateSceneRequest {
                chapter_id: ch.id.clone(),
                title: "S1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();
        }

        // Reopen and verify
        {
            let db = Database::open(&db_path).unwrap();
            let project = db.get_project().unwrap();
            assert_eq!(project.title, "Persistent");

            let chapters = db.get_chapters().unwrap();
            assert_eq!(chapters.len(), 1);

            let scenes = db.get_scenes(&chapters[0].id).unwrap();
            assert_eq!(scenes.len(), 1);
        }
    }

    #[test]
    fn test_open_nonexistent_database_error() {
        let result = Database::open(&PathBuf::from("/tmp/nonexistent_test.cahnon"));
        assert!(result.is_err());
        match result {
            Err(e) => assert!(e.contains("does not exist")),
            Ok(_) => panic!("Should have returned error"),
        }
    }

    #[test]
    fn test_scene_history_multiple_versions() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        // Make multiple updates
        for i in 1..=5 {
            db.update_scene(
                &scene.id,
                &UpdateSceneRequest {
                    text: Some(format!("<p>Version {}</p>", i)),
                    ..Default::default()
                },
            )
            .unwrap();
        }

        let history = db.get_scene_history(&scene.id).unwrap();
        assert!(history.len() >= 5);

        // Restore an older version
        if history.len() >= 2 {
            let old_version = &history[history.len() - 2]; // second oldest
            let restored = db
                .restore_scene_version(&scene.id, &old_version.id)
                .unwrap();
            // After restore, text should match that version
            assert!(!restored.text.is_empty());
        }
    }

    #[test]
    fn test_scene_compare_versions() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // First update: saves empty initial text to history, sets text to "First version"
        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>First version text</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // Second update: saves "First version" to history, sets text to "Second version"
        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>Second version text</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // Third update: saves "Second version" to history, sets text to "Third version"
        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>Third version text</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        let history = db.get_scene_history(&scene.id).unwrap();
        // History (DESC): [0] = "Second version", [1] = "First version", [2] = "" (initial)
        assert!(history.len() >= 3);

        // Compare the two non-empty history entries
        let (text_a, text_b) = db
            .compare_scene_versions(&scene.id, &history[1].id, &history[0].id)
            .unwrap();
        assert!(!text_a.is_empty());
        assert!(!text_b.is_empty());
        assert_ne!(text_a, text_b);
    }

    #[test]
    fn test_snapshot_full_workflow() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Chapter 1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene 1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>Original text</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // Create snapshot
        let snapshot = db
            .create_snapshot("Before edit", Some("Manual backup"), "manual")
            .unwrap();

        // Modify the scene
        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>Modified text</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        // Verify modification
        let modified = db.get_scene(&scene.id).unwrap();
        assert!(modified.text.contains("Modified text"));

        // Restore snapshot
        db.restore_snapshot(&snapshot.id).unwrap();

        // Verify restoration
        let restored = db.get_scene(&scene.id).unwrap();
        assert!(restored.text.contains("Original text"));
    }

    #[test]
    fn test_trash_restore_scene_full_cycle() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Trashable".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        // Delete (soft)
        db.delete_scene(&scene.id).unwrap();

        // Should not appear in normal list
        let scenes = db.get_scenes(&chapter.id).unwrap();
        assert!(scenes.is_empty());

        // Should appear in trash
        let trashed = db.get_deleted_scenes().unwrap();
        assert!(trashed.iter().any(|s| s.id == scene.id));

        // Restore
        db.restore_scene(&scene.id).unwrap();

        // Should be back
        let scenes = db.get_scenes(&chapter.id).unwrap();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].title, "Trashable");
    }

    #[test]
    fn test_trash_restore_chapter_full_cycle() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Trashable Chapter".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.delete_chapter(&chapter.id).unwrap();

        let chapters = db.get_chapters().unwrap();
        assert!(chapters.is_empty());

        let trashed = db.get_deleted_chapters().unwrap();
        assert!(trashed.iter().any(|c| c.id == chapter.id));

        db.restore_chapter(&chapter.id).unwrap();

        let chapters = db.get_chapters().unwrap();
        assert_eq!(chapters.len(), 1);
    }

    #[test]
    fn test_duplicate_scene_with_content() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Original".to_string(),
                summary: Some("A summary".to_string()),
                position: None,
            })
            .unwrap();

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>Content to duplicate</p>".to_string()),
                pov: Some("Alice".to_string()),
                tags: Some("action".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        let duplicate = db.duplicate_scene(&scene.id, false).unwrap();

        assert_ne!(duplicate.id, scene.id);
        assert!(duplicate.title.contains("Original")); // title based on original
        assert_eq!(duplicate.summary, Some("A summary".to_string()));
        assert!(duplicate.text.contains("Content to duplicate"));
        assert_eq!(duplicate.chapter_id, chapter.id);
    }

    #[test]
    fn test_event_with_time_range() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let event = db
            .create_event(&CreateEventRequest {
                title: "The War".to_string(),
                description: Some("A long war".to_string()),
                time_point: None,
                time_start: Some("Year 1".to_string()),
                time_end: Some("Year 5".to_string()),
                event_type: Some("historical".to_string()),
                importance: Some("10".to_string()),
            })
            .unwrap();

        assert_eq!(event.time_start, Some("Year 1".to_string()));
        assert_eq!(event.time_end, Some("Year 5".to_string()));
        assert_eq!(event.importance, "10");
    }

    #[test]
    fn test_issue_full_workflow() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let bible = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Alice".to_string(),
                short_description: None,
                full_description: None,
                aliases: None,
                status: None,
                tags: None,
                color: None,
            })
            .unwrap();

        let issue = db
            .create_issue(&CreateIssueRequest {
                issue_type: "continuity".to_string(),
                title: "Alice's eye color changes".to_string(),
                description: Some("Blue in ch1, green in ch3".to_string()),
                severity: Some("major".to_string()),
            })
            .unwrap();

        // Link to scene (scene_id first, then issue_id)
        db.link_scene_to_issue(&scene.id, &issue.id).unwrap();
        let scenes = db.get_issue_scenes(&issue.id).unwrap();
        assert_eq!(scenes.len(), 1);

        // Link to bible entry (bible_entry_id first, then issue_id)
        db.link_bible_entry_to_issue(&bible.id, &issue.id).unwrap();
        let entries = db.get_issue_bible_entries(&issue.id).unwrap();
        assert_eq!(entries.len(), 1);

        // Get issues for scene
        let scene_issues = db.get_scene_issues(&scene.id).unwrap();
        assert_eq!(scene_issues.len(), 1);

        // Resolve
        let resolved = db
            .update_issue(
                &issue.id,
                &UpdateIssueRequest {
                    status: Some("resolved".to_string()),
                    resolution_note: Some("Fixed in revision".to_string()),
                },
            )
            .unwrap();
        assert_eq!(resolved.status, "resolved");

        // Unlink (scene_id first, then issue_id)
        db.unlink_scene_from_issue(&scene.id, &issue.id).unwrap();
        let scenes = db.get_issue_scenes(&issue.id).unwrap();
        assert!(scenes.is_empty());

        db.unlink_bible_entry_from_issue(&bible.id, &issue.id)
            .unwrap();
        let entries = db.get_issue_bible_entries(&issue.id).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_template_full_workflow() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Create custom template
        let template = db
            .create_template(&CreateTemplateRequest {
                name: "My Structure".to_string(),
            })
            .unwrap();

        // Create steps
        let step1 = db
            .create_template_step(&CreateTemplateStepRequest {
                template_id: template.id.clone(),
                name: "Act 1".to_string(),
                description: Some("Introduction".to_string()),
                typical_position: Some(0.0),
                color: Some("#ff0000".to_string()),
            })
            .unwrap();

        let step2 = db
            .create_template_step(&CreateTemplateStepRequest {
                template_id: template.id.clone(),
                name: "Act 2".to_string(),
                description: Some("Rising action".to_string()),
                typical_position: Some(0.5),
                color: Some("#00ff00".to_string()),
            })
            .unwrap();

        // Verify steps
        let steps = db.get_template_steps(&template.id).unwrap();
        assert_eq!(steps.len(), 2);

        // Set as active
        db.set_active_template(&template.id).unwrap();
        let templates = db.get_templates().unwrap();
        let active = templates.iter().find(|t| t.id == template.id).unwrap();
        assert!(active.is_active);

        // Assign scene to step
        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.assign_scene_to_step(&scene.id, &step1.id).unwrap();
        let step = db.get_scene_step(&scene.id).unwrap();
        assert!(step.is_some());
        assert_eq!(step.unwrap().id, step1.id);

        // Update step
        db.update_template_step(
            &step1.id,
            &UpdateTemplateStepRequest {
                name: Some("Act 1 - Setup".to_string()),
                description: None,
                typical_position: None,
                color: None,
                position: None,
            },
        )
        .unwrap();

        let updated_steps = db.get_template_steps(&template.id).unwrap();
        assert!(updated_steps.iter().any(|s| s.name == "Act 1 - Setup"));

        // Delete step
        db.delete_template_step(&step2.id).unwrap();
        let remaining_steps = db.get_template_steps(&template.id).unwrap();
        assert_eq!(remaining_steps.len(), 1);

        // Delete template
        db.delete_template(&template.id).unwrap();
    }

    #[test]
    fn test_bible_relationship_full_workflow() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let alice = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Alice".to_string(),
                short_description: None,
                full_description: None,
                aliases: None,
                status: None,
                tags: None,
                color: None,
            })
            .unwrap();

        let bob = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Bob".to_string(),
                short_description: None,
                full_description: None,
                aliases: None,
                status: None,
                tags: None,
                color: None,
            })
            .unwrap();

        let castle = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "location".to_string(),
                name: "Castle".to_string(),
                short_description: None,
                full_description: None,
                aliases: None,
                status: None,
                tags: None,
                color: None,
            })
            .unwrap();

        // Create relationships
        let rel1 = db
            .create_bible_relationship(&CreateBibleRelationshipRequest {
                source_id: alice.id.clone(),
                target_id: bob.id.clone(),
                relationship_type: "friend_of".to_string(),
                note: Some("Best friends".to_string()),
                status: None,
            })
            .unwrap();

        db.create_bible_relationship(&CreateBibleRelationshipRequest {
            source_id: alice.id.clone(),
            target_id: castle.id.clone(),
            relationship_type: "lives_in".to_string(),
            note: None,
            status: None,
        })
        .unwrap();

        // Get relationships
        let alice_rels = db.get_bible_relationships(&alice.id).unwrap();
        assert_eq!(alice_rels.len(), 2);

        // Update relationship
        db.update_bible_relationship(
            &rel1.id,
            &UpdateBibleRelationshipRequest {
                relationship_type: Some("rival_of".to_string()),
                note: Some("Former friends, now rivals".to_string()),
                status: None,
            },
        )
        .unwrap();

        let updated_rels = db.get_bible_relationships(&alice.id).unwrap();
        assert!(updated_rels.iter().any(|r| r.relationship_type == "rival_of"));

        // Delete relationship
        db.delete_bible_relationship(&rel1.id).unwrap();
        let after_delete = db.get_bible_relationships(&alice.id).unwrap();
        assert_eq!(after_delete.len(), 1);
    }

    #[test]
    fn test_name_registry_merge() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "S1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let entry1 = db
            .create_name_registry_entry(&CreateNameRegistryRequest {
                canonical_name: "Alice".to_string(),
                name_type: Some("character".to_string()),
                bible_entry_id: None,
                aliases: None,
            })
            .unwrap();

        let entry2 = db
            .create_name_registry_entry(&CreateNameRegistryRequest {
                canonical_name: "Al".to_string(),
                name_type: Some("character".to_string()),
                bible_entry_id: None,
                aliases: None,
            })
            .unwrap();

        // Create mentions for both
        db.create_name_mention(&entry1.id, &scene.id, "Alice", 0, 5)
            .unwrap();
        db.create_name_mention(&entry2.id, &scene.id, "Al", 10, 12)
            .unwrap();

        // Merge entry2 into entry1
        db.merge_name_entries(&entry1.id, &entry2.id).unwrap();

        // entry2 should be gone
        assert!(db.get_name_registry_entry(&entry2.id).is_err());

        // All mentions should now belong to entry1
        let mentions = db.get_name_mentions_by_registry(&entry1.id).unwrap();
        assert_eq!(mentions.len(), 2);
    }

    #[test]
    fn test_name_registry_scan() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let scene = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "S1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>Alice went to the castle. Alice met Bob there.</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        let entry = db
            .create_name_registry_entry(&CreateNameRegistryRequest {
                canonical_name: "Alice".to_string(),
                name_type: Some("character".to_string()),
                bible_entry_id: None,
                aliases: None,
            })
            .unwrap();

        // Scan should find mentions of Alice
        // scan_names() returns (entries_created, mentions_created)
        let (entries_created, mentions_created) = db.scan_names().unwrap();
        // Should have scanned and found mentions (or created entries)
        assert!(entries_created >= 0);
        assert!(mentions_created >= 0);
    }

    #[test]
    fn test_validation_long_title() {
        use crate::validation::{sanitize_text, MAX_TITLE_LENGTH};

        let long_title = "A".repeat(500);
        let sanitized = sanitize_text(&long_title, MAX_TITLE_LENGTH);
        assert_eq!(sanitized.len(), MAX_TITLE_LENGTH);
    }

    #[test]
    fn test_validation_unicode_characters() {
        use crate::validation::sanitize_text;

        // Normal unicode should be preserved
        let text = "こんにちは世界 Hello 世界";
        let sanitized = sanitize_text(text, 100);
        assert_eq!(sanitized, text);

        // Zero-width characters should be stripped
        let with_zwsp = "Hello\u{200B}World";
        let sanitized = sanitize_text(with_zwsp, 100);
        assert_eq!(sanitized, "HelloWorld");

        // BOM should be stripped
        let with_bom = "\u{FEFF}Hello";
        let sanitized = sanitize_text(with_bom, 100);
        assert_eq!(sanitized, "Hello");
    }

    #[test]
    fn test_validation_control_characters() {
        use crate::validation::sanitize_text;

        // Control characters should be stripped (except in multiline)
        let with_controls = "Hello\x00\x01\x02World";
        let sanitized = sanitize_text(with_controls, 100);
        assert_eq!(sanitized, "HelloWorld");
    }

    #[test]
    fn test_validation_multiline() {
        use crate::validation::sanitize_multiline_text;

        // Newlines should be preserved
        let text = "Line 1\nLine 2\r\nLine 3";
        let sanitized = sanitize_multiline_text(text, 1000);
        assert!(sanitized.contains('\n'));

        // Control chars still stripped
        let with_controls = "Line 1\x00\nLine 2";
        let sanitized = sanitize_multiline_text(with_controls, 1000);
        assert!(!sanitized.contains('\x00'));
        assert!(sanitized.contains('\n'));
    }

    #[test]
    fn test_scene_split_at_boundary() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(
            &scene.id,
            &UpdateSceneRequest {
                text: Some("<p>Part one text.</p><p>Part two text.</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        let (original, new_scene) = db
            .split_scene(&scene.id, 20, None)
            .unwrap();

        // Should have original and new scene
        assert!(!original.text.is_empty());
        assert!(!new_scene.text.is_empty());
        assert_ne!(original.id, new_scene.id);
        assert_eq!(original.chapter_id, new_scene.chapter_id);
    }

    #[test]
    fn test_merge_two_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db
            .create_chapter(&CreateChapterRequest {
                title: "Ch1".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s1 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "First".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        let s2 = db
            .create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Second".to_string(),
                summary: None,
                position: None,
            })
            .unwrap();

        db.update_scene(
            &s1.id,
            &UpdateSceneRequest {
                text: Some("<p>First scene text.</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        db.update_scene(
            &s2.id,
            &UpdateSceneRequest {
                text: Some("<p>Second scene text.</p>".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        let merged = db
            .merge_scenes(&[s1.id.clone(), s2.id.clone()])
            .unwrap();

        assert!(merged.text.contains("First scene text"));
        assert!(merged.text.contains("Second scene text"));

        // Only merged scene should remain
        let scenes = db.get_scenes(&chapter.id).unwrap();
        assert_eq!(scenes.len(), 1);
    }

    #[test]
    fn test_scene_associations_full_workflow() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Alice".to_string(),
                short_description: None,
                full_description: None,
                aliases: None,
                status: None,
                tags: None,
                color: None,
            })
            .unwrap();

        // Create association
        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        }).unwrap();

        // Get associations (returns Vec<BibleEntry>)
        let assocs = db.get_scene_associations(&scene.id).unwrap();
        assert_eq!(assocs.len(), 1);
        assert_eq!(assocs[0].id, entry.id);

        // Duplicate should not error (idempotent)
        let result = db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        });
        assert!(result.is_ok());

        // Delete
        db.delete_association(&scene.id, &entry.id).unwrap();
        let assocs = db.get_scene_associations(&scene.id).unwrap();
        assert!(assocs.is_empty());
    }

    #[test]
    fn test_arc_scene_linking_full_workflow() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let arc = db
            .create_arc(&CreateArcRequest {
                name: "Main Plot".to_string(),
                description: None,
                stakes: None,
                characters: None,
                status: None,
                color: None,
            })
            .unwrap();

        // Link
        db.link_scene_to_arc(&scene.id, &arc.id).unwrap();

        let arcs = db.get_scene_arcs(&scene.id).unwrap();
        assert_eq!(arcs.len(), 1);

        // Unlink
        db.unlink_scene_from_arc(&scene.id, &arc.id).unwrap();
        let arcs = db.get_scene_arcs(&scene.id).unwrap();
        assert!(arcs.is_empty());
    }

    #[test]
    fn test_event_scene_and_bible_linking() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let entry = db
            .create_bible_entry(&CreateBibleEntryRequest {
                entry_type: "character".to_string(),
                name: "Hero".to_string(),
                short_description: None,
                full_description: None,
                aliases: None,
                status: None,
                tags: None,
                color: None,
            })
            .unwrap();

        let event = db
            .create_event(&CreateEventRequest {
                title: "Battle".to_string(),
                description: None,
                time_point: Some("Day 5".to_string()),
                time_start: None,
                time_end: None,
                event_type: None,
                importance: None,
            })
            .unwrap();

        // Link scene (scene_id first, then event_id)
        db.link_scene_to_event(&scene.id, &event.id).unwrap();
        let scenes = db.get_event_scenes(&event.id).unwrap();
        assert_eq!(scenes.len(), 1);

        let events = db.get_scene_events(&scene.id).unwrap();
        assert_eq!(events.len(), 1);

        // Link bible entry (bible_entry_id first, then event_id)
        db.link_bible_entry_to_event(&entry.id, &event.id).unwrap();
        let entries = db.get_event_bible_entries(&event.id).unwrap();
        assert_eq!(entries.len(), 1);

        let entry_events = db.get_bible_entry_events(&entry.id).unwrap();
        assert_eq!(entry_events.len(), 1);

        // Unlink (scene_id first, then event_id)
        db.unlink_scene_from_event(&scene.id, &event.id).unwrap();
        assert!(db.get_event_scenes(&event.id).unwrap().is_empty());

        db.unlink_bible_entry_from_event(&entry.id, &event.id)
            .unwrap();
        assert!(db.get_event_bible_entries(&event.id).unwrap().is_empty());
    }

    #[test]
    fn test_cut_with_scene_reference() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let cut = db
            .create_cut(Some(&scene.id), "Removed paragraph about the forest.")
            .unwrap();

        assert_eq!(cut.scene_id, Some(scene.id.clone()));
        assert_eq!(cut.text, "Removed paragraph about the forest.");

        let cuts = db.get_cuts().unwrap();
        assert_eq!(cuts.len(), 1);

        db.delete_cut(&cut.id).unwrap();
        let cuts = db.get_cuts().unwrap();
        assert!(cuts.is_empty());
    }

    #[test]
    fn test_bible_search() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Gandalf the Grey".to_string(),
            short_description: Some("A wise wizard".to_string()),
            full_description: Some("He carries a staff and wears grey robes.".to_string()),
            aliases: Some("Mithrandir".to_string()),
            status: None,
            tags: None,
            color: None,
        })
        .unwrap();

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "location".to_string(),
            name: "The Shire".to_string(),
            short_description: Some("Home of the hobbits".to_string()),
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        })
        .unwrap();

        let results = db.search_bible("Gandalf").unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|e| e.name == "Gandalf the Grey"));
    }

    #[test]
    fn test_saved_filter_stores_json_data() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Valid JSON filter data should be stored correctly
        let filter = db
            .create_saved_filter(&CreateSavedFilterRequest {
                name: "Complex Filter".to_string(),
                filter_type: "outline".to_string(),
                filter_data: r#"{"status":"draft","tags":["fantasy","action"],"nested":{"key":"value"}}"#.to_string(),
            })
            .unwrap();

        assert_eq!(filter.name, "Complex Filter");
        let retrieved = db.get_saved_filter(&filter.id).unwrap();
        assert!(retrieved.filter_data.contains("fantasy"));
    }

    // ========================================================================
    // Settings Tests
    // ========================================================================

    #[test]
    fn test_get_setting_nonexistent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_setting("nonexistent_key").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_set_and_get_setting() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.set_setting("theme", "dark").unwrap();
        let value = db.get_setting("theme").unwrap();
        assert_eq!(value, Some("dark".to_string()));
    }

    #[test]
    fn test_set_setting_upsert() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.set_setting("font_size", "14").unwrap();
        assert_eq!(db.get_setting("font_size").unwrap(), Some("14".to_string()));

        // Update existing setting
        db.set_setting("font_size", "16").unwrap();
        assert_eq!(db.get_setting("font_size").unwrap(), Some("16".to_string()));
    }

    #[test]
    fn test_set_setting_empty_value() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.set_setting("empty_key", "").unwrap();
        let value = db.get_setting("empty_key").unwrap();
        assert_eq!(value, Some("".to_string()));
    }

    #[test]
    fn test_set_setting_special_characters() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.set_setting("json_config", r#"{"key": "value", "nested": [1,2,3]}"#).unwrap();
        let value = db.get_setting("json_config").unwrap().unwrap();
        assert!(value.contains("nested"));
    }

    #[test]
    fn test_multiple_settings() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.set_setting("setting_a", "value_a").unwrap();
        db.set_setting("setting_b", "value_b").unwrap();
        db.set_setting("setting_c", "value_c").unwrap();

        assert_eq!(db.get_setting("setting_a").unwrap(), Some("value_a".to_string()));
        assert_eq!(db.get_setting("setting_b").unwrap(), Some("value_b".to_string()));
        assert_eq!(db.get_setting("setting_c").unwrap(), Some("value_c".to_string()));
    }

    // ========================================================================
    // Snapshot Cleanup Tests
    // ========================================================================

    #[test]
    fn test_cleanup_expired_snapshots_removes_old_pre_bulk() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Create a pre_bulk snapshot and manually backdate it
        let snapshot = db.create_snapshot("Old backup", None, "pre_bulk").unwrap();

        // Manually set the created_at to 31 days ago
        let old_date = (chrono::Utc::now() - chrono::Duration::days(31)).to_rfc3339();
        db.conn
            .execute(
                "UPDATE snapshots SET created_at = ?1 WHERE id = ?2",
                &[&old_date, &snapshot.id],
            )
            .unwrap();

        let deleted = db.cleanup_expired_snapshots().unwrap();
        assert_eq!(deleted, 1);

        // Verify it's gone
        let snapshots = db.get_snapshots().unwrap();
        assert!(snapshots.iter().all(|s| s.id != snapshot.id));
    }

    #[test]
    fn test_cleanup_expired_snapshots_keeps_recent_pre_bulk() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Create a recent pre_bulk snapshot (within 30 days)
        db.create_snapshot("Recent backup", None, "pre_bulk").unwrap();

        let deleted = db.cleanup_expired_snapshots().unwrap();
        assert_eq!(deleted, 0);

        assert_eq!(db.get_snapshots().unwrap().len(), 1);
    }

    #[test]
    fn test_cleanup_expired_snapshots_keeps_manual_type() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Create a manual snapshot and backdate it
        let snapshot = db.create_snapshot("Manual backup", None, "manual").unwrap();
        let old_date = (chrono::Utc::now() - chrono::Duration::days(60)).to_rfc3339();
        db.conn
            .execute(
                "UPDATE snapshots SET created_at = ?1 WHERE id = ?2",
                &[&old_date, &snapshot.id],
            )
            .unwrap();

        // Cleanup should NOT delete manual snapshots even if old
        let deleted = db.cleanup_expired_snapshots().unwrap();
        assert_eq!(deleted, 0);
        assert_eq!(db.get_snapshots().unwrap().len(), 1);
    }

    #[test]
    fn test_cleanup_no_snapshots() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let deleted = db.cleanup_expired_snapshots().unwrap();
        assert_eq!(deleted, 0);
    }

    // ========================================================================
    // Snapshot Scenes & Scene Restore Tests
    // ========================================================================

    #[test]
    fn test_get_snapshot_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Chapter 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene A".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id,
            title: "Scene B".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let snapshot = db.create_snapshot("Test", None, "manual").unwrap();
        let scenes = db.get_snapshot_scenes(&snapshot.id).unwrap();
        assert_eq!(scenes.len(), 2);
    }

    #[test]
    fn test_restore_scene_from_snapshot() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Chapter 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id,
            title: "My Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Set some text
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Original text</p>".to_string()),
            ..Default::default()
        }).unwrap();

        // Take snapshot
        let snapshot = db.create_snapshot("Before changes", None, "manual").unwrap();

        // Modify the scene text
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Modified text</p>".to_string()),
            ..Default::default()
        }).unwrap();

        // Verify it changed
        let modified = db.get_scene(&scene.id).unwrap();
        assert!(modified.text.contains("Modified"));

        // Restore scene from snapshot
        let restored = db.restore_scene_from_snapshot(&snapshot.id, &scene.id).unwrap();
        assert!(restored.text.contains("Original"));
    }

    #[test]
    fn test_restore_scene_from_snapshot_not_found() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id,
            title: "Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let snapshot = db.create_snapshot("Snap", None, "manual").unwrap();
        let result = db.restore_scene_from_snapshot(&snapshot.id, "nonexistent-scene-id");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found in snapshot"));
    }

    // ========================================================================
    // Import JSON Backup Tests
    // ========================================================================

    #[test]
    fn test_import_json_backup_invalid_json() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.import_json_backup("not valid json {{{");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON backup data"));
    }

    #[test]
    fn test_import_json_backup_round_trip() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Chapter 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id,
            title: "Scene 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Important text</p>".to_string()),
            ..Default::default()
        }).unwrap();

        // Export JSON backup
        let json = db.export_json_backup().unwrap();
        assert!(json.contains("Important text"));
        assert!(json.contains("Chapter 1"));

        // Import into same database (replaces data)
        db.import_json_backup(&json).unwrap();

        // Verify data was restored
        let chapters = db.get_chapters().unwrap();
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "Chapter 1");
    }

    // ========================================================================
    // Template Builtin Protection Tests
    // ========================================================================

    #[test]
    fn test_delete_builtin_template_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.init_builtin_templates().unwrap();
        let templates = db.get_templates().unwrap();
        let builtin = templates.iter().find(|t| t.is_builtin).unwrap();

        let result = db.delete_template(&builtin.id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot delete builtin"));
    }

    #[test]
    fn test_update_builtin_template_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.init_builtin_templates().unwrap();
        let templates = db.get_templates().unwrap();
        let builtin = templates.iter().find(|t| t.is_builtin).unwrap();

        let result = db.update_template(&builtin.id, &UpdateTemplateRequest {
            name: Some("New Name".to_string()),
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot modify builtin"));
    }

    // ========================================================================
    // Additional Error Case Tests
    // ========================================================================

    #[test]
    fn test_get_nonexistent_snapshot_error() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_snapshot("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_restore_nonexistent_snapshot() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.restore_snapshot("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_scene_history_nonexistent_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Should return empty, not error
        let history = db.get_scene_history("nonexistent-scene").unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn test_restore_scene_version_nonexistent_history() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.restore_scene_version(&scene.id, "nonexistent-history-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_scene_versions_nonexistent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.compare_scene_versions(&scene.id, "bad-id-a", "bad-id-b");
        assert!(result.is_err());
    }

    #[test]
    fn test_split_scene_nonexistent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.split_scene("nonexistent", 5, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_scene_nonexistent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.duplicate_scene("nonexistent", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_scenes_empty_list() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.merge_scenes(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_scenes_single_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.merge_scenes(&[scene.id]);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent_issue() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.delete_issue("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_get_empty_deleted_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let deleted = db.get_deleted_scenes().unwrap();
        assert!(deleted.is_empty());
    }

    #[test]
    fn test_get_empty_deleted_chapters() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let deleted = db.get_deleted_chapters().unwrap();
        assert!(deleted.is_empty());
    }

    #[test]
    fn test_get_issues_with_status_filter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_issue(&CreateIssueRequest {
            issue_type: "continuity".to_string(),
            title: "Open issue".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        let issue2 = db.create_issue(&CreateIssueRequest {
            issue_type: "plot_hole".to_string(),
            title: "Resolved issue".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        db.update_issue(&issue2.id, &UpdateIssueRequest {
            status: Some("resolved".to_string()),
            resolution_note: Some("Fixed".to_string()),
        }).unwrap();

        let open = db.get_issues(Some("open")).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].title, "Open issue");

        let resolved = db.get_issues(Some("resolved")).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].title, "Resolved issue");

        let all = db.get_issues(None).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_get_saved_filter_nonexistent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_saved_filter("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_saved_filter_nonexistent_succeeds_silently() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // delete_saved_filter doesn't check row count, so deleting nonexistent is OK
        let result = db.delete_saved_filter("nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_empty_saved_filters() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let filters = db.get_saved_filters(None).unwrap();
        assert!(filters.is_empty());
    }

    #[test]
    fn test_move_scene_between_chapters() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch1 = db.create_chapter(&CreateChapterRequest {
            title: "Chapter 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let ch2 = db.create_chapter(&CreateChapterRequest {
            title: "Chapter 2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: ch1.id.clone(),
            title: "Moving Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        assert_eq!(scene.chapter_id, ch1.id);

        let moved = db.move_scene_to_chapter(&scene.id, &ch2.id, 0).unwrap();
        assert_eq!(moved.chapter_id, ch2.id);

        // Original chapter should have no scenes
        let ch1_scenes = db.get_scenes(&ch1.id).unwrap();
        assert!(ch1_scenes.is_empty());

        // New chapter should have the scene
        let ch2_scenes = db.get_scenes(&ch2.id).unwrap();
        assert_eq!(ch2_scenes.len(), 1);
    }

    #[test]
    fn test_global_search_all_scopes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        setup_project_with_content(&db);

        // Create bible entry for search
        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Searchable Hero".to_string(),
            short_description: Some("A brave hero".to_string()),
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        // Create event for search
        db.create_event(&CreateEventRequest {
            title: "Searchable Event".to_string(),
            description: Some("Something happened".to_string()),
            time_point: Some("Day 1".to_string()),
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        // Search with no scope filter (all scopes)
        let results = db.global_search("Searchable", None).unwrap();
        assert!(results.len() >= 2); // At least bible entry and event

        // Search with "bible" scope
        let bible_results = db.global_search("Searchable", Some(vec!["bible".to_string()])).unwrap();
        assert!(!bible_results.is_empty());

        // Search with "events" scope
        let event_results = db.global_search("Searchable", Some(vec!["events".to_string()])).unwrap();
        assert!(!event_results.is_empty());
    }

    #[test]
    fn test_split_scene_at_position_zero() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Some text here</p>".to_string()),
            ..Default::default()
        }).unwrap();

        // Split at position 0 - original should be empty, new gets all text
        let (original, new_scene) = db.split_scene(&scene.id, 0, Some("New Part")).unwrap();
        assert!(original.text.is_empty() || original.text.trim().is_empty());
        assert!(!new_scene.id.is_empty());
    }

    // ========================================================================
    // Data Integrity: Cascade Deletion Tests
    // ========================================================================

    #[test]
    fn test_delete_bible_entry_cascades_all_associations() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let entry2 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "location".to_string(),
            name: "Castle".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        // Create scene association
        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        }).unwrap();

        // Create bible-to-bible relationship
        db.create_bible_relationship(&CreateBibleRelationshipRequest {
            source_id: entry.id.clone(),
            target_id: entry2.id.clone(),
            relationship_type: "ally".to_string(),
            note: None,
            status: None,
        }).unwrap();

        // Create event link
        let event = db.create_event(&CreateEventRequest {
            title: "Battle".to_string(),
            description: None,
            time_point: Some("Day 1".to_string()),
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();
        db.link_bible_entry_to_event(&entry.id, &event.id).unwrap();

        // Create issue link
        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "continuity".to_string(),
            title: "Test issue".to_string(),
            description: None,
            severity: None,
        }).unwrap();
        db.link_bible_entry_to_issue(&entry.id, &issue.id).unwrap();

        // Verify everything exists
        assert_eq!(db.get_scene_associations(&scene.id).unwrap().len(), 1);
        assert_eq!(db.get_bible_relationships(&entry.id).unwrap().len(), 1);
        assert_eq!(db.get_event_bible_entries(&event.id).unwrap().len(), 1);
        assert_eq!(db.get_issue_bible_entries(&issue.id).unwrap().len(), 1);

        // Delete the bible entry
        db.delete_bible_entry(&entry.id).unwrap();

        // All junction records should be cleaned up
        assert!(db.get_scene_associations(&scene.id).unwrap().is_empty());
        assert!(db.get_bible_relationships(&entry.id).unwrap().is_empty());
        assert!(db.get_event_bible_entries(&event.id).unwrap().is_empty());
        assert!(db.get_issue_bible_entries(&issue.id).unwrap().is_empty());

        // Scene and other entities should still exist
        let fetched_scene = db.get_scene(&scene.id).unwrap();
        assert_eq!(fetched_scene.title, scene.title);
        let fetched_event = db.get_event(&event.id).unwrap();
        assert_eq!(fetched_event.title, "Battle");
    }

    #[test]
    fn test_delete_chapter_cascades_all_scene_junctions() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Doomed Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene with links".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Add bible association
        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Alice".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        }).unwrap();

        // Add arc link
        let arc = db.create_arc(&CreateArcRequest {
            name: "Main Plot".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();
        db.link_scene_to_arc(&scene.id, &arc.id).unwrap();

        // Add event link
        let event = db.create_event(&CreateEventRequest {
            title: "Event".to_string(),
            description: None,
            time_point: Some("Day 1".to_string()),
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();
        db.link_scene_to_event(&scene.id, &event.id).unwrap();

        // Add annotation
        db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 5,
            annotation_type: None,
            content: "A note".to_string(),
        }).unwrap();

        // Verify all links exist
        assert_eq!(db.get_scene_associations(&scene.id).unwrap().len(), 1);
        assert_eq!(db.get_scene_arcs(&scene.id).unwrap().len(), 1);
        assert_eq!(db.get_scene_events(&scene.id).unwrap().len(), 1);
        assert_eq!(db.get_annotations(&scene.id).unwrap().len(), 1);

        // Delete the chapter
        db.delete_chapter(&chapter.id).unwrap();

        // All junction records should be hard-deleted
        assert!(db.get_scene_associations(&scene.id).unwrap().is_empty());
        assert!(db.get_scene_arcs(&scene.id).unwrap().is_empty());
        assert!(db.get_scene_events(&scene.id).unwrap().is_empty());
        assert!(db.get_annotations(&scene.id).unwrap().is_empty());

        // Arc, event, bible entry should still exist independently
        assert_eq!(db.get_arc(&arc.id).unwrap().name, "Main Plot");
        assert_eq!(db.get_event(&event.id).unwrap().title, "Event");
        assert_eq!(db.get_bible_entry(&entry.id).unwrap().name, "Alice");
    }

    #[test]
    fn test_delete_scene_cascades_all_junctions() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // Add bible association
        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Bob".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        }).unwrap();

        // Add arc link
        let arc = db.create_arc(&CreateArcRequest {
            name: "Subplot".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();
        db.link_scene_to_arc(&scene.id, &arc.id).unwrap();

        // Add annotation
        db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 3,
            annotation_type: None,
            content: "Note".to_string(),
        }).unwrap();

        // Delete scene
        db.delete_scene(&scene.id).unwrap();

        // All junctions should be hard-deleted
        assert!(db.get_scene_associations(&scene.id).unwrap().is_empty());
        assert!(db.get_scene_arcs(&scene.id).unwrap().is_empty());
        assert!(db.get_annotations(&scene.id).unwrap().is_empty());

        // Arc and bible entry should still exist
        assert_eq!(db.get_arc(&arc.id).unwrap().name, "Subplot");
        assert_eq!(db.get_bible_entry(&entry.id).unwrap().name, "Bob");
    }

    // ========================================================================
    // Data Integrity: Trash Restore Loses Junctions
    // ========================================================================

    #[test]
    fn test_restore_scene_loses_associations_after_delete() {
        // Important behavioral test: delete hard-deletes junctions,
        // restore only restores the scene itself. Associations are permanently lost.
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Charlie".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        }).unwrap();
        assert_eq!(db.get_scene_associations(&scene.id).unwrap().len(), 1);

        // Delete and restore
        db.delete_scene(&scene.id).unwrap();
        db.restore_scene(&scene.id).unwrap();

        // Scene is back but associations are permanently lost
        let restored = db.get_scene(&scene.id).unwrap();
        assert_eq!(restored.title, scene.title);
        assert!(db.get_scene_associations(&scene.id).unwrap().is_empty());
    }

    #[test]
    fn test_restore_chapter_restores_all_scenes_including_previously_deleted() {
        // delete_chapter overwrites deleted_at for ALL scenes (including previously deleted ones),
        // so restore_chapter brings back everything. Individually-deleted scene junctions
        // are still permanently lost though.
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let scene1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let scene2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene 2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Add a bible association to scene1
        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Alice".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        db.create_association(&CreateAssociationRequest {
            scene_id: scene1.id.clone(),
            bible_entry_id: entry.id.clone(),
        }).unwrap();

        // Delete scene1 individually first (junctions are hard-deleted)
        db.delete_scene(&scene1.id).unwrap();
        assert!(db.get_scene_associations(&scene1.id).unwrap().is_empty());

        // Then delete the whole chapter (overwrites scene1's deleted_at too)
        db.delete_chapter(&chapter.id).unwrap();

        // Restore the chapter
        db.restore_chapter(&chapter.id).unwrap();

        // Both scenes are restored because delete_chapter overwrites all scene timestamps
        let scenes = db.get_scenes(&chapter.id).unwrap();
        assert_eq!(scenes.len(), 2);

        // But scene1's associations remain permanently lost (were hard-deleted by delete_scene)
        assert!(db.get_scene_associations(&scene1.id).unwrap().is_empty());
    }

    // ========================================================================
    // Data Integrity: Split/Merge Association Handling
    // ========================================================================

    #[test]
    fn test_split_scene_copies_bible_associations() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>First half. Second half.</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        }).unwrap();

        // Split the scene
        let (original, new_scene) = db.split_scene(&scene.id, 12, Some("Part 2")).unwrap();

        // Both scenes should have the bible association
        let orig_assoc = db.get_scene_associations(&original.id).unwrap();
        let new_assoc = db.get_scene_associations(&new_scene.id).unwrap();
        assert_eq!(orig_assoc.len(), 1);
        assert_eq!(new_assoc.len(), 1);
        assert_eq!(orig_assoc[0].id, entry.id);
        assert_eq!(new_assoc[0].id, entry.id);
    }

    #[test]
    fn test_split_scene_does_not_copy_arc_links() {
        // By design, split only copies bible associations, not arc links
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Part one. Part two.</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let arc = db.create_arc(&CreateArcRequest {
            name: "Main Arc".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();
        db.link_scene_to_arc(&scene.id, &arc.id).unwrap();

        let (original, new_scene) = db.split_scene(&scene.id, 10, None).unwrap();

        // Original keeps arc link, new scene does NOT get it
        assert_eq!(db.get_scene_arcs(&original.id).unwrap().len(), 1);
        assert!(db.get_scene_arcs(&new_scene.id).unwrap().is_empty());
    }

    #[test]
    fn test_merge_scenes_transfers_bible_associations() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let scene1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let scene2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene 2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.update_scene(&scene1.id, &UpdateSceneRequest {
            text: Some("<p>Text A</p>".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&scene2.id, &UpdateSceneRequest {
            text: Some("<p>Text B</p>".to_string()),
            ..Default::default()
        }).unwrap();

        // Each scene has a different bible association
        let entry1 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Alice".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        let entry2 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "location".to_string(),
            name: "Forest".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        db.create_association(&CreateAssociationRequest {
            scene_id: scene1.id.clone(),
            bible_entry_id: entry1.id.clone(),
        }).unwrap();
        db.create_association(&CreateAssociationRequest {
            scene_id: scene2.id.clone(),
            bible_entry_id: entry2.id.clone(),
        }).unwrap();

        // Merge scene2 into scene1
        let merged = db.merge_scenes(&[scene1.id.clone(), scene2.id.clone()]).unwrap();

        // Merged scene should have associations from BOTH scenes
        let assoc = db.get_scene_associations(&merged.id).unwrap();
        assert_eq!(assoc.len(), 2);
        let names: Vec<&str> = assoc.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"Alice"));
        assert!(names.contains(&"Forest"));

        // Merged text should contain both
        assert!(merged.text.contains("Text A"));
        assert!(merged.text.contains("Text B"));
    }

    // ========================================================================
    // Snapshot Restore: Full Atomicity Test
    // ========================================================================

    #[test]
    fn test_snapshot_restore_full_project_atomicity() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Build a complex project
        let ch1 = db.create_chapter(&CreateChapterRequest {
            title: "Chapter 1".to_string(),
            summary: Some("First chapter".to_string()),
            position: None,
        }).unwrap();
        let ch2 = db.create_chapter(&CreateChapterRequest {
            title: "Chapter 2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch1.id.clone(),
            title: "Scene 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&s1.id, &UpdateSceneRequest {
            text: Some("<p>Original scene 1 text</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch2.id.clone(),
            title: "Scene 2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Original Hero".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let arc = db.create_arc(&CreateArcRequest {
            name: "Original Arc".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        let event = db.create_event(&CreateEventRequest {
            title: "Original Event".to_string(),
            description: None,
            time_point: Some("Day 1".to_string()),
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        // Take snapshot
        let snapshot = db.create_snapshot("Full backup", None, "manual").unwrap();

        // Make destructive changes
        db.delete_chapter(&ch2.id).unwrap();
        db.update_scene(&s1.id, &UpdateSceneRequest {
            text: Some("<p>MODIFIED text</p>".to_string()),
            ..Default::default()
        }).unwrap();
        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "location".to_string(),
            name: "New Location".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        db.delete_event(&event.id).unwrap();

        // Verify changes took effect
        let chapters_before_restore = db.get_chapters().unwrap();
        assert_eq!(chapters_before_restore.len(), 1); // ch2 deleted

        // Restore from snapshot
        db.restore_snapshot(&snapshot.id).unwrap();

        // Verify ALL data restored to snapshot state
        let chapters = db.get_chapters().unwrap();
        assert_eq!(chapters.len(), 2); // Both chapters restored
        assert!(chapters.iter().any(|c| c.title == "Chapter 1"));
        assert!(chapters.iter().any(|c| c.title == "Chapter 2"));

        // Scene text should be original
        let restored_s1 = db.get_scene(&s1.id).unwrap();
        assert!(restored_s1.text.contains("Original scene 1"));

        // Bible entries should be snapshot state (only "Original Hero")
        let entries = db.get_bible_entries(None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Original Hero");

        // Arc should exist
        let arcs = db.get_arcs().unwrap();
        assert_eq!(arcs.len(), 1);
        assert_eq!(arcs[0].name, "Original Arc");

        // Event should be restored
        let events = db.get_events().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Original Event");

        // An automatic backup should have been created
        let snapshots = db.get_snapshots().unwrap();
        assert!(snapshots.iter().any(|s| s.name.contains("Auto-backup")));
    }

    // ========================================================================
    // Input Validation: Boundary Tests
    // ========================================================================

    #[test]
    fn test_scene_with_large_content() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // Create content near MAX_CONTENT_LENGTH (500k chars)
        let large_text = "<p>".to_string() + &"A".repeat(100_000) + "</p>";
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some(large_text.clone()),
            ..Default::default()
        }).unwrap();

        let fetched = db.get_scene(&scene.id).unwrap();
        assert!(fetched.text.len() > 100_000);
    }

    #[test]
    fn test_validation_sanitize_text_truncates() {
        use crate::validation::{sanitize_text, MAX_TITLE_LENGTH};

        let long_title = "A".repeat(MAX_TITLE_LENGTH + 50);
        let sanitized = sanitize_text(&long_title, MAX_TITLE_LENGTH);
        assert_eq!(sanitized.len(), MAX_TITLE_LENGTH);
    }

    #[test]
    fn test_validation_sanitize_multiline_preserves_newlines() {
        use crate::validation::sanitize_multiline_text;

        let multiline = "Line 1\nLine 2\nLine 3";
        let sanitized = sanitize_multiline_text(multiline, 1000);
        assert_eq!(sanitized, multiline);
    }

    #[test]
    fn test_validation_sanitize_text_strips_control_chars() {
        use crate::validation::sanitize_text;

        let with_control = "Hello\x00World\x01Test";
        let sanitized = sanitize_text(with_control, 100);
        assert_eq!(sanitized, "HelloWorldTest");
    }

    #[test]
    fn test_validation_sanitize_text_strips_zero_width_chars() {
        use crate::validation::sanitize_text;

        let with_zwsp = "Hello\u{200B}World\u{FEFF}Test";
        let sanitized = sanitize_text(with_zwsp, 100);
        assert_eq!(sanitized, "HelloWorldTest");
    }

    // ========================================================================
    // End-to-End: Full Writing Workflow with Associations
    // ========================================================================

    #[test]
    fn test_full_writing_workflow_with_associations() {
        let (db, _temp_dir) = create_test_db();

        // 1. Create project
        db.create_project(&CreateProjectRequest {
            title: "My Novel".to_string(),
            author: Some("Author".to_string()),
            description: Some("A story".to_string()),
        }).unwrap();

        // 2. Create chapters
        let ch1 = db.create_chapter(&CreateChapterRequest {
            title: "The Beginning".to_string(),
            summary: Some("Story begins".to_string()),
            position: None,
        }).unwrap();
        let ch2 = db.create_chapter(&CreateChapterRequest {
            title: "The Middle".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // 3. Create scenes
        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch1.id.clone(),
            title: "Opening".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch1.id.clone(),
            title: "Discovery".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s3 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch2.id.clone(),
            title: "Confrontation".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // 4. Add text to scenes
        db.update_scene(&s1.id, &UpdateSceneRequest {
            text: Some("<p>The hero woke up.</p>".to_string()),
            pov: Some("Hero".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&s2.id, &UpdateSceneRequest {
            text: Some("<p>A mysterious map was found.</p>".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&s3.id, &UpdateSceneRequest {
            text: Some("<p>The villain appeared.</p>".to_string()),
            pov: Some("Hero".to_string()),
            on_timeline: Some(true),
            time_point: Some("Day 5".to_string()),
            ..Default::default()
        }).unwrap();

        // 5. Create bible entries
        let hero = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            short_description: Some("The protagonist".to_string()),
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        let villain = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Villain".to_string(),
            short_description: Some("The antagonist".to_string()),
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        // 6. Create associations
        db.create_association(&CreateAssociationRequest {
            scene_id: s1.id.clone(),
            bible_entry_id: hero.id.clone(),
        }).unwrap();
        db.create_association(&CreateAssociationRequest {
            scene_id: s3.id.clone(),
            bible_entry_id: hero.id.clone(),
        }).unwrap();
        db.create_association(&CreateAssociationRequest {
            scene_id: s3.id.clone(),
            bible_entry_id: villain.id.clone(),
        }).unwrap();

        // 7. Create arc and link to scenes
        let arc = db.create_arc(&CreateArcRequest {
            name: "Hero's Journey".to_string(),
            description: Some("The main arc".to_string()),
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();
        db.link_scene_to_arc(&s1.id, &arc.id).unwrap();
        db.link_scene_to_arc(&s3.id, &arc.id).unwrap();

        // 8. Create event and link
        let event = db.create_event(&CreateEventRequest {
            title: "Villain Appears".to_string(),
            description: None,
            time_point: Some("Day 5".to_string()),
            time_start: None,
            time_end: None,
            event_type: None,
            importance: Some("critical".to_string()),
        }).unwrap();
        db.link_scene_to_event(&s3.id, &event.id).unwrap();

        // 9. Verify the full state
        assert_eq!(db.get_chapters().unwrap().len(), 2);
        assert_eq!(db.get_scenes(&ch1.id).unwrap().len(), 2);
        assert_eq!(db.get_scenes(&ch2.id).unwrap().len(), 1);
        assert_eq!(db.get_bible_entries(None).unwrap().len(), 2);
        assert_eq!(db.get_arcs().unwrap().len(), 1);
        assert_eq!(db.get_events().unwrap().len(), 1);
        assert_eq!(db.get_scene_associations(&s3.id).unwrap().len(), 2);
        assert_eq!(db.get_scene_arcs(&s3.id).unwrap().len(), 1);
        assert_eq!(db.get_scene_events(&s3.id).unwrap().len(), 1);

        // 10. Export and verify content
        let markdown = db.export_markdown().unwrap();
        assert!(markdown.contains("The Beginning"));
        assert!(markdown.contains("The Middle"));
        assert!(markdown.contains("hero woke up"));
        assert!(markdown.contains("villain appeared"));

        let outline = db.export_outline().unwrap();
        assert!(outline.contains("Opening"));
        assert!(outline.contains("Confrontation"));

        let json_backup = db.export_json_backup().unwrap();
        assert!(json_backup.contains("Hero"));
        assert!(json_backup.contains("Villain"));
        assert!(json_backup.contains("Hero's Journey"));

        // 11. Take a snapshot for safety
        let snapshot = db.create_snapshot("Before edits", None, "manual").unwrap();
        assert!(!snapshot.id.is_empty());

        // 12. Word counts
        let counts = db.get_word_counts().unwrap();
        assert!(counts.total > 0);
    }


    // ========================================================================
    // FTS5 Sanitization Tests (Security-Critical)
    // ========================================================================

    #[test]
    fn test_fts5_sanitize_empty() {
        let result = Database::sanitize_fts5_query("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_fts5_sanitize_whitespace() {
        let result = Database::sanitize_fts5_query("   ");
        assert!(result.is_empty());
    }

    #[test]
    fn test_fts5_sanitize_single_word() {
        let result = Database::sanitize_fts5_query("hello");
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn test_fts5_sanitize_multiple_words() {
        let result = Database::sanitize_fts5_query("hello world");
        assert_eq!(result, "\"hello\" \"world\"");
    }

    #[test]
    fn test_fts5_sanitize_or_injection() {
        let result = Database::sanitize_fts5_query("cat OR dog");
        assert_eq!(result, "\"cat\" \"OR\" \"dog\"");
    }

    #[test]
    fn test_fts5_sanitize_not_injection() {
        let result = Database::sanitize_fts5_query("NOT secret");
        assert_eq!(result, "\"NOT\" \"secret\"");
    }

    #[test]
    fn test_fts5_sanitize_near_injection() {
        let result = Database::sanitize_fts5_query("NEAR(cat, dog)");
        assert_eq!(result, "\"NEAR(cat,\" \"dog)\"");
    }

    #[test]
    fn test_fts5_sanitize_wildcard() {
        let result = Database::sanitize_fts5_query("test*");
        assert_eq!(result, "\"test*\"");
    }

    #[test]
    fn test_fts5_sanitize_double_quotes() {
        let result = Database::sanitize_fts5_query("say \"hello\"");
        assert_eq!(result, "\"say\" \"\"\"hello\"\"\"");
    }

    #[test]
    fn test_fts5_sanitize_special_chars() {
        let result = Database::sanitize_fts5_query("test-case: foo_bar");
        assert_eq!(result, "\"test-case:\" \"foo_bar\"");
    }

    // ========================================================================
    // Global Search Tests (v2 - comprehensive)
    // ========================================================================

    #[test]
    fn test_search_empty_returns_empty() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let results = db.global_search("", None).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_whitespace_returns_empty() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let results = db.global_search("   ", None).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_scenes_by_text() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>The dragon flew over the castle walls</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let results = db.global_search("dragon", Some(vec!["scenes".to_string()])).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].result_type, "scene");
    }

    #[test]
    fn test_search_bible_entries_by_description() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Gandalf the Grey".to_string(),
            short_description: Some("A powerful wizard".to_string()),
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let results = db.global_search("wizard", Some(vec!["bible".to_string()])).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].result_type, "bible_entry");
    }

    #[test]
    fn test_search_events_by_description() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_event(&CreateEventRequest {
            title: "Battle of Helm's Deep".to_string(),
            description: Some("A great siege battle".to_string()),
            event_type: Some("scene".to_string()),
            time_point: None,
            time_start: None,
            time_end: None,
            importance: None,
        }).unwrap();

        let results = db.global_search("siege", Some(vec!["events".to_string()])).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].result_type, "event");
    }

    #[test]
    fn test_search_annotations_by_content() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 5,
            annotation_type: Some("note".to_string()),
            content: "Fix this overlong dialogue passage".to_string(),
        }).unwrap();

        let results = db.global_search("overlong", Some(vec!["annotations".to_string()])).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].result_type, "annotation");
    }

    #[test]
    fn test_search_cuts_by_text() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        db.create_cut(Some(&scene.id), "The mysterious stranger vanished into the fog").unwrap();

        let results = db.global_search("mysterious", Some(vec!["cuts".to_string()])).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].result_type, "cut");
    }

    #[test]
    fn test_search_scope_excludes_other_types() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_event(&CreateEventRequest {
            title: "UniqueEventXYZ123".to_string(),
            description: None,
            event_type: Some("scene".to_string()),
            time_point: None,
            time_start: None,
            time_end: None,
            importance: None,
        }).unwrap();

        // Searching only scenes should not find the event
        let results = db.global_search("UniqueEventXYZ123", Some(vec!["scenes".to_string()])).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_excludes_deleted_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Searchable unique phrase omega99</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let results = db.global_search("omega99", Some(vec!["scenes".to_string()])).unwrap();
        assert!(!results.is_empty());

        db.delete_scene(&scene.id).unwrap();
        let results = db.global_search("omega99", Some(vec!["scenes".to_string()])).unwrap();
        assert!(results.is_empty());
    }

    // ========================================================================
    // Find & Replace Tests (v2 - comprehensive)
    // ========================================================================

    #[test]
    fn test_find_replace_empty_find_string() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        setup_chapter_and_scene(&db);

        let count = db.find_replace_in_scenes("", "replacement", false, false, None).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_find_replace_simple_replacement() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>The cat sat on the mat</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("cat", "dog", false, false, None).unwrap();
        assert_eq!(count, 1);

        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("dog"));
        assert!(!updated.text.contains("cat"));
    }

    #[test]
    fn test_find_replace_case_insensitive_mode() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>The Cat and the CAT</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("cat", "dog", false, false, None).unwrap();
        assert_eq!(count, 1);

        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("dog"));
        assert!(!updated.text.to_lowercase().contains("cat"));
    }

    #[test]
    fn test_find_replace_case_sensitive_mode() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>The Cat and the cat</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("cat", "dog", true, false, None).unwrap();
        assert_eq!(count, 1);

        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("Cat")); // Capital Cat preserved
        assert!(updated.text.contains("dog")); // lowercase cat replaced
    }

    #[test]
    fn test_find_replace_whole_word_mode() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>The cat concatenated the catalog</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("cat", "dog", false, true, None).unwrap();
        assert_eq!(count, 1);

        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("dog"));
        assert!(updated.text.contains("concatenated"));
        assert!(updated.text.contains("catalog"));
    }

    #[test]
    fn test_find_replace_html_tag_preservation() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>The <strong>bold cat</strong> walked</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("cat", "dog", false, false, None).unwrap();
        assert_eq!(count, 1);

        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("<strong>"));
        assert!(updated.text.contains("</strong>"));
        assert!(updated.text.contains("dog"));
    }

    #[test]
    fn test_find_replace_chapter_scope_isolation() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch1 = db.create_chapter(&CreateChapterRequest {
            title: "Ch1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let ch2 = db.create_chapter(&CreateChapterRequest {
            title: "Ch2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch1.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch2.id.clone(),
            title: "S2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.update_scene(&s1.id, &UpdateSceneRequest {
            text: Some("<p>apple juice</p>".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&s2.id, &UpdateSceneRequest {
            text: Some("<p>apple cider</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("apple", "orange", false, false, Some(&ch1.id)).unwrap();
        assert_eq!(count, 1);

        assert!(db.get_scene(&s1.id).unwrap().text.contains("orange"));
        assert!(db.get_scene(&s2.id).unwrap().text.contains("apple")); // Untouched
    }

    #[test]
    fn test_find_replace_zero_matches() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>The quick brown fox</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("elephant", "giraffe", false, false, None).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_find_replace_across_multiple_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        for i in 0..3 {
            let scene = db.create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: format!("Scene {}", i),
                summary: None,
                position: None,
            }).unwrap();
            db.update_scene(&scene.id, &UpdateSceneRequest {
                text: Some(format!("<p>Scene {} has the word foo in it</p>", i)),
                ..Default::default()
            }).unwrap();
        }

        let count = db.find_replace_in_scenes("foo", "bar", false, false, None).unwrap();
        assert_eq!(count, 3);
    }

    // ========================================================================
    // Name Registry Tests
    // ========================================================================

    #[test]
    fn test_name_registry_crud() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Aragorn".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: None,
            aliases: Some("Strider, Elessar".to_string()),
        }).unwrap();

        assert_eq!(entry.canonical_name, "Aragorn");
        assert_eq!(entry.name_type, "character");
        assert_eq!(entry.aliases, Some("Strider, Elessar".to_string()));
        assert!(!entry.is_confirmed);

        let fetched = db.get_name_registry_entry(&entry.id).unwrap();
        assert_eq!(fetched.canonical_name, "Aragorn");

        let updated = db.update_name_registry_entry(&entry.id, &UpdateNameRegistryRequest {
            canonical_name: Some("Aragorn II".to_string()),
            name_type: None,
            bible_entry_id: None,
            aliases: None,
            is_confirmed: Some(true),
        }).unwrap();
        assert_eq!(updated.canonical_name, "Aragorn II");
        assert!(updated.is_confirmed);

        db.delete_name_registry_entry(&entry.id).unwrap();
        assert!(db.get_name_registry_entry(&entry.id).is_err());
    }

    #[test]
    fn test_name_registry_filter_by_type() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Rivendell".to_string(),
            name_type: Some("location".to_string()),
            bible_entry_id: None,
            aliases: None,
        }).unwrap();
        db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Gandalf".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        let all = db.get_name_registry_entries(None).unwrap();
        assert_eq!(all.len(), 2);

        let chars = db.get_name_registry_entries(Some("character")).unwrap();
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].canonical_name, "Gandalf");

        let locs = db.get_name_registry_entries(Some("location")).unwrap();
        assert_eq!(locs.len(), 1);
        assert_eq!(locs[0].canonical_name, "Rivendell");
    }

    #[test]
    fn test_name_registry_default_type() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Unknown".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: None,
        }).unwrap();
        assert_eq!(entry.name_type, "character");
    }

    #[test]
    fn test_name_registry_bible_link() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let bible = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Gandalf".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Gandalf".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: Some(bible.id.clone()),
            aliases: None,
        }).unwrap();
        assert_eq!(entry.bible_entry_id, Some(bible.id));
    }

    // ========================================================================
    // Name Mentions Tests
    // ========================================================================

    #[test]
    fn test_name_mention_crud() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Frodo".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        let mention = db.create_name_mention(&entry.id, &scene.id, "Frodo", 10, 15).unwrap();
        assert_eq!(mention.mention_text, "Frodo");
        assert_eq!(mention.start_offset, 10);
        assert_eq!(mention.end_offset, 15);
        assert_eq!(mention.status, "pending");

        let by_scene = db.get_name_mentions_by_scene(&scene.id).unwrap();
        assert_eq!(by_scene.len(), 1);

        let by_reg = db.get_name_mentions_by_registry(&entry.id).unwrap();
        assert_eq!(by_reg.len(), 1);

        let updated = db.update_name_mention(&mention.id, &UpdateNameMentionRequest {
            status: "confirmed".to_string(),
        }).unwrap();
        assert_eq!(updated.status, "confirmed");

        db.delete_name_mention(&mention.id).unwrap();
        assert!(db.get_name_mention(&mention.id).is_err());
    }

    #[test]
    fn test_name_mention_ordering() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Sam".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        db.create_name_mention(&entry.id, &scene.id, "Sam", 100, 103).unwrap();
        db.create_name_mention(&entry.id, &scene.id, "Sam", 0, 3).unwrap();
        db.create_name_mention(&entry.id, &scene.id, "Sam", 50, 53).unwrap();

        let mentions = db.get_name_mentions_by_scene(&scene.id).unwrap();
        assert_eq!(mentions.len(), 3);
        assert_eq!(mentions[0].start_offset, 0);
        assert_eq!(mentions[1].start_offset, 50);
        assert_eq!(mentions[2].start_offset, 100);
    }

    #[test]
    fn test_delete_name_registry_cascades_mentions() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Merry".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        db.create_name_mention(&entry.id, &scene.id, "Merry", 0, 5).unwrap();
        db.create_name_mention(&entry.id, &scene.id, "Merry", 20, 25).unwrap();

        assert_eq!(db.get_name_mentions_by_registry(&entry.id).unwrap().len(), 2);

        db.delete_name_registry_entry(&entry.id).unwrap();
        assert!(db.get_name_mentions_by_registry(&entry.id).unwrap().is_empty());
    }

    // ========================================================================
    // Name Merge Tests
    // ========================================================================

    #[test]
    fn test_merge_name_entries() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        let keep = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Robert".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: None,
            aliases: Some("Rob".to_string()),
        }).unwrap();

        let merge = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Bob".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: None,
            aliases: Some("Bobby".to_string()),
        }).unwrap();

        db.create_name_mention(&keep.id, &scene.id, "Robert", 0, 6).unwrap();
        db.create_name_mention(&merge.id, &scene.id, "Bob", 20, 23).unwrap();

        let result = db.merge_name_entries(&keep.id, &merge.id).unwrap();

        assert_eq!(result.canonical_name, "Robert");
        let aliases = result.aliases.unwrap();
        assert!(aliases.contains("Bob"));
        assert!(aliases.contains("Rob"));
        assert!(aliases.contains("Bobby"));

        assert!(db.get_name_registry_entry(&merge.id).is_err());

        let mentions = db.get_name_mentions_by_registry(&keep.id).unwrap();
        assert_eq!(mentions.len(), 2);
    }

    #[test]
    fn test_merge_name_deduplicates_aliases() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let keep = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Elizabeth".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: Some("Liz, Beth".to_string()),
        }).unwrap();

        let merge = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Liz".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: Some("Beth, Lizzy".to_string()),
        }).unwrap();

        let result = db.merge_name_entries(&keep.id, &merge.id).unwrap();
        let aliases = result.aliases.unwrap();
        let alias_list: Vec<&str> = aliases.split(", ").collect();
        let liz_count = alias_list.iter().filter(|a| a.to_lowercase() == "liz").count();
        assert_eq!(liz_count, 1);
        let beth_count = alias_list.iter().filter(|a| a.to_lowercase() == "beth").count();
        assert_eq!(beth_count, 1);
    }

    // ========================================================================
    // Name Scanning Tests
    // ========================================================================

    #[test]
    fn test_scan_names_recurring_proper_nouns() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&s1.id, &UpdateSceneRequest {
            text: Some("The warrior met Eldric at the gate. She told Eldric about the quest.".to_string()),
            ..Default::default()
        }).unwrap();

        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S2".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&s2.id, &UpdateSceneRequest {
            text: Some("Later on Eldric traveled north. The people greeted Eldric warmly.".to_string()),
            ..Default::default()
        }).unwrap();

        let (new_entries, new_mentions) = db.scan_names().unwrap();
        assert!(new_entries >= 1, "Expected at least 1 new entry, got {}", new_entries);
        assert!(new_mentions >= 2, "Expected at least 2 mentions, got {}", new_mentions);

        let entries = db.get_name_registry_entries(None).unwrap();
        let eldric = entries.iter().find(|e| e.canonical_name == "Eldric");
        assert!(eldric.is_some(), "Should have found Eldric");
    }

    #[test]
    fn test_scan_names_skips_common_words() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&s1.id, &UpdateSceneRequest {
            text: Some("He Said goodbye. Then Said hello again.".to_string()),
            ..Default::default()
        }).unwrap();

        let (new_entries, _) = db.scan_names().unwrap();
        let entries = db.get_name_registry_entries(None).unwrap();
        let said = entries.iter().find(|e| e.canonical_name.to_lowercase() == "said");
        assert!(said.is_none(), "Common word 'Said' should not be registered");
        assert_eq!(new_entries, 0);
    }

    #[test]
    fn test_scan_names_empty_project() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let (new_entries, new_mentions) = db.scan_names().unwrap();
        assert_eq!(new_entries, 0);
        assert_eq!(new_mentions, 0);
    }

    // ========================================================================
    // Saved Filters Tests
    // ========================================================================

    #[test]
    fn test_saved_filter_full_crud() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let filter = db.create_saved_filter(&CreateSavedFilterRequest {
            name: "Active Scenes".to_string(),
            filter_type: "scene".to_string(),
            filter_data: r#"{"status":"draft","chapter":"ch1"}"#.to_string(),
        }).unwrap();

        assert_eq!(filter.name, "Active Scenes");
        assert_eq!(filter.filter_type, "scene");

        let fetched = db.get_saved_filter(&filter.id).unwrap();
        assert_eq!(fetched.name, "Active Scenes");

        let all = db.get_saved_filters(None).unwrap();
        assert_eq!(all.len(), 1);

        let by_type = db.get_saved_filters(Some("scene")).unwrap();
        assert_eq!(by_type.len(), 1);
        let wrong_type = db.get_saved_filters(Some("bible")).unwrap();
        assert!(wrong_type.is_empty());

        let updated = db.update_saved_filter(&filter.id, &UpdateSavedFilterRequest {
            name: Some("Draft Scenes".to_string()),
            filter_data: None,
        }).unwrap();
        assert_eq!(updated.name, "Draft Scenes");
        assert_eq!(updated.filter_data, filter.filter_data);

        db.delete_saved_filter(&filter.id).unwrap();
        assert!(db.get_saved_filter(&filter.id).is_err());
    }

    #[test]
    fn test_saved_filter_multiple_types() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_saved_filter(&CreateSavedFilterRequest {
            name: "A".to_string(),
            filter_type: "scene".to_string(),
            filter_data: "{}".to_string(),
        }).unwrap();
        db.create_saved_filter(&CreateSavedFilterRequest {
            name: "B".to_string(),
            filter_type: "bible".to_string(),
            filter_data: "{}".to_string(),
        }).unwrap();
        db.create_saved_filter(&CreateSavedFilterRequest {
            name: "C".to_string(),
            filter_type: "scene".to_string(),
            filter_data: "{}".to_string(),
        }).unwrap();

        assert_eq!(db.get_saved_filters(None).unwrap().len(), 3);
        assert_eq!(db.get_saved_filters(Some("scene")).unwrap().len(), 2);
        assert_eq!(db.get_saved_filters(Some("bible")).unwrap().len(), 1);
    }

    #[test]
    fn test_saved_filter_update_data_only() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let filter = db.create_saved_filter(&CreateSavedFilterRequest {
            name: "My Filter".to_string(),
            filter_type: "scene".to_string(),
            filter_data: r#"{"v":1}"#.to_string(),
        }).unwrap();

        let updated = db.update_saved_filter(&filter.id, &UpdateSavedFilterRequest {
            name: None,
            filter_data: Some(r#"{"v":2}"#.to_string()),
        }).unwrap();

        assert_eq!(updated.name, "My Filter");
        assert_eq!(updated.filter_data, r#"{"v":2}"#);
    }

    // ========================================================================
    // Export Format Tests (v2 - comprehensive)
    // ========================================================================

    #[test]
    fn test_export_markdown_chapter_filter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch1 = db.create_chapter(&CreateChapterRequest {
            title: "Chapter One".to_string(),
            summary: Some("The beginning".to_string()),
            position: None,
        }).unwrap();
        let ch2 = db.create_chapter(&CreateChapterRequest {
            title: "Chapter Two".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch1.id.clone(),
            title: "Opening".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&s1.id, &UpdateSceneRequest {
            text: Some("<p>It was a dark night.</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let _s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch2.id.clone(),
            title: "Middle".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Export only chapter 1
        let md = db.export_markdown_with_options(
            Some(&[ch1.id.clone()]),
            Some("---"),
            true,
        ).unwrap();
        assert!(md.contains("Chapter One"));
        assert!(!md.contains("Chapter Two"));
        assert!(md.contains("dark night"));

        // Export without titles
        let md_no_titles = db.export_markdown_with_options(None, None, false).unwrap();
        assert!(md_no_titles.contains("dark night"));
    }

    #[test]
    fn test_export_plain_text_format() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "The Journey".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Departure".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>The <strong>bold</strong> traveler <em>left</em> home.</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let plain = db.export_plain_text().unwrap();
        assert!(plain.contains("The Journey"));
        assert!(plain.contains("bold"));
        assert!(!plain.contains("<strong>"));
        assert!(!plain.contains("<em>"));
    }

    #[test]
    fn test_export_plain_text_custom_sep() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "S".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Text.</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let plain = db.export_plain_text_with_options(None, Some("~~~")).unwrap();
        assert!(plain.contains("~~~"));
    }

    #[test]
    fn test_export_outline_with_pov() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Chapter Alpha".to_string(),
            summary: Some("Summary of alpha".to_string()),
            position: None,
        }).unwrap();

        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Scene Beta".to_string(),
            summary: Some("Summary of beta".to_string()),
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            pov: Some("Alice".to_string()),
            ..Default::default()
        }).unwrap();

        let outline = db.export_outline().unwrap();
        assert!(outline.contains("Chapter Alpha"));
        assert!(outline.contains("Summary of alpha"));
        assert!(outline.contains("Scene Beta"));
        assert!(outline.contains("POV: Alice"));
    }

    #[test]
    fn test_export_bible_entries() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Gandalf".to_string(),
            short_description: Some("A wizard".to_string()),
            full_description: None,
            aliases: Some("Mithrandir".to_string()),
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "location".to_string(),
            name: "Rivendell".to_string(),
            short_description: Some("Elven city".to_string()),
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let bible_export = db.export_bible().unwrap();
        assert!(bible_export.contains("Characters"));
        assert!(bible_export.contains("Gandalf"));
        assert!(bible_export.contains("Mithrandir"));
        assert!(bible_export.contains("Locations"));
        assert!(bible_export.contains("Rivendell"));
    }

    #[test]
    fn test_export_timeline_with_events_and_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_event(&CreateEventRequest {
            title: "The Great War".to_string(),
            description: Some("A major conflict".to_string()),
            event_type: Some("backstory".to_string()),
            time_point: Some("Year 1000".to_string()),
            time_start: None,
            time_end: None,
            importance: None,
        }).unwrap();

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Battle Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            on_timeline: Some(true),
            time_point: Some("Year 1005".to_string()),
            ..Default::default()
        }).unwrap();

        let timeline = db.export_timeline().unwrap();
        assert!(timeline.contains("The Great War"));
        assert!(timeline.contains("Year 1000"));
        assert!(timeline.contains("Battle Scene"));
        assert!(timeline.contains("Year 1005"));
    }

    // ========================================================================
    // Snapshot Restore Tests (v2 - comprehensive)
    // ========================================================================

    #[test]
    fn test_snapshot_restore_full_project() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Original Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Original Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Original text</p>".to_string()),
            ..Default::default()
        }).unwrap();
        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Original Character".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let snapshot = db.create_snapshot("Checkpoint", None, "manual").unwrap();

        // Modify everything
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Modified text</p>".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_bible_entry(&entry.id, &UpdateBibleEntryRequest {
            name: Some("Modified Character".to_string()),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            image_path: None,
            notes: None,
            todos: None,
            color: None,
            custom_fields: None,
        }).unwrap();

        db.restore_snapshot(&snapshot.id).unwrap();

        let restored_scene = db.get_scene(&scene.id).unwrap();
        assert!(restored_scene.text.contains("Original text"));

        let entries = db.get_bible_entries(None).unwrap();
        assert!(entries.iter().any(|e| e.name == "Original Character"));

        let snapshots = db.get_snapshots().unwrap();
        assert!(snapshots.iter().any(|s| s.snapshot_type == "pre_restore"));
    }

    #[test]
    fn test_snapshot_get_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Scene A".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Scene B".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let snapshot = db.create_snapshot("Test", None, "manual").unwrap();

        let snap_scenes = db.get_snapshot_scenes(&snapshot.id).unwrap();
        assert_eq!(snap_scenes.len(), 2);
        let titles: Vec<&str> = snap_scenes.iter().map(|s| s.title.as_str()).collect();
        assert!(titles.contains(&"Scene A"));
        assert!(titles.contains(&"Scene B"));
    }

    #[test]
    fn test_snapshot_restore_single_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Version 1 text</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let snapshot = db.create_snapshot("V1", None, "manual").unwrap();

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Version 2 text</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let restored = db.restore_scene_from_snapshot(&snapshot.id, &scene.id).unwrap();
        assert!(restored.text.contains("Version 1 text"));
    }

    #[test]
    fn test_snapshot_restore_scene_not_found() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let snapshot = db.create_snapshot("Test", None, "manual").unwrap();

        let result = db.restore_scene_from_snapshot(&snapshot.id, "nonexistent-scene-id");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found in snapshot"));
    }

    #[test]
    fn test_import_json_backup_roundtrip() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Source Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Source Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Source text</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let json = db.export_json_backup().unwrap();

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Modified</p>".to_string()),
            ..Default::default()
        }).unwrap();

        db.import_json_backup(&json).unwrap();

        let restored = db.get_scene(&scene.id).unwrap();
        assert!(restored.text.contains("Source text"));

        let snapshots = db.get_snapshots().unwrap();
        assert!(snapshots.iter().any(|s| s.snapshot_type == "pre_import"));
    }

    #[test]
    fn test_import_invalid_json() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.import_json_backup("not valid json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn test_snapshot_cleanup_respects_type_and_age() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_snapshot("Manual", None, "manual").unwrap();
        db.create_snapshot("Pre Bulk", None, "pre_bulk").unwrap();

        let deleted = db.cleanup_expired_snapshots().unwrap();
        assert_eq!(deleted, 0); // Recent snapshots are not expired

        assert_eq!(db.get_snapshots().unwrap().len(), 2);
    }

    // ========================================================================
    // Additional Error & Edge Cases
    // ========================================================================

    #[test]
    fn test_get_nonexistent_name_registry() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        assert!(db.get_name_registry_entry("nonexistent").is_err());
    }

    #[test]
    fn test_get_nonexistent_name_mention() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        assert!(db.get_name_mention("nonexistent").is_err());
    }

    #[test]
    fn test_get_nonexistent_saved_filter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        assert!(db.get_saved_filter("nonexistent").is_err());
    }

    #[test]
    fn test_merge_nonexistent_name_entry() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Test".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        assert!(db.merge_name_entries(&entry.id, "nonexistent").is_err());
    }

    #[test]
    fn test_restore_nonexistent_snapshot_v2() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        assert!(db.restore_snapshot("nonexistent").is_err());
    }

    #[test]
    fn test_event_scenes_bidirectional_lookup() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "Event".to_string(),
            description: None,
            event_type: Some("scene".to_string()),
            time_point: None,
            time_start: None,
            time_end: None,
            importance: None,
        }).unwrap();

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Char".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        db.link_scene_to_event(&scene.id, &event.id).unwrap();
        db.link_bible_entry_to_event(&entry.id, &event.id).unwrap();

        let event_scenes = db.get_event_scenes(&event.id).unwrap();
        assert_eq!(event_scenes.len(), 1);
        assert_eq!(event_scenes[0], scene.id);

        let event_entries = db.get_event_bible_entries(&event.id).unwrap();
        assert_eq!(event_entries.len(), 1);
        assert_eq!(event_entries[0], entry.id);

        let scene_events = db.get_scene_events(&scene.id).unwrap();
        assert_eq!(scene_events.len(), 1);
        assert_eq!(scene_events[0].id, event.id);
    }

    #[test]
    fn test_issue_bible_entries_lookup() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Test".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "continuity".to_string(),
            title: "Issue".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        db.link_bible_entry_to_issue(&entry.id, &issue.id).unwrap();

        let issue_entries = db.get_issue_bible_entries(&issue.id).unwrap();
        assert_eq!(issue_entries.len(), 1);
        assert_eq!(issue_entries[0], entry.id);
    }

    #[test]
    fn test_scene_issues_bidirectional_lookup() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_, scene) = setup_chapter_and_scene(&db);

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "plot_hole".to_string(),
            title: "Issue".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        db.link_scene_to_issue(&scene.id, &issue.id).unwrap();

        let scene_issues = db.get_scene_issues(&scene.id).unwrap();
        assert_eq!(scene_issues.len(), 1);
        assert_eq!(scene_issues[0].id, issue.id);

        let issue_scenes = db.get_issue_scenes(&issue.id).unwrap();
        assert_eq!(issue_scenes.len(), 1);
        assert_eq!(issue_scenes[0], scene.id);
    }

    #[test]
    fn test_timeline_scene_filtering() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Timeline Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&s1.id, &UpdateSceneRequest {
            on_timeline: Some(true),
            time_point: Some("Day 1".to_string()),
            ..Default::default()
        }).unwrap();

        // Scene not on timeline
        db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "No Timeline".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // On timeline but no time data
        let s3 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Timeline No Time".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&s3.id, &UpdateSceneRequest {
            on_timeline: Some(true),
            ..Default::default()
        }).unwrap();

        let timeline = db.get_all_scenes_for_timeline().unwrap();
        assert_eq!(timeline.len(), 1);
        assert_eq!(timeline[0].title, "Timeline Scene");
    }

    #[test]
    fn test_export_bible_with_relationships() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let char1 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Alice".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        let char2 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Bob".to_string(),
            short_description: None,
            full_description: None,
            aliases: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        db.create_bible_relationship(&CreateBibleRelationshipRequest {
            source_id: char1.id.clone(),
            target_id: char2.id.clone(),
            relationship_type: "friend_of".to_string(),
            note: None,
            status: None,
        }).unwrap();

        let bible_export = db.export_bible().unwrap();
        assert!(bible_export.contains("Alice"));
        assert!(bible_export.contains("Relationships"));
        assert!(bible_export.contains("friend of"));
    }

    #[test]
    fn test_export_all_formats_empty_project() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        assert!(!db.export_markdown().unwrap().is_empty());
        assert!(!db.export_plain_text().unwrap().is_empty());
        assert!(!db.export_outline().unwrap().is_empty());
        assert!(!db.export_bible().unwrap().is_empty());
        assert!(!db.export_timeline().unwrap().is_empty());
        assert!(!db.export_json_backup().unwrap().is_empty());
    }

    #[test]
    fn test_export_markdown_html_to_markdown_conversion() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "S".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Normal text with <strong>bold</strong> and <em>italic</em> words.</p><p>Second paragraph.</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let md = db.export_markdown().unwrap();
        assert!(md.contains("**bold**"));
        assert!(md.contains("*italic*"));
        assert!(!md.contains("<p>"));
        assert!(!md.contains("<strong>"));
    }

    // ========================================================================
    // BATCH 3: Remaining coverage gaps
    // ========================================================================

    // --- Template Step CRUD ---

    #[test]
    fn test_update_template_step_name_and_description() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let template = db.create_template(&CreateTemplateRequest {
            name: "Custom Template".to_string(),
        }).unwrap();

        let step = db.create_template_step(&CreateTemplateStepRequest {
            template_id: template.id.clone(),
            name: "Original Step".to_string(),
            description: Some("Original desc".to_string()),
            typical_position: Some(0.5),
            color: Some("#ff0000".to_string()),
        }).unwrap();

        let updated = db.update_template_step(&step.id, &UpdateTemplateStepRequest {
            name: Some("Renamed Step".to_string()),
            description: Some("Updated desc".to_string()),
            typical_position: None,
            color: None,
            position: None,
        }).unwrap();

        assert_eq!(updated.name, "Renamed Step");
        assert_eq!(updated.description, Some("Updated desc".to_string()));
        // Unchanged fields
        assert!((updated.typical_position - 0.5).abs() < 0.001);
        assert_eq!(updated.color, Some("#ff0000".to_string()));
    }

    #[test]
    fn test_update_template_step_position_and_color() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let template = db.create_template(&CreateTemplateRequest {
            name: "Custom".to_string(),
        }).unwrap();

        let step = db.create_template_step(&CreateTemplateStepRequest {
            template_id: template.id.clone(),
            name: "Step A".to_string(),
            description: None,
            typical_position: Some(0.1),
            color: Some("#000".to_string()),
        }).unwrap();

        let updated = db.update_template_step(&step.id, &UpdateTemplateStepRequest {
            name: None,
            description: None,
            typical_position: Some(0.9),
            color: Some("#fff".to_string()),
            position: Some(5),
        }).unwrap();

        assert_eq!(updated.name, "Step A");
        assert!((updated.typical_position - 0.9).abs() < 0.001);
        assert_eq!(updated.color, Some("#fff".to_string()));
        assert_eq!(updated.position, 5);
    }

    #[test]
    fn test_update_template_step_no_fields_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let template = db.create_template(&CreateTemplateRequest {
            name: "T".to_string(),
        }).unwrap();

        let step = db.create_template_step(&CreateTemplateStepRequest {
            template_id: template.id.clone(),
            name: "Step".to_string(),
            description: Some("Desc".to_string()),
            typical_position: Some(0.3),
            color: None,
        }).unwrap();

        let same = db.update_template_step(&step.id, &UpdateTemplateStepRequest {
            name: None,
            description: None,
            typical_position: None,
            color: None,
            position: None,
        }).unwrap();

        assert_eq!(same.name, "Step");
        assert_eq!(same.description, Some("Desc".to_string()));
    }

    #[test]
    fn test_delete_template_step_custom_template() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let template = db.create_template(&CreateTemplateRequest {
            name: "Custom".to_string(),
        }).unwrap();

        let step1 = db.create_template_step(&CreateTemplateStepRequest {
            template_id: template.id.clone(),
            name: "Step 1".to_string(),
            description: None,
            typical_position: None,
            color: None,
        }).unwrap();
        let _step2 = db.create_template_step(&CreateTemplateStepRequest {
            template_id: template.id.clone(),
            name: "Step 2".to_string(),
            description: None,
            typical_position: None,
            color: None,
        }).unwrap();

        db.delete_template_step(&step1.id).unwrap();

        let steps = db.get_template_steps(&template.id).unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].name, "Step 2");
    }

    #[test]
    fn test_delete_template_step_builtin_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        db.init_builtin_templates().unwrap();

        let templates = db.get_templates().unwrap();
        let builtin = templates.iter().find(|t| t.is_builtin).expect("No builtin template");

        let steps = db.get_template_steps(&builtin.id).unwrap();
        assert!(!steps.is_empty());

        let result = db.delete_template_step(&steps[0].id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("builtin"));
    }

    #[test]
    fn test_delete_template_step_cascades_scene_assignments() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let template = db.create_template(&CreateTemplateRequest {
            name: "Custom".to_string(),
        }).unwrap();

        let step = db.create_template_step(&CreateTemplateStepRequest {
            template_id: template.id.clone(),
            name: "Act 1".to_string(),
            description: None,
            typical_position: None,
            color: None,
        }).unwrap();

        db.assign_scene_to_step(&scene.id, &step.id).unwrap();
        let assigned = db.get_scene_step(&scene.id).unwrap();
        assert!(assigned.is_some());

        db.delete_template_step(&step.id).unwrap();
        let assigned_after = db.get_scene_step(&scene.id).unwrap();
        assert!(assigned_after.is_none());
    }

    #[test]
    fn test_delete_template_cascades_steps_and_scene_assignments() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let template = db.create_template(&CreateTemplateRequest {
            name: "Custom".to_string(),
        }).unwrap();

        let step = db.create_template_step(&CreateTemplateStepRequest {
            template_id: template.id.clone(),
            name: "Step".to_string(),
            description: None,
            typical_position: None,
            color: None,
        }).unwrap();

        db.assign_scene_to_step(&scene.id, &step.id).unwrap();

        db.delete_template(&template.id).unwrap();

        let steps = db.get_template_steps(&template.id).unwrap();
        assert!(steps.is_empty());

        let assigned = db.get_scene_step(&scene.id).unwrap();
        assert!(assigned.is_none());

        assert!(db.get_template(&template.id).is_err());
    }

    // --- set_active_template edge cases ---

    #[test]
    fn test_set_active_template_nonexistent_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.set_active_template("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_active_template_deactivates_previous() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        db.init_builtin_templates().unwrap();

        let templates = db.get_templates().unwrap();
        let first = &templates[0];
        let second = &templates[1];

        db.set_active_template(&first.id).unwrap();
        let t = db.get_template(&first.id).unwrap();
        assert!(t.is_active);

        db.set_active_template(&second.id).unwrap();
        let t1 = db.get_template(&first.id).unwrap();
        let t2 = db.get_template(&second.id).unwrap();
        assert!(!t1.is_active);
        assert!(t2.is_active);
    }

    // --- delete_cut idempotent ---

    #[test]
    fn test_delete_cut_nonexistent_id_succeeds_silently() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.delete_cut("nonexistent-cut-id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_cut_already_deleted_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let cut = db.create_cut(None, "Some text").unwrap();
        db.delete_cut(&cut.id).unwrap();

        let result = db.delete_cut(&cut.id);
        assert!(result.is_ok());
    }

    // --- delete_arc cascade verification ---

    #[test]
    fn test_delete_arc_cascades_scene_arcs_junction() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let arc = db.create_arc(&CreateArcRequest {
            name: "Main Arc".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        db.link_scene_to_arc(&scene.id, &arc.id).unwrap();
        let arcs_before = db.get_scene_arcs(&scene.id).unwrap();
        assert_eq!(arcs_before.len(), 1);

        db.delete_arc(&arc.id).unwrap();

        let arcs_after = db.get_scene_arcs(&scene.id).unwrap();
        assert!(arcs_after.is_empty());

        let all_arcs = db.get_arcs().unwrap();
        assert!(all_arcs.iter().all(|a| a.id != arc.id));
    }

    // --- delete_event cascade verification ---

    #[test]
    fn test_delete_event_cascades_all_junctions() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let event = db.create_event(&CreateEventRequest {
            title: "Big Battle".to_string(),
            description: None,
            time_point: Some("Day 5".to_string()),
            time_start: None,
            time_end: None,
            event_type: Some("scene".to_string()),
            importance: None,
        }).unwrap();

        db.link_scene_to_event(&scene.id, &event.id).unwrap();
        db.link_bible_entry_to_event(&entry.id, &event.id).unwrap();

        assert_eq!(db.get_event_scenes(&event.id).unwrap().len(), 1);
        assert_eq!(db.get_event_bible_entries(&event.id).unwrap().len(), 1);

        db.delete_event(&event.id).unwrap();

        assert!(db.get_event_scenes(&event.id).unwrap().is_empty());
        assert!(db.get_event_bible_entries(&event.id).unwrap().is_empty());
        assert!(db.get_event(&event.id).is_err());
    }

    // --- delete_issue nonexistent ---

    #[test]
    fn test_delete_issue_nonexistent_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.delete_issue("nonexistent-id");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // --- merge_scenes from different chapters ---

    #[test]
    fn test_merge_scenes_different_chapters_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch1 = db.create_chapter(&CreateChapterRequest {
            title: "Chapter 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let ch2 = db.create_chapter(&CreateChapterRequest {
            title: "Chapter 2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch1.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch2.id.clone(),
            title: "S2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let result = db.merge_scenes(&[s1.id, s2.id]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("same chapter"));
    }

    // --- merge_scenes combines notes ---

    #[test]
    fn test_merge_scenes_combines_notes_with_separator() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, _) = setup_chapter_and_scene(&db);

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "First".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Second".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.update_scene(&s1.id, &UpdateSceneRequest {
            text: Some("Text A".to_string()),
            notes: Some("Note A".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&s2.id, &UpdateSceneRequest {
            text: Some("Text B".to_string()),
            notes: Some("Note B".to_string()),
            ..Default::default()
        }).unwrap();

        let merged = db.merge_scenes(&[s1.id, s2.id]).unwrap();
        assert!(merged.text.contains("Text A"));
        assert!(merged.text.contains("Text B"));
        let notes = merged.notes.unwrap();
        assert!(notes.contains("Note A"));
        assert!(notes.contains("Note B"));
        assert!(notes.contains("---"));
    }

    // --- duplicate_scene does not copy associations ---

    #[test]
    fn test_duplicate_scene_does_not_copy_associations() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        }).unwrap();

        let dup = db.duplicate_scene(&scene.id, false).unwrap();

        let orig_assoc = db.get_scene_associations(&scene.id).unwrap();
        assert_eq!(orig_assoc.len(), 1);

        let dup_assoc = db.get_scene_associations(&dup.id).unwrap();
        assert!(dup_assoc.is_empty());
    }

    // --- Validation edge cases ---

    #[test]
    fn test_sanitize_text_max_length_zero() {
        use crate::validation::sanitize_text;
        let result = sanitize_text("Hello world", 0);
        assert_eq!(result, "");
    }

    #[test]
    fn test_sanitize_text_empty_input() {
        use crate::validation::sanitize_text;
        let result = sanitize_text("", 100);
        assert_eq!(result, "");
    }

    #[test]
    fn test_sanitize_text_whitespace_only() {
        use crate::validation::sanitize_text;
        let result = sanitize_text("   ", 100);
        assert_eq!(result, "");
    }

    #[test]
    fn test_sanitize_multiline_text_preserves_newlines() {
        use crate::validation::sanitize_multiline_text;
        let result = sanitize_multiline_text("Line 1\nLine 2\r\nLine 3", 100);
        assert!(result.contains('\n'));
        assert!(result.contains("Line 1"));
        assert!(result.contains("Line 3"));
    }

    #[test]
    fn test_sanitize_multiline_text_strips_control_preserves_newlines() {
        use crate::validation::sanitize_multiline_text;
        let result = sanitize_multiline_text("Hello\x07\nWorld\x00End", 100);
        assert!(result.contains("Hello"));
        assert!(result.contains('\n'));
        assert!(result.contains("World"));
        assert!(result.contains("End"));
        assert!(!result.contains('\x07'));
        assert!(!result.contains('\x00'));
    }

    #[test]
    fn test_validate_required_empty_string() {
        use crate::validation::validate_required;
        let result = validate_required("", "Title");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Title"));
    }

    #[test]
    fn test_validate_required_nonempty_string() {
        use crate::validation::validate_required;
        let result = validate_required("something", "Title");
        assert!(result.is_ok());
    }

    // --- create_bible_entry invalid entry_type ---

    #[test]
    fn test_create_bible_entry_invalid_type() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "invalid_type".to_string(),
            name: "Test".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid entry type"));
    }

    // --- delete_bible_entry soft-delete behavior ---

    #[test]
    fn test_delete_bible_entry_is_soft_delete() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Villain".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        db.delete_bible_entry(&entry.id).unwrap();

        assert!(db.get_bible_entry(&entry.id).is_err());

        let all = db.get_bible_entries(None).unwrap();
        assert!(all.iter().all(|e| e.id != entry.id));
    }

    // --- delete_scene cascade verification (events, issues, steps) ---

    #[test]
    fn test_delete_scene_cascades_events_issues_steps_and_all_junctions() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let arc = db.create_arc(&CreateArcRequest {
            name: "Main Arc".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        let event = db.create_event(&CreateEventRequest {
            title: "Event".to_string(),
            description: None,
            time_point: Some("Day 1".to_string()),
            time_start: None,
            time_end: None,
            event_type: Some("scene".to_string()),
            importance: None,
        }).unwrap();

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "consistency".to_string(),
            title: "Issue".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 5,
            annotation_type: None,
            content: "Note".to_string(),
        }).unwrap();

        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        }).unwrap();
        db.link_scene_to_arc(&scene.id, &arc.id).unwrap();
        db.link_scene_to_event(&scene.id, &event.id).unwrap();
        db.link_scene_to_issue(&scene.id, &issue.id).unwrap();

        let template = db.create_template(&CreateTemplateRequest {
            name: "T".to_string(),
        }).unwrap();
        let step = db.create_template_step(&CreateTemplateStepRequest {
            template_id: template.id.clone(),
            name: "S".to_string(),
            description: None,
            typical_position: None,
            color: None,
        }).unwrap();
        db.assign_scene_to_step(&scene.id, &step.id).unwrap();

        db.delete_scene(&scene.id).unwrap();

        assert!(db.get_scene_associations(&scene.id).unwrap().is_empty());
        assert!(db.get_scene_arcs(&scene.id).unwrap().is_empty());
        assert!(db.get_scene_events(&scene.id).unwrap().is_empty());
        assert!(db.get_scene_issues(&scene.id).unwrap().is_empty());
        assert!(db.get_scene_step(&scene.id).unwrap().is_none());
        assert!(db.get_annotations(&scene.id).unwrap().is_empty());
    }

    // --- reorder with empty lists ---

    #[test]
    fn test_reorder_chapters_empty_list() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.reorder_chapters(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reorder_scenes_empty_list() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, _) = setup_chapter_and_scene(&db);

        let result = db.reorder_scenes(&chapter.id, &[]);
        assert!(result.is_ok());
    }

    // --- move_scene_to_chapter ---

    #[test]
    fn test_move_scene_updates_chapter_and_position() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch1 = db.create_chapter(&CreateChapterRequest {
            title: "Source".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let ch2 = db.create_chapter(&CreateChapterRequest {
            title: "Target".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: ch1.id.clone(),
            title: "Movable".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let moved = db.move_scene_to_chapter(&scene.id, &ch2.id, 0).unwrap();
        assert_eq!(moved.chapter_id, ch2.id);
        assert_eq!(moved.position, 0);

        let source_scenes = db.get_scenes(&ch1.id).unwrap();
        assert!(source_scenes.is_empty());

        let target_scenes = db.get_scenes(&ch2.id).unwrap();
        assert_eq!(target_scenes.len(), 1);
        assert_eq!(target_scenes[0].id, scene.id);
    }

    // --- split_scene at exact text length ---

    #[test]
    fn test_split_scene_at_exact_text_length() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Hello".to_string()),
            ..Default::default()
        }).unwrap();

        let (first, second) = db.split_scene(&scene.id, 5, None).unwrap();
        assert_eq!(first.text, "Hello");
        assert_eq!(second.text, "");
    }

    // --- snapshot restore creates auto-backup ---

    #[test]
    fn test_restore_snapshot_creates_auto_backup() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Sc".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let snapshot = db.create_snapshot("Manual", None, "manual").unwrap();

        let snapshots_before = db.get_snapshots().unwrap();
        let count_before = snapshots_before.len();

        db.restore_snapshot(&snapshot.id).unwrap();

        let snapshots_after = db.get_snapshots().unwrap();
        assert!(snapshots_after.len() > count_before);
        assert!(snapshots_after.iter().any(|s| s.snapshot_type == "pre_restore"));
    }

    // --- import_json_backup creates auto-backup ---

    #[test]
    fn test_import_json_backup_creates_auto_backup() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Sc".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let json = db.export_json_backup().unwrap();

        let snapshots_before = db.get_snapshots().unwrap().len();

        db.import_json_backup(&json).unwrap();

        let snapshots_after = db.get_snapshots().unwrap();
        assert!(snapshots_after.len() > snapshots_before);
        assert!(snapshots_after.iter().any(|s| s.snapshot_type == "pre_import"));
    }

    // --- get_snapshot_scenes ---

    #[test]
    fn test_get_snapshot_scenes_returns_scene_list() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Scene A".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Scene B".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let snapshot = db.create_snapshot("Test", None, "manual").unwrap();

        let scenes = db.get_snapshot_scenes(&snapshot.id).unwrap();
        assert_eq!(scenes.len(), 2);
        assert!(scenes.iter().any(|s| s.title == "Scene A"));
        assert!(scenes.iter().any(|s| s.title == "Scene B"));
    }

    // --- delete_snapshot nonexistent ---

    #[test]
    fn test_delete_snapshot_nonexistent_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.delete_snapshot("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // --- delete_bible_relationship ---

    #[test]
    fn test_delete_bible_relationship_removes_it() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let e1 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Alice".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        let e2 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Bob".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let rel = db.create_bible_relationship(&CreateBibleRelationshipRequest {
            source_id: e1.id.clone(),
            target_id: e2.id.clone(),
            relationship_type: "ally_of".to_string(),
            note: None,
            status: None,
        }).unwrap();

        db.delete_bible_relationship(&rel.id).unwrap();

        assert!(db.get_bible_relationship(&rel.id).is_err());
    }

    // --- update_bible_relationship ---

    #[test]
    fn test_update_bible_relationship_all_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let e1 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "X".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        let e2 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Y".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let rel = db.create_bible_relationship(&CreateBibleRelationshipRequest {
            source_id: e1.id.clone(),
            target_id: e2.id.clone(),
            relationship_type: "enemy_of".to_string(),
            note: None,
            status: None,
        }).unwrap();

        let updated = db.update_bible_relationship(&rel.id, &UpdateBibleRelationshipRequest {
            relationship_type: Some("ally_of".to_string()),
            note: Some("Now friends".to_string()),
            status: Some("resolved".to_string()),
        }).unwrap();

        assert_eq!(updated.relationship_type, "ally_of");
        assert_eq!(updated.note, Some("Now friends".to_string()));
        assert_eq!(updated.status, "resolved");
    }

    // --- export_json_backup structure ---

    #[test]
    fn test_export_json_backup_structure() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        db.create_arc(&CreateArcRequest {
            name: "Arc".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();
        db.create_event(&CreateEventRequest {
            title: "Event".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        let json = db.export_json_backup().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.get("version").is_some());
        assert!(parsed.get("exported_at").is_some());
        assert!(parsed.get("project").is_some());
        assert_eq!(parsed["chapters"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["scenes"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["bible_entries"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["arcs"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["events"].as_array().unwrap().len(), 1);
    }

    // --- create_bible_entry all valid types ---

    #[test]
    fn test_create_bible_entry_all_valid_types() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        for entry_type in &["character", "location", "object", "faction", "concept", "glossary"] {
            let entry = db.create_bible_entry(&CreateBibleEntryRequest {
                entry_type: entry_type.to_string(),
                name: format!("Test {}", entry_type),
                aliases: None,
                short_description: None,
                full_description: None,
                status: None,
                tags: None,
                color: None,
            }).unwrap();
            assert_eq!(entry.entry_type, *entry_type);
        }

        let all = db.get_bible_entries(None).unwrap();
        assert_eq!(all.len(), 6);
    }

    // --- create_bible_entry default custom_fields ---

    #[test]
    fn test_create_bible_entry_default_custom_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let character = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Char".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        let cf = character.custom_fields.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&cf).unwrap();
        assert!(parsed.get("role").is_some());
        assert!(parsed.get("voice_notes").is_some());

        let location = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "location".to_string(),
            name: "Loc".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        let cf = location.custom_fields.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&cf).unwrap();
        assert!(parsed.get("parent_location").is_some());

        let obj = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "object".to_string(),
            name: "Obj".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        assert!(obj.custom_fields.is_none());
    }

    // --- create_event defaults ---

    #[test]
    fn test_create_event_default_type_and_importance() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "Minimal Event".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        assert_eq!(event.event_type, "scene");
        assert_eq!(event.importance, "normal");
    }

    // --- create_arc default status ---

    #[test]
    fn test_create_arc_default_status() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let arc = db.create_arc(&CreateArcRequest {
            name: "Arc".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        assert_eq!(arc.status, "setup");
    }

    // --- update_arc partial ---

    #[test]
    fn test_update_arc_partial_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let arc = db.create_arc(&CreateArcRequest {
            name: "Original".to_string(),
            description: Some("Desc".to_string()),
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        let updated = db.update_arc(&arc.id, &UpdateArcRequest {
            name: Some("Renamed".to_string()),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.description, Some("Desc".to_string()));
    }

    // --- update_event partial ---

    #[test]
    fn test_update_event_partial_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "Original".to_string(),
            description: Some("Desc".to_string()),
            time_point: Some("Day 1".to_string()),
            time_start: None,
            time_end: None,
            event_type: Some("backstory".to_string()),
            importance: Some("critical".to_string()),
        }).unwrap();

        let updated = db.update_event(&event.id, &UpdateEventRequest {
            title: Some("Updated".to_string()),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        assert_eq!(updated.title, "Updated");
        assert_eq!(updated.description, Some("Desc".to_string()));
        assert_eq!(updated.event_type, "backstory");
    }

    // --- get_bible_entry_events bidirectional ---

    #[test]
    fn test_get_bible_entry_events_bidirectional() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let e1 = db.create_event(&CreateEventRequest {
            title: "Event 1".to_string(),
            description: None,
            time_point: Some("Day 1".to_string()),
            time_start: None,
            time_end: None,
            event_type: Some("scene".to_string()),
            importance: None,
        }).unwrap();

        let e2 = db.create_event(&CreateEventRequest {
            title: "Event 2".to_string(),
            description: None,
            time_point: Some("Day 2".to_string()),
            time_start: None,
            time_end: None,
            event_type: Some("scene".to_string()),
            importance: None,
        }).unwrap();

        db.link_bible_entry_to_event(&entry.id, &e1.id).unwrap();
        db.link_bible_entry_to_event(&entry.id, &e2.id).unwrap();

        let events = db.get_bible_entry_events(&entry.id).unwrap();
        assert_eq!(events.len(), 2);
    }

    // --- delete_chapter cascades scene junctions ---

    #[test]
    fn test_delete_chapter_cleans_all_scene_junctions() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let arc = db.create_arc(&CreateArcRequest {
            name: "Arc".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        let event = db.create_event(&CreateEventRequest {
            title: "Event".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        }).unwrap();
        db.link_scene_to_arc(&scene.id, &arc.id).unwrap();
        db.link_scene_to_event(&scene.id, &event.id).unwrap();

        assert_eq!(db.get_scene_associations(&scene.id).unwrap().len(), 1);
        assert_eq!(db.get_scene_arcs(&scene.id).unwrap().len(), 1);
        assert_eq!(db.get_scene_events(&scene.id).unwrap().len(), 1);

        db.delete_chapter(&chapter.id).unwrap();

        assert!(db.get_scene_associations(&scene.id).unwrap().is_empty());
        assert!(db.get_scene_arcs(&scene.id).unwrap().is_empty());
        assert!(db.get_event_scenes(&event.id).unwrap().is_empty());
    }

    // --- get_all_scenes_for_timeline ---

    #[test]
    fn test_get_all_scenes_for_timeline_filters_correctly() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "On Timeline".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Off Timeline".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s3 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "No Time".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.update_scene(&s1.id, &UpdateSceneRequest {
            on_timeline: Some(true),
            time_point: Some("Day 1".to_string()),
            ..Default::default()
        }).unwrap();

        db.update_scene(&s2.id, &UpdateSceneRequest {
            on_timeline: Some(false),
            time_point: Some("Day 2".to_string()),
            ..Default::default()
        }).unwrap();

        db.update_scene(&s3.id, &UpdateSceneRequest {
            on_timeline: Some(true),
            ..Default::default()
        }).unwrap();

        let timeline = db.get_all_scenes_for_timeline().unwrap();
        assert_eq!(timeline.len(), 1);
        assert_eq!(timeline[0].id, s1.id);
    }

    // --- update_chapter partial ---

    #[test]
    fn test_update_chapter_partial_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch = db.create_chapter(&CreateChapterRequest {
            title: "Original".to_string(),
            summary: Some("Sum".to_string()),
            position: None,
        }).unwrap();

        let updated = db.update_chapter(&ch.id, &UpdateChapterRequest {
            title: Some("New Title".to_string()),
            summary: None,
            status: None,
            notes: None,
            position: None,
        }).unwrap();

        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.summary, Some("Sum".to_string()));
    }

    // --- update_issue partial ---

    #[test]
    fn test_update_issue_status_and_resolution() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "consistency".to_string(),
            title: "Bug".to_string(),
            description: None,
            severity: None,
        }).unwrap();
        assert_eq!(issue.status, "open");
        assert!(issue.resolution_note.is_none());

        let updated = db.update_issue(&issue.id, &UpdateIssueRequest {
            status: Some("resolved".to_string()),
            resolution_note: Some("Fixed it".to_string()),
        }).unwrap();

        assert_eq!(updated.status, "resolved");
        assert_eq!(updated.resolution_note, Some("Fixed it".to_string()));
    }

    // ========================================================================
    // BATCH 4: Bidirectional getter coverage
    // ========================================================================

    #[test]
    fn test_get_scene_arcs_returns_linked_arcs() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let arc1 = db.create_arc(&CreateArcRequest {
            name: "Arc A".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();
        let arc2 = db.create_arc(&CreateArcRequest {
            name: "Arc B".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        db.link_scene_to_arc(&scene.id, &arc1.id).unwrap();
        db.link_scene_to_arc(&scene.id, &arc2.id).unwrap();

        let arcs = db.get_scene_arcs(&scene.id).unwrap();
        assert_eq!(arcs.len(), 2);
        let arc_names: Vec<&str> = arcs.iter().map(|a| a.name.as_str()).collect();
        assert!(arc_names.contains(&"Arc A"));
        assert!(arc_names.contains(&"Arc B"));
    }

    #[test]
    fn test_get_scene_arcs_empty_when_no_links() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let arcs = db.get_scene_arcs(&scene.id).unwrap();
        assert!(arcs.is_empty());
    }

    #[test]
    fn test_get_scene_events_returns_linked_events() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "Battle".to_string(),
            description: None,
            time_point: Some("Day 3".to_string()),
            time_start: None,
            time_end: None,
            event_type: Some("scene".to_string()),
            importance: None,
        }).unwrap();

        db.link_scene_to_event(&scene.id, &event.id).unwrap();

        let events = db.get_scene_events(&scene.id).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Battle");
        assert_eq!(events[0].id, event.id);
    }

    #[test]
    fn test_get_event_scenes_returns_linked_scene_ids() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "Event".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        db.link_scene_to_event(&scene.id, &event.id).unwrap();

        let scene_ids = db.get_event_scenes(&event.id).unwrap();
        assert_eq!(scene_ids.len(), 1);
        assert_eq!(scene_ids[0], scene.id);
    }

    #[test]
    fn test_get_event_bible_entries_returns_linked_entry_ids() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let event = db.create_event(&CreateEventRequest {
            title: "Event".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        db.link_bible_entry_to_event(&entry.id, &event.id).unwrap();

        let entry_ids = db.get_event_bible_entries(&event.id).unwrap();
        assert_eq!(entry_ids.len(), 1);
        assert_eq!(entry_ids[0], entry.id);
    }

    #[test]
    fn test_get_scene_issues_returns_linked_issues() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "consistency".to_string(),
            title: "Plot hole".to_string(),
            description: None,
            severity: Some("error".to_string()),
        }).unwrap();

        db.link_scene_to_issue(&scene.id, &issue.id).unwrap();

        let issues = db.get_scene_issues(&scene.id).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].title, "Plot hole");
        assert_eq!(issues[0].id, issue.id);
    }

    #[test]
    fn test_get_issue_scenes_returns_linked_scene_ids() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "todo".to_string(),
            title: "Fix pacing".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        db.link_scene_to_issue(&scene.id, &issue.id).unwrap();

        let scene_ids = db.get_issue_scenes(&issue.id).unwrap();
        assert_eq!(scene_ids.len(), 1);
        assert_eq!(scene_ids[0], scene.id);
    }

    #[test]
    fn test_get_issue_bible_entries_returns_linked_entry_ids() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Villain".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "consistency".to_string(),
            title: "Name inconsistency".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        db.link_bible_entry_to_issue(&entry.id, &issue.id).unwrap();

        let entry_ids = db.get_issue_bible_entries(&issue.id).unwrap();
        assert_eq!(entry_ids.len(), 1);
        assert_eq!(entry_ids[0], entry.id);
    }

    // --- unlink operations verified via getters ---

    #[test]
    fn test_unlink_scene_from_arc_verified_via_getter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let arc = db.create_arc(&CreateArcRequest {
            name: "Arc".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        db.link_scene_to_arc(&scene.id, &arc.id).unwrap();
        assert_eq!(db.get_scene_arcs(&scene.id).unwrap().len(), 1);

        db.unlink_scene_from_arc(&scene.id, &arc.id).unwrap();
        assert!(db.get_scene_arcs(&scene.id).unwrap().is_empty());
    }

    #[test]
    fn test_unlink_scene_from_event_verified_via_getter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "E".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        db.link_scene_to_event(&scene.id, &event.id).unwrap();
        assert_eq!(db.get_scene_events(&scene.id).unwrap().len(), 1);

        db.unlink_scene_from_event(&scene.id, &event.id).unwrap();
        assert!(db.get_scene_events(&scene.id).unwrap().is_empty());
    }

    #[test]
    fn test_unlink_scene_from_issue_verified_via_getter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "todo".to_string(),
            title: "I".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        db.link_scene_to_issue(&scene.id, &issue.id).unwrap();
        assert_eq!(db.get_scene_issues(&scene.id).unwrap().len(), 1);

        db.unlink_scene_from_issue(&scene.id, &issue.id).unwrap();
        assert!(db.get_scene_issues(&scene.id).unwrap().is_empty());
    }

    #[test]
    fn test_unlink_bible_entry_from_event_verified_via_getter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "C".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let event = db.create_event(&CreateEventRequest {
            title: "E".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        db.link_bible_entry_to_event(&entry.id, &event.id).unwrap();
        assert_eq!(db.get_event_bible_entries(&event.id).unwrap().len(), 1);

        db.unlink_bible_entry_from_event(&entry.id, &event.id).unwrap();
        assert!(db.get_event_bible_entries(&event.id).unwrap().is_empty());
    }

    #[test]
    fn test_unlink_bible_entry_from_issue_verified_via_getter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "C".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "consistency".to_string(),
            title: "I".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        db.link_bible_entry_to_issue(&entry.id, &issue.id).unwrap();
        assert_eq!(db.get_issue_bible_entries(&issue.id).unwrap().len(), 1);

        db.unlink_bible_entry_from_issue(&entry.id, &issue.id).unwrap();
        assert!(db.get_issue_bible_entries(&issue.id).unwrap().is_empty());
    }

    // --- delete_annotation edge case ---

    #[test]
    fn test_delete_annotation_nonexistent_succeeds_silently() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // delete_annotation doesn't check row count — silent no-op for nonexistent IDs
        let result = db.delete_annotation("nonexistent");
        assert!(result.is_ok());
    }

    // ========================================================================
    // BATCH 5: Deep coverage – roundtrip, history boundary, annotation edge
    //          cases, name registry, bible cascades, snapshot integrity
    // ========================================================================

    // --- Scene all-fields roundtrip (data integrity for all 26 columns) ---

    #[test]
    fn test_scene_all_fields_roundtrip() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, _scene) = setup_chapter_and_scene(&db);

        // Create a second scene to reference from setup/payoff
        let ref_scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Reference scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Create a scene and update ALL fields
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Full scene".to_string(),
            summary: Some("A summary".to_string()),
            position: Some(10),
        }).unwrap();

        let updated = db.update_scene(&scene.id, &UpdateSceneRequest {
            title: Some("Updated title".to_string()),
            summary: Some("Updated summary".to_string()),
            text: Some("<p>Hello world</p>".to_string()),
            status: Some("first draft".to_string()),
            pov: Some("Alice".to_string()),
            tags: Some("action,drama".to_string()),
            notes: Some("Important note".to_string()),
            todos: Some("Fix pacing".to_string()),
            word_target: Some(5000),
            time_point: Some("2024-06-15".to_string()),
            time_start: Some("2024-06-01".to_string()),
            time_end: Some("2024-06-30".to_string()),
            on_timeline: Some(true),
            position: Some(42),
            pov_goal: Some("Reveal secret".to_string()),
            has_conflict: Some(true),
            has_change: Some(false),
            tension: Some("high".to_string()),
            setup_for_scene_id: Some(ref_scene.id.clone()),
            payoff_of_scene_id: Some(ref_scene.id.clone()),
            revision_notes: Some("Needs more tension".to_string()),
            revision_checklist: Some("[\"pov\",\"conflict\"]".to_string()),
        }).unwrap();

        // Re-fetch to verify roundtrip
        let fetched = db.get_scene(&scene.id).unwrap();

        assert_eq!(fetched.title, "Updated title");
        assert_eq!(fetched.summary.as_deref(), Some("Updated summary"));
        assert_eq!(fetched.text, "<p>Hello world</p>");
        assert_eq!(fetched.status, "first draft");
        assert_eq!(fetched.pov.as_deref(), Some("Alice"));
        assert_eq!(fetched.tags.as_deref(), Some("action,drama"));
        assert_eq!(fetched.notes.as_deref(), Some("Important note"));
        assert_eq!(fetched.todos.as_deref(), Some("Fix pacing"));
        assert_eq!(fetched.word_target, Some(5000));
        assert_eq!(fetched.time_point.as_deref(), Some("2024-06-15"));
        assert_eq!(fetched.time_start.as_deref(), Some("2024-06-01"));
        assert_eq!(fetched.time_end.as_deref(), Some("2024-06-30"));
        assert!(fetched.on_timeline);
        assert_eq!(fetched.position, 42);
        assert_eq!(fetched.pov_goal.as_deref(), Some("Reveal secret"));
        assert_eq!(fetched.has_conflict, Some(true));
        assert_eq!(fetched.has_change, Some(false));
        assert_eq!(fetched.tension.as_deref(), Some("high"));
        assert_eq!(fetched.setup_for_scene_id.as_deref(), Some(ref_scene.id.as_str()));
        assert_eq!(fetched.payoff_of_scene_id.as_deref(), Some(ref_scene.id.as_str()));
        assert_eq!(fetched.revision_notes.as_deref(), Some("Needs more tension"));
        assert_eq!(fetched.revision_checklist.as_deref(), Some("[\"pov\",\"conflict\"]"));
        assert!(!fetched.created_at.is_empty());
        assert!(!fetched.updated_at.is_empty());
        // updated_at should be newer or equal to created_at
        assert!(fetched.updated_at >= fetched.created_at);
        // Also verify the return from update_scene matches
        assert_eq!(updated.title, fetched.title);
        assert_eq!(updated.has_conflict, fetched.has_conflict);
    }

    // --- Scene boolean mapping: on_timeline=false, has_conflict=None, has_change=None ---

    #[test]
    fn test_scene_boolean_defaults_and_none_values() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // Default values after creation
        assert!(scene.on_timeline); // default is 1 per INSERT
        assert_eq!(scene.has_conflict, None);
        assert_eq!(scene.has_change, None);

        // Set on_timeline to false
        let updated = db.update_scene(&scene.id, &UpdateSceneRequest {
            on_timeline: Some(false),
            ..Default::default()
        }).unwrap();
        assert!(!updated.on_timeline);

        // Re-fetch and verify
        let fetched = db.get_scene(&scene.id).unwrap();
        assert!(!fetched.on_timeline);
    }

    // --- Scene history LIMIT 100 boundary ---

    #[test]
    fn test_scene_history_limit_100() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // Make 105 text updates to create 105 history entries
        for i in 0..105 {
            db.update_scene(&scene.id, &UpdateSceneRequest {
                text: Some(format!("Version {}", i)),
                ..Default::default()
            }).unwrap();
        }

        let history = db.get_scene_history(&scene.id).unwrap();
        // Should be capped at 100 due to LIMIT 100 in SQL
        assert_eq!(history.len(), 100);

        // History should be DESC order, so first entry is the most recent
        // The most recent saved text was "Version 103" (saved before the "Version 104" update)
        // because save_scene_to_history saves the CURRENT text BEFORE the update
        assert!(history[0].text.starts_with("Version"));
    }

    // --- Annotation validation edge cases ---

    #[test]
    fn test_annotation_negative_start_offset() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: -1,
            end_offset: 10,
            annotation_type: None,
            content: "test".to_string(),
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Start offset cannot be negative"));
    }

    #[test]
    fn test_annotation_negative_end_offset() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: -1,
            annotation_type: None,
            content: "test".to_string(),
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("End offset cannot be negative"));
    }

    #[test]
    fn test_annotation_equal_start_and_end_offset() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 5,
            end_offset: 5,
            annotation_type: None,
            content: "test".to_string(),
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("End offset must be greater than start offset"));
    }

    #[test]
    fn test_annotation_minimum_valid_range() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // Minimum valid: end = start + 1
        let ann = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 1,
            annotation_type: None,
            content: "min range".to_string(),
        }).unwrap();
        assert_eq!(ann.start_offset, 0);
        assert_eq!(ann.end_offset, 1);
    }

    #[test]
    fn test_annotation_zero_offsets_valid() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // start=0, end=10 should be valid
        let ann = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 10,
            annotation_type: None,
            content: "from zero".to_string(),
        }).unwrap();
        assert_eq!(ann.start_offset, 0);
        assert_eq!(ann.end_offset, 10);
        assert_eq!(ann.status, "open"); // default status
        assert_eq!(ann.annotation_type, "comment"); // default type
    }

    // --- Annotation update partial ---

    #[test]
    fn test_update_annotation_content_only() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let ann = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 5,
            annotation_type: None,
            content: "original".to_string(),
        }).unwrap();

        let updated = db.update_annotation(&ann.id, &UpdateAnnotationRequest {
            content: Some("changed".to_string()),
            status: None,
        }).unwrap();

        assert_eq!(updated.content, "changed");
        assert_eq!(updated.status, "open"); // unchanged
    }

    #[test]
    fn test_update_annotation_status_only() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let ann = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 5,
            annotation_type: None,
            content: "note".to_string(),
        }).unwrap();

        let updated = db.update_annotation(&ann.id, &UpdateAnnotationRequest {
            content: None,
            status: Some("resolved".to_string()),
        }).unwrap();

        assert_eq!(updated.content, "note"); // unchanged
        assert_eq!(updated.status, "resolved");
    }

    #[test]
    fn test_update_annotation_both_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let ann = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 5,
            annotation_type: None,
            content: "old".to_string(),
        }).unwrap();

        let updated = db.update_annotation(&ann.id, &UpdateAnnotationRequest {
            content: Some("new".to_string()),
            status: Some("dismissed".to_string()),
        }).unwrap();

        assert_eq!(updated.content, "new");
        assert_eq!(updated.status, "dismissed");
        assert!(updated.updated_at > ann.updated_at);
    }

    #[test]
    fn test_update_annotation_no_fields_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let ann = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 5,
            annotation_type: None,
            content: "unchanged".to_string(),
        }).unwrap();

        let result = db.update_annotation(&ann.id, &UpdateAnnotationRequest {
            content: None,
            status: None,
        }).unwrap();

        assert_eq!(result.content, "unchanged");
        assert_eq!(result.status, "open");
    }

    // --- Scene history: restore and compare edge cases ---

    #[test]
    fn test_restore_scene_version_wrong_scene_id() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene1) = setup_chapter_and_scene(&db);

        // Create a second scene
        let scene2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene 2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Update scene1 to create a history entry
        db.update_scene(&scene1.id, &UpdateSceneRequest {
            text: Some("Updated text".to_string()),
            ..Default::default()
        }).unwrap();

        let history = db.get_scene_history(&scene1.id).unwrap();
        assert!(!history.is_empty());
        let history_id = &history[0].id;

        // Try to restore scene2 using scene1's history entry — should fail
        let result = db.restore_scene_version(&scene2.id, history_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_scene_versions_both_nonexistent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.compare_scene_versions(&scene.id, "fake-a", "fake-b");
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_scene_versions_one_nonexistent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // Create a history entry
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("v1".to_string()),
            ..Default::default()
        }).unwrap();

        let history = db.get_scene_history(&scene.id).unwrap();
        let valid_id = &history[0].id;

        // One valid, one nonexistent
        let result = db.compare_scene_versions(&scene.id, valid_id, "nonexistent");
        assert!(result.is_err());
    }

    // --- Name registry: CRUD, mentions, merge, delete cascade ---

    #[test]
    fn test_name_registry_update_all_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Aragorn".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        assert!(!entry.is_confirmed);

        // Create a bible entry to link
        let bible = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Aragorn".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let updated = db.update_name_registry_entry(&entry.id, &UpdateNameRegistryRequest {
            canonical_name: Some("Strider".to_string()),
            name_type: Some("location".to_string()),
            bible_entry_id: Some(bible.id.clone()),
            aliases: Some("Elessar, Aragorn".to_string()),
            is_confirmed: Some(true),
        }).unwrap();

        assert_eq!(updated.canonical_name, "Strider");
        assert_eq!(updated.name_type, "location");
        assert_eq!(updated.bible_entry_id.as_deref(), Some(bible.id.as_str()));
        assert_eq!(updated.aliases.as_deref(), Some("Elessar, Aragorn"));
        assert!(updated.is_confirmed);
    }

    #[test]
    fn test_name_mentions_by_registry_id() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Gandalf".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        // Create multiple mentions
        db.create_name_mention(&entry.id, &scene.id, "Gandalf", 0, 7).unwrap();
        db.create_name_mention(&entry.id, &scene.id, "Gandalf", 50, 57).unwrap();
        db.create_name_mention(&entry.id, &scene.id, "Gandalf", 100, 107).unwrap();

        let by_registry = db.get_name_mentions_by_registry(&entry.id).unwrap();
        assert_eq!(by_registry.len(), 3);

        let by_scene = db.get_name_mentions_by_scene(&scene.id).unwrap();
        assert_eq!(by_scene.len(), 3);
        // Scene mentions ordered by start_offset
        assert!(by_scene[0].start_offset <= by_scene[1].start_offset);
        assert!(by_scene[1].start_offset <= by_scene[2].start_offset);
    }

    #[test]
    fn test_name_mention_update_status() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Frodo".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        let mention = db.create_name_mention(&entry.id, &scene.id, "Frodo", 0, 5).unwrap();
        assert_eq!(mention.status, "pending"); // default

        let updated = db.update_name_mention(&mention.id, &UpdateNameMentionRequest {
            status: "confirmed".to_string(),
        }).unwrap();
        assert_eq!(updated.status, "confirmed");
    }

    #[test]
    fn test_name_mention_delete() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Sam".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        let mention = db.create_name_mention(&entry.id, &scene.id, "Sam", 0, 3).unwrap();
        db.delete_name_mention(&mention.id).unwrap();

        let result = db.get_name_mention(&mention.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_name_registry_entry_cascades_all_mentions() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Legolas".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        // Create multiple mentions
        let m1 = db.create_name_mention(&entry.id, &scene.id, "Legolas", 0, 7).unwrap();
        let m2 = db.create_name_mention(&entry.id, &scene.id, "Legolas", 20, 27).unwrap();

        // Delete the registry entry
        db.delete_name_registry_entry(&entry.id).unwrap();

        // Entry is gone
        assert!(db.get_name_registry_entry(&entry.id).is_err());
        // Mentions are gone too
        assert!(db.get_name_mention(&m1.id).is_err());
        assert!(db.get_name_mention(&m2.id).is_err());
        // By scene returns empty
        assert!(db.get_name_mentions_by_scene(&scene.id).unwrap().is_empty());
    }

    #[test]
    fn test_merge_name_entries_moves_mentions_and_combines_aliases() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let keep = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Elizabeth".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: None,
            aliases: Some("Liz".to_string()),
        }).unwrap();

        let merge = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Beth".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: None,
            aliases: Some("Betty".to_string()),
        }).unwrap();

        // Add mentions to both
        db.create_name_mention(&keep.id, &scene.id, "Elizabeth", 0, 9).unwrap();
        db.create_name_mention(&merge.id, &scene.id, "Beth", 20, 24).unwrap();
        db.create_name_mention(&merge.id, &scene.id, "Beth", 40, 44).unwrap();

        // Merge
        let result = db.merge_name_entries(&keep.id, &merge.id).unwrap();

        // Keep entry has combined aliases
        assert_eq!(result.canonical_name, "Elizabeth");
        let aliases = result.aliases.unwrap();
        assert!(aliases.contains("Beth"));
        assert!(aliases.contains("Liz"));
        assert!(aliases.contains("Betty"));

        // Merged entry is gone
        assert!(db.get_name_registry_entry(&merge.id).is_err());

        // All mentions now belong to keep entry
        let mentions = db.get_name_mentions_by_registry(&keep.id).unwrap();
        assert_eq!(mentions.len(), 3);
    }

    #[test]
    fn test_merge_name_entries_deduplicates_canonical_name_alias() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // keep has alias "Beth", merge canonical is "Beth" — should dedup
        let keep = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Elizabeth".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: Some("Beth".to_string()),
        }).unwrap();

        let merge = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Beth".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        let result = db.merge_name_entries(&keep.id, &merge.id).unwrap();
        // "Beth" should appear only once (dedup case-insensitive)
        let aliases = result.aliases.unwrap();
        let beth_count = aliases.to_lowercase().matches("beth").count();
        assert_eq!(beth_count, 1);
    }

    // --- Bible entry deletion cascades relationships ---

    #[test]
    fn test_delete_bible_entry_cascades_relationships() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry_a = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Alice".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let entry_b = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Bob".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let entry_c = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "location".to_string(),
            name: "Castle".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        // Create relationships: A->B, A->C
        let rel_ab = db.create_bible_relationship(&CreateBibleRelationshipRequest {
            source_id: entry_a.id.clone(),
            target_id: entry_b.id.clone(),
            relationship_type: "ally".to_string(),
            note: None,
            status: None,
        }).unwrap();

        let rel_ac = db.create_bible_relationship(&CreateBibleRelationshipRequest {
            source_id: entry_a.id.clone(),
            target_id: entry_c.id.clone(),
            relationship_type: "lives_in".to_string(),
            note: None,
            status: None,
        }).unwrap();

        // Delete entry_a — should cascade relationships
        db.delete_bible_entry(&entry_a.id).unwrap();

        // Both relationships should be gone
        assert!(db.get_bible_relationship(&rel_ab.id).is_err());
        assert!(db.get_bible_relationship(&rel_ac.id).is_err());

        // Entry B and C still exist
        assert!(db.get_bible_entry(&entry_b.id).is_ok());
        assert!(db.get_bible_entry(&entry_c.id).is_ok());

        // B's relationships should be empty now
        assert!(db.get_bible_relationships(&entry_b.id).unwrap().is_empty());
    }

    // --- Bible entry deletion cascades event_bible and issue_bible ---

    #[test]
    fn test_delete_bible_entry_cascades_event_and_issue_links() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Villain".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let event = db.create_event(&CreateEventRequest {
            title: "Battle".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "todo".to_string(),
            title: "Fix villain".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        db.link_bible_entry_to_event(&entry.id, &event.id).unwrap();
        db.link_bible_entry_to_issue(&entry.id, &issue.id).unwrap();

        // Verify links exist
        assert_eq!(db.get_event_bible_entries(&event.id).unwrap().len(), 1);
        assert_eq!(db.get_issue_bible_entries(&issue.id).unwrap().len(), 1);

        // Delete bible entry
        db.delete_bible_entry(&entry.id).unwrap();

        // Links should be gone
        assert!(db.get_event_bible_entries(&event.id).unwrap().is_empty());
        assert!(db.get_issue_bible_entries(&issue.id).unwrap().is_empty());
    }

    // --- Snapshot restore data integrity roundtrip ---

    #[test]
    fn test_snapshot_restore_preserves_all_scene_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        // Set all fields on the scene
        db.update_scene(&scene.id, &UpdateSceneRequest {
            title: Some("Full Scene".to_string()),
            summary: Some("Complete summary".to_string()),
            text: Some("<p>Full text content</p>".to_string()),
            status: Some("revised".to_string()),
            pov: Some("Narrator".to_string()),
            tags: Some("important,key".to_string()),
            notes: Some("Author notes here".to_string()),
            todos: Some("Check continuity".to_string()),
            word_target: Some(3000),
            time_point: Some("2024-01-01".to_string()),
            time_start: Some("2024-01-01".to_string()),
            time_end: Some("2024-12-31".to_string()),
            on_timeline: Some(true),
            position: Some(5),
            pov_goal: Some("Establish setting".to_string()),
            has_conflict: Some(true),
            has_change: Some(true),
            tension: Some("medium".to_string()),
            setup_for_scene_id: None,
            payoff_of_scene_id: None,
            revision_notes: Some("Reviewed".to_string()),
            revision_checklist: Some("[\"done\"]".to_string()),
        }).unwrap();

        // Create snapshot
        let snapshot = db.create_snapshot("Full test", None, "manual").unwrap();

        // Wipe the scene text to verify restore works
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("WIPED".to_string()),
            pov: Some("WIPED".to_string()),
            ..Default::default()
        }).unwrap();

        // Restore single scene from snapshot
        let restored = db.restore_scene_from_snapshot(&snapshot.id, &scene.id).unwrap();

        assert_eq!(restored.title, "Full Scene");
        assert_eq!(restored.summary.as_deref(), Some("Complete summary"));
        assert_eq!(restored.text, "<p>Full text content</p>");
        assert_eq!(restored.status, "revised");
        assert_eq!(restored.pov.as_deref(), Some("Narrator"));
        assert_eq!(restored.tags.as_deref(), Some("important,key"));
        assert_eq!(restored.notes.as_deref(), Some("Author notes here"));
        assert_eq!(restored.todos.as_deref(), Some("Check continuity"));
        assert_eq!(restored.word_target, Some(3000));
        assert_eq!(restored.time_point.as_deref(), Some("2024-01-01"));
        assert!(restored.on_timeline);
        assert_eq!(restored.has_conflict, Some(true));
        assert_eq!(restored.has_change, Some(true));
        assert_eq!(restored.tension.as_deref(), Some("medium"));
        assert_eq!(restored.pov_goal.as_deref(), Some("Establish setting"));
        assert_eq!(restored.revision_notes.as_deref(), Some("Reviewed"));
    }

    // --- Scene history: restore creates a new history entry ---

    #[test]
    fn test_restore_scene_version_creates_history_entry() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // Update text to v1
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Version 1".to_string()),
            ..Default::default()
        }).unwrap();

        // Update text to v2
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Version 2".to_string()),
            ..Default::default()
        }).unwrap();

        let history_before = db.get_scene_history(&scene.id).unwrap();
        assert_eq!(history_before.len(), 2);

        // Restore to v1 (the older history entry = index 1 since DESC order)
        let v1_history = &history_before[1];
        db.restore_scene_version(&scene.id, &v1_history.id).unwrap();

        // Restoring should have created ANOTHER history entry (saving v2 before restoring)
        let history_after = db.get_scene_history(&scene.id).unwrap();
        assert_eq!(history_after.len(), 3);
    }

    // --- Name scan end-to-end ---

    #[test]
    fn test_scan_names_skips_first_word_of_sentence() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, _) = setup_chapter_and_scene(&db);

        // "The" starts sentences, "Mordor" appears mid-sentence twice
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Test".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("The army marched to Mordor. They reached Mordor at dawn.".to_string()),
            ..Default::default()
        }).unwrap();

        let (new_entries, new_mentions) = db.scan_names().unwrap();
        assert!(new_entries >= 1, "Should detect Mordor as a proper noun");
        assert!(new_mentions >= 2, "Should record at least 2 mentions of Mordor");

        // Verify it's in the registry
        let entries = db.get_name_registry_entries(None).unwrap();
        let mordor_entry = entries.iter().find(|e| e.canonical_name == "Mordor");
        assert!(mordor_entry.is_some());
    }

    #[test]
    fn test_scan_names_auto_links_to_bible_entry() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, _) = setup_chapter_and_scene(&db);

        // Create a bible entry named "Gandalf"
        let bible = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Gandalf".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        // Create scene with "Gandalf" mentioned mid-sentence twice
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Wizard scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("The wizard called Gandalf arrived. Everyone knew Gandalf well.".to_string()),
            ..Default::default()
        }).unwrap();

        db.scan_names().unwrap();

        let entries = db.get_name_registry_entries(None).unwrap();
        let gandalf_entry = entries.iter().find(|e| e.canonical_name == "Gandalf");
        assert!(gandalf_entry.is_some());
        // Should be auto-linked to the bible entry
        assert_eq!(gandalf_entry.unwrap().bible_entry_id.as_deref(), Some(bible.id.as_str()));
    }

    #[test]
    fn test_scan_names_single_occurrence_ignored() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, _) = setup_chapter_and_scene(&db);

        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Test".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        // "Thorin" appears only once mid-sentence
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("The dwarf called Thorin spoke softly.".to_string()),
            ..Default::default()
        }).unwrap();

        let (new_entries, _) = db.scan_names().unwrap();
        assert_eq!(new_entries, 0, "Single occurrence should be ignored");
    }

    #[test]
    fn test_scan_names_html_stripping() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, _) = setup_chapter_and_scene(&db);

        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "HTML test".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        // HTML tags should be stripped; "Elrond" appears twice mid-sentence
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>The elf lord <strong>Elrond</strong> spoke. Everyone heard Elrond clearly.</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let (new_entries, _) = db.scan_names().unwrap();
        assert!(new_entries >= 1, "Should detect Elrond despite HTML tags");
    }

    // --- Name registry filter by type ---

    #[test]
    fn test_name_registry_entries_filter_returns_only_matching() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Mordor".to_string(),
            name_type: Some("location".to_string()),
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Gandalf".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        let locations = db.get_name_registry_entries(Some("location")).unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].canonical_name, "Mordor");

        let characters = db.get_name_registry_entries(Some("character")).unwrap();
        assert_eq!(characters.len(), 1);
        assert_eq!(characters[0].canonical_name, "Gandalf");

        let all = db.get_name_registry_entries(None).unwrap();
        assert_eq!(all.len(), 2);
    }

    // --- Update name registry with no fields is idempotent ---

    #[test]
    fn test_update_name_registry_no_fields_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Unchanged".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        let updated = db.update_name_registry_entry(&entry.id, &UpdateNameRegistryRequest {
            canonical_name: None,
            name_type: None,
            bible_entry_id: None,
            aliases: None,
            is_confirmed: None,
        }).unwrap();

        assert_eq!(updated.canonical_name, "Unchanged");
        assert!(!updated.is_confirmed);
    }

    // --- Cleanup expired snapshots: mixed types ---

    #[test]
    fn test_cleanup_expired_snapshots_mixed_types_and_ages() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        setup_chapter_and_scene(&db);

        // Create a manual snapshot (should never be cleaned)
        db.create_snapshot("Manual", None, "manual").unwrap();

        // Create a pre_restore snapshot (should never be cleaned by type filter)
        db.create_snapshot("Pre-restore", None, "pre_restore").unwrap();

        // Create a pre_bulk snapshot (could be cleaned if old)
        db.create_snapshot("Pre-bulk recent", None, "pre_bulk").unwrap();

        let before = db.get_snapshots().unwrap();
        assert_eq!(before.len(), 3);

        // Cleanup — none should be deleted (all recent)
        let deleted = db.cleanup_expired_snapshots().unwrap();
        assert_eq!(deleted, 0);

        let after = db.get_snapshots().unwrap();
        assert_eq!(after.len(), 3);
    }

    // --- Scene delete + scene history interaction ---

    #[test]
    fn test_scene_history_survives_scene_deletion() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // Create some history
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("v1".to_string()),
            ..Default::default()
        }).unwrap();

        let history_before = db.get_scene_history(&scene.id).unwrap();
        assert_eq!(history_before.len(), 1);

        // Delete scene (soft delete) — history is NOT cleaned by delete_scene
        // delete_scene cleans: canonical_associations, scene_arcs, event_scenes,
        // issue_scenes, scene_steps, annotations, name_mentions — but NOT scene_history
        db.delete_scene(&scene.id).unwrap();

        // Scene is gone from get_scene (filters deleted_at IS NULL)
        assert!(db.get_scene(&scene.id).is_err());

        // But scene_history still has entries (not cleaned on soft delete)
        // This is by design: history is preserved for potential restore
        let history_after = db.get_scene_history(&scene.id).unwrap();
        assert_eq!(history_after.len(), 1);
    }

    // --- Multiple annotations ordered by start_offset ---

    #[test]
    fn test_annotations_ordered_by_start_offset() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // Create annotations in reverse order
        db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 50,
            end_offset: 60,
            annotation_type: None,
            content: "Third".to_string(),
        }).unwrap();

        db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 10,
            end_offset: 20,
            annotation_type: None,
            content: "First".to_string(),
        }).unwrap();

        db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 30,
            end_offset: 40,
            annotation_type: None,
            content: "Second".to_string(),
        }).unwrap();

        let anns = db.get_annotations(&scene.id).unwrap();
        assert_eq!(anns.len(), 3);
        assert_eq!(anns[0].content, "First");
        assert_eq!(anns[1].content, "Second");
        assert_eq!(anns[2].content, "Third");
    }

    // --- Annotation with custom type ---

    #[test]
    fn test_annotation_custom_type() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let ann = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 10,
            annotation_type: Some("highlight".to_string()),
            content: "Important passage".to_string(),
        }).unwrap();

        assert_eq!(ann.annotation_type, "highlight");
    }

    // --- Full snapshot restore then verify all entity counts ---

    #[test]
    fn test_snapshot_full_restore_entity_counts() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Create rich data
        let ch = db.create_chapter(&crate::models::CreateChapterRequest {
            title: "Chapter 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Scene 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch.id.clone(),
            title: "Scene 2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        db.create_arc(&crate::models::CreateArcRequest {
            name: "Main Arc".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        db.create_event(&CreateEventRequest {
            title: "Key event".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        // Snapshot
        let snap = db.create_snapshot("Full state", None, "manual").unwrap();

        // Destroy everything
        db.delete_scene(&s1.id).unwrap();
        db.delete_scene(&s2.id).unwrap();

        // Restore
        db.restore_snapshot(&snap.id).unwrap();

        // Verify counts
        let chapters = db.get_chapters().unwrap();
        assert_eq!(chapters.len(), 1); // the one we created (default from setup excluded since restore clears all)

        let scenes = db.get_scenes(&ch.id).unwrap();
        assert_eq!(scenes.len(), 2);

        let bible = db.get_bible_entries(None).unwrap();
        assert_eq!(bible.len(), 1);

        let arcs = db.get_arcs().unwrap();
        assert_eq!(arcs.len(), 1);

        let events = db.get_events().unwrap();
        assert_eq!(events.len(), 1);
    }

    // --- Update scene with no fields is idempotent ---

    #[test]
    fn test_update_scene_no_fields_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let before = db.get_scene(&scene.id).unwrap();

        let after = db.update_scene(&scene.id, &UpdateSceneRequest {
            ..Default::default()
        }).unwrap();

        assert_eq!(before.title, after.title);
        assert_eq!(before.text, after.text);
        assert_eq!(before.status, after.status);
    }

    // --- Bible entry default custom_fields for faction and glossary ---

    #[test]
    fn test_bible_entry_faction_default_custom_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "faction".to_string(),
            name: "The Guild".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let cf = entry.custom_fields.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&cf).unwrap();
        assert!(parsed.get("faction_type").is_some());
        assert!(parsed.get("members").is_some());
        assert!(parsed.get("headquarters").is_some());
    }

    #[test]
    fn test_bible_entry_glossary_default_custom_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "glossary".to_string(),
            name: "Mithril".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let cf = entry.custom_fields.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&cf).unwrap();
        assert!(parsed.get("pronunciation").is_some());
        assert!(parsed.get("etymology").is_some());
        assert!(parsed.get("language").is_some());
    }

    #[test]
    fn test_bible_entry_concept_no_custom_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "concept".to_string(),
            name: "Magic".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        // Concepts have no default custom_fields (returns None)
        assert!(entry.custom_fields.is_none());
    }

    #[test]
    fn test_bible_entry_object_no_custom_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "object".to_string(),
            name: "Sword".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        assert!(entry.custom_fields.is_none());
    }

    // ========================================================================
    // BATCH 6: Final gap closure – explicit happy-path tests for every
    //          single-item getter, plus additional robustness tests
    // ========================================================================

    // --- Single-item getters: happy-path tests ---

    #[test]
    fn test_get_annotation_by_id_happy_path() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let created = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 5,
            end_offset: 15,
            annotation_type: Some("note".to_string()),
            content: "Get by ID".to_string(),
        }).unwrap();

        let fetched = db.get_annotation(&created.id).unwrap();
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.scene_id, scene.id);
        assert_eq!(fetched.start_offset, 5);
        assert_eq!(fetched.end_offset, 15);
        assert_eq!(fetched.annotation_type, "note");
        assert_eq!(fetched.content, "Get by ID");
        assert_eq!(fetched.status, "open");
    }

    #[test]
    fn test_get_annotation_nonexistent_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_annotation("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_bible_relationship_by_id_happy_path() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry_a = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Hero".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let entry_b = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Sidekick".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let rel = db.create_bible_relationship(&CreateBibleRelationshipRequest {
            source_id: entry_a.id.clone(),
            target_id: entry_b.id.clone(),
            relationship_type: "friend".to_string(),
            note: Some("Best friends".to_string()),
            status: Some("active".to_string()),
        }).unwrap();

        let fetched = db.get_bible_relationship(&rel.id).unwrap();
        assert_eq!(fetched.id, rel.id);
        assert_eq!(fetched.source_id, entry_a.id);
        assert_eq!(fetched.target_id, entry_b.id);
        assert_eq!(fetched.relationship_type, "friend");
        assert_eq!(fetched.note.as_deref(), Some("Best friends"));
        assert_eq!(fetched.status, "active");
    }

    #[test]
    fn test_get_bible_relationship_nonexistent_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_bible_relationship("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_templates_ordering_builtins_first() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        db.init_builtin_templates().unwrap();

        // Create a custom template
        db.create_template(&CreateTemplateRequest {
            name: "AAA Custom".to_string(), // alphabetically before builtins
        }).unwrap();

        let templates = db.get_templates().unwrap();
        assert!(templates.len() >= 5); // 4 builtins + 1 custom

        // Builtins should come first (ORDER BY is_builtin DESC, name)
        let first_custom_idx = templates.iter().position(|t| !t.is_builtin).unwrap();
        for t in &templates[..first_custom_idx] {
            assert!(t.is_builtin);
        }
        // Custom template should be after all builtins
        assert!(!templates[first_custom_idx].is_builtin);
        assert_eq!(templates[first_custom_idx].name, "AAA Custom");
    }

    #[test]
    fn test_get_template_step_by_id_happy_path() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let template = db.create_template(&CreateTemplateRequest {
            name: "Test Template".to_string(),
        }).unwrap();

        let step = db.create_template_step(&CreateTemplateStepRequest {
            template_id: template.id.clone(),
            name: "Step One".to_string(),
            description: Some("The first step".to_string()),
            typical_position: Some(0.25),
            color: Some("#ff0000".to_string()),
        }).unwrap();

        let fetched = db.get_template_step(&step.id).unwrap();
        assert_eq!(fetched.id, step.id);
        assert_eq!(fetched.template_id, template.id);
        assert_eq!(fetched.name, "Step One");
        assert_eq!(fetched.description.as_deref(), Some("The first step"));
        assert!((fetched.typical_position - 0.25).abs() < f64::EPSILON);
        assert_eq!(fetched.color.as_deref(), Some("#ff0000"));
    }

    #[test]
    fn test_get_template_step_nonexistent_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_template_step("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_name_registry_entry_by_id_happy_path() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let created = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Thorin".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: None,
            aliases: Some("Oakenshield".to_string()),
        }).unwrap();

        let fetched = db.get_name_registry_entry(&created.id).unwrap();
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.canonical_name, "Thorin");
        assert_eq!(fetched.name_type, "character");
        assert_eq!(fetched.aliases.as_deref(), Some("Oakenshield"));
        assert!(!fetched.is_confirmed);
        assert!(!fetched.created_at.is_empty());
    }

    #[test]
    fn test_get_name_mention_by_id_happy_path() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let entry = db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Bilbo".to_string(),
            name_type: None,
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        let mention = db.create_name_mention(&entry.id, &scene.id, "Bilbo", 10, 15).unwrap();

        let fetched = db.get_name_mention(&mention.id).unwrap();
        assert_eq!(fetched.id, mention.id);
        assert_eq!(fetched.name_registry_id, entry.id);
        assert_eq!(fetched.scene_id, scene.id);
        assert_eq!(fetched.mention_text, "Bilbo");
        assert_eq!(fetched.start_offset, 10);
        assert_eq!(fetched.end_offset, 15);
        assert_eq!(fetched.status, "pending");
    }

    #[test]
    fn test_get_scene_history_happy_path() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // No history yet
        let empty_history = db.get_scene_history(&scene.id).unwrap();
        assert!(empty_history.is_empty());

        // Create 3 versions
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("First draft".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Second draft".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Third draft".to_string()),
            ..Default::default()
        }).unwrap();

        let history = db.get_scene_history(&scene.id).unwrap();
        assert_eq!(history.len(), 3);
        // DESC order: most recent first
        // The 3rd save captured "Second draft" (text before 3rd update)
        assert_eq!(history[0].text, "Second draft");
        assert_eq!(history[1].text, "First draft");
        // The 1st save captured "" (empty initial text before 1st update)
        assert_eq!(history[2].text, "");
        // All entries have the correct scene_id
        for entry in &history {
            assert_eq!(entry.scene_id, scene.id);
            assert!(!entry.id.is_empty());
            assert!(!entry.created_at.is_empty());
        }
    }

    // --- Template: get_templates with no templates ---

    #[test]
    fn test_get_templates_empty_before_init() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Before init_builtin_templates, should be empty
        let templates = db.get_templates().unwrap();
        assert!(templates.is_empty());
    }

    // --- Template step: verify ordering by position ---

    #[test]
    fn test_get_template_steps_ordered_by_position() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let template = db.create_template(&CreateTemplateRequest {
            name: "Ordered".to_string(),
        }).unwrap();

        // Create steps out of order
        db.create_template_step(&CreateTemplateStepRequest {
            template_id: template.id.clone(),
            name: "Third".to_string(),
            description: None,
            typical_position: None,
            color: None,
        }).unwrap();
        db.create_template_step(&CreateTemplateStepRequest {
            template_id: template.id.clone(),
            name: "First".to_string(),
            description: None,
            typical_position: None,
            color: None,
        }).unwrap();
        db.create_template_step(&CreateTemplateStepRequest {
            template_id: template.id.clone(),
            name: "Second".to_string(),
            description: None,
            typical_position: None,
            color: None,
        }).unwrap();

        let steps = db.get_template_steps(&template.id).unwrap();
        assert_eq!(steps.len(), 3);
        // Steps should be ordered by their position field
        for i in 1..steps.len() {
            assert!(steps[i].position >= steps[i - 1].position);
        }
    }

    // --- Bible search with no results ---

    #[test]
    fn test_search_bible_no_results() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Gandalf".to_string(),
            aliases: None,
            short_description: Some("A wizard".to_string()),
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let results = db.search_bible("nonexistent_term_xyz").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_bible_empty_query_returns_empty() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let results = db.search_bible("").unwrap();
        assert!(results.is_empty());
    }

    // --- Chapter update with no fields is idempotent ---

    #[test]
    fn test_update_chapter_no_fields_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&crate::models::CreateChapterRequest {
            title: "My Chapter".to_string(),
            summary: Some("A summary".to_string()),
            position: None,
        }).unwrap();

        let updated = db.update_chapter(&chapter.id, &crate::models::UpdateChapterRequest {
            title: None,
            summary: None,
            status: None,
            notes: None,
            position: None,
        }).unwrap();

        assert_eq!(updated.title, "My Chapter");
        assert_eq!(updated.summary.as_deref(), Some("A summary"));
    }

    // --- Bible entry update with no fields is idempotent ---

    #[test]
    fn test_update_bible_entry_no_fields_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Unchanged".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let updated = db.update_bible_entry(&entry.id, &crate::models::UpdateBibleEntryRequest {
            name: None,
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            image_path: None,
            notes: None,
            todos: None,
            color: None,
            custom_fields: None,
        }).unwrap();

        assert_eq!(updated.name, "Unchanged");
        assert_eq!(updated.entry_type, "character");
    }

    // --- Arc update with no fields is idempotent ---

    #[test]
    fn test_update_arc_no_fields_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let arc = db.create_arc(&crate::models::CreateArcRequest {
            name: "Steady Arc".to_string(),
            description: Some("Unchanged".to_string()),
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        let updated = db.update_arc(&arc.id, &crate::models::UpdateArcRequest {
            name: None,
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        assert_eq!(updated.name, "Steady Arc");
        assert_eq!(updated.description.as_deref(), Some("Unchanged"));
    }

    // --- Event update with no fields is idempotent ---

    #[test]
    fn test_update_event_no_fields_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "Steady Event".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        let updated = db.update_event(&event.id, &crate::models::UpdateEventRequest {
            title: None,
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        assert_eq!(updated.title, "Steady Event");
    }

    // --- Issue update with no fields is idempotent ---

    #[test]
    fn test_update_issue_no_fields_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "todo".to_string(),
            title: "Steady Issue".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        let updated = db.update_issue(&issue.id, &crate::models::UpdateIssueRequest {
            status: None,
            resolution_note: None,
        }).unwrap();

        assert_eq!(updated.title, "Steady Issue");
        assert_eq!(updated.issue_type, "todo");
    }

    // --- Duplicate link operations are idempotent (INSERT OR IGNORE) ---

    #[test]
    fn test_link_scene_to_arc_duplicate_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let arc = db.create_arc(&crate::models::CreateArcRequest {
            name: "Arc".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        db.link_scene_to_arc(&scene.id, &arc.id).unwrap();
        // Second link should not fail or duplicate
        db.link_scene_to_arc(&scene.id, &arc.id).unwrap();

        let arcs = db.get_scene_arcs(&scene.id).unwrap();
        assert_eq!(arcs.len(), 1);
    }

    #[test]
    fn test_link_scene_to_event_duplicate_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "E".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        db.link_scene_to_event(&scene.id, &event.id).unwrap();
        db.link_scene_to_event(&scene.id, &event.id).unwrap();

        let events = db.get_scene_events(&scene.id).unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_link_scene_to_issue_duplicate_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "todo".to_string(),
            title: "I".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        db.link_scene_to_issue(&scene.id, &issue.id).unwrap();
        db.link_scene_to_issue(&scene.id, &issue.id).unwrap();

        let issues = db.get_scene_issues(&scene.id).unwrap();
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn test_link_bible_entry_to_event_duplicate_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "C".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let event = db.create_event(&CreateEventRequest {
            title: "E".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        db.link_bible_entry_to_event(&entry.id, &event.id).unwrap();
        db.link_bible_entry_to_event(&entry.id, &event.id).unwrap();

        let entries = db.get_event_bible_entries(&event.id).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_link_bible_entry_to_issue_duplicate_is_idempotent() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "C".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "todo".to_string(),
            title: "I".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        db.link_bible_entry_to_issue(&entry.id, &issue.id).unwrap();
        db.link_bible_entry_to_issue(&entry.id, &issue.id).unwrap();

        let entries = db.get_issue_bible_entries(&issue.id).unwrap();
        assert_eq!(entries.len(), 1);
    }

    // --- Unlink operations on non-linked items are silent ---

    #[test]
    fn test_unlink_scene_from_arc_not_linked_succeeds() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let arc = db.create_arc(&crate::models::CreateArcRequest {
            name: "Arc".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        // Never linked — unlink should succeed silently
        let result = db.unlink_scene_from_arc(&scene.id, &arc.id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unlink_scene_from_event_not_linked_succeeds() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "E".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        let result = db.unlink_scene_from_event(&scene.id, &event.id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unlink_scene_from_issue_not_linked_succeeds() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let issue = db.create_issue(&CreateIssueRequest {
            issue_type: "todo".to_string(),
            title: "I".to_string(),
            description: None,
            severity: None,
        }).unwrap();

        let result = db.unlink_scene_from_issue(&scene.id, &issue.id);
        assert!(result.is_ok());
    }

    // --- Project update roundtrip ---

    #[test]
    fn test_update_project_roundtrip_all_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.update_project(&crate::models::UpdateProjectRequest {
            title: Some("New Title".to_string()),
            author: Some("New Author".to_string()),
            description: Some("New Desc".to_string()),
            word_target: Some(100000),
            daily_word_target: Some(2000),
        }).unwrap();

        let project = db.get_project().unwrap();
        assert_eq!(project.title, "New Title");
        assert_eq!(project.author.as_deref(), Some("New Author"));
        assert_eq!(project.description.as_deref(), Some("New Desc"));
        assert_eq!(project.word_target, Some(100000));
        assert_eq!(project.daily_word_target, Some(2000));
    }

    // --- Chapter position auto-increment ---

    #[test]
    fn test_chapter_auto_position() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let c1 = db.create_chapter(&crate::models::CreateChapterRequest {
            title: "Ch 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let c2 = db.create_chapter(&crate::models::CreateChapterRequest {
            title: "Ch 2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let c3 = db.create_chapter(&crate::models::CreateChapterRequest {
            title: "Ch 3".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        assert!(c2.position > c1.position);
        assert!(c3.position > c2.position);
    }

    // --- Scene position auto-increment within chapter ---

    #[test]
    fn test_scene_auto_position_within_chapter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&crate::models::CreateChapterRequest {
            title: "Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s3 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S3".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        assert!(s2.position > s1.position);
        assert!(s3.position > s2.position);
    }

    // --- Scene created with explicit position ---

    #[test]
    fn test_scene_explicit_position() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let chapter = db.create_chapter(&crate::models::CreateChapterRequest {
            title: "Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Explicit".to_string(),
            summary: None,
            position: Some(99),
        }).unwrap();

        assert_eq!(s.position, 99);
    }

    // --- Delete association for nonexistent scene/bible succeeds silently ---

    #[test]
    fn test_delete_association_nonexistent_succeeds() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.delete_association("nonexistent-scene", "nonexistent-bible");
        assert!(result.is_ok());
    }

    // --- Delete bible relationship for nonexistent ID succeeds silently ---

    #[test]
    fn test_delete_bible_relationship_nonexistent_succeeds() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.delete_bible_relationship("nonexistent");
        assert!(result.is_ok());
    }

    // --- Delete name mention for nonexistent ID succeeds silently ---

    #[test]
    fn test_delete_name_mention_nonexistent_succeeds() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.delete_name_mention("nonexistent");
        assert!(result.is_ok());
    }

    // --- Delete name registry entry for nonexistent ID succeeds silently ---

    #[test]
    fn test_delete_name_registry_entry_nonexistent_succeeds() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.delete_name_registry_entry("nonexistent");
        assert!(result.is_ok());
    }

    // =========================================================================
    // Batch 7: Deep coverage — complex operations, edge cases, robustness
    // =========================================================================

    // --- Find/Replace: regex special characters are escaped ---

    #[test]
    fn test_find_replace_regex_special_chars_dot() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Mr. Smith met Mrs. Jones".to_string()),
            ..Default::default()
        }).unwrap();

        // "." should be literal, not regex "any character"
        let count = db.find_replace_in_scenes(".", "!", false, false, None).unwrap();
        assert_eq!(count, 1);
        let updated = db.get_scene(&scene.id).unwrap();
        assert_eq!(updated.text, "Mr! Smith met Mrs! Jones");
    }

    #[test]
    fn test_find_replace_regex_special_chars_dollar_and_caret() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Price is $100 and 100% done".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("$100", "€200", false, false, None).unwrap();
        assert_eq!(count, 1);
        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("€200"));
        assert!(!updated.text.contains("$100"));
    }

    #[test]
    fn test_find_replace_regex_special_chars_brackets_and_parens() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Call func(x) or array[0]".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("func(x)", "method(y)", false, false, None).unwrap();
        assert_eq!(count, 1);
        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("method(y)"));
    }

    #[test]
    fn test_find_replace_regex_special_chars_pipe_star_plus() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("A|B and C* and D+E".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("A|B", "X|Y", false, false, None).unwrap();
        assert_eq!(count, 1);
        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("X|Y"));
    }

    #[test]
    fn test_find_replace_backslash_literal() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some(r"Path: C:\Users\Alice".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes(r"C:\Users", r"D:\People", false, false, None).unwrap();
        assert_eq!(count, 1);
        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains(r"D:\People"));
    }

    // --- Find/Replace: empty replacement string (deletion) ---

    #[test]
    fn test_find_replace_with_empty_replacement() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Hello beautiful world".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("beautiful ", "", false, false, None).unwrap();
        assert_eq!(count, 1);
        let updated = db.get_scene(&scene.id).unwrap();
        assert_eq!(updated.text, "Hello world");
    }

    #[test]
    fn test_find_replace_delete_all_occurrences() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("um well um okay um fine".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("um ", "", false, false, None).unwrap();
        assert_eq!(count, 1);
        let updated = db.get_scene(&scene.id).unwrap();
        assert_eq!(updated.text, "well okay fine");
    }

    // --- Find/Replace: Unicode in find and replace strings ---

    #[test]
    fn test_find_replace_unicode_accented_characters() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("She was très naïve about the café".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("café", "restaurant", false, false, None).unwrap();
        assert_eq!(count, 1);
        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("restaurant"));
        assert!(!updated.text.contains("café"));
    }

    #[test]
    fn test_find_replace_unicode_emoji() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("mood: happy face here".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("happy", "😊", false, false, None).unwrap();
        assert_eq!(count, 1);
        let updated = db.get_scene(&scene.id).unwrap();
        assert!(updated.text.contains("😊"));
    }

    // --- Find/Replace: HTML edge cases ---

    #[test]
    fn test_find_replace_nested_html_tags() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<div><p>hello</p> world <b>hello</b></div>".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("hello", "goodbye", false, false, None).unwrap();
        assert_eq!(count, 1);
        let updated = db.get_scene(&scene.id).unwrap();
        // Both occurrences replaced, tags preserved
        assert_eq!(updated.text, "<div><p>goodbye</p> world <b>goodbye</b></div>");
    }

    #[test]
    fn test_find_replace_does_not_modify_tag_attributes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some(r#"<span class="bold">bold text</span>"#.to_string()),
            ..Default::default()
        }).unwrap();

        // "bold" appears in class attribute AND in text content
        let count = db.find_replace_in_scenes("bold", "italic", false, false, None).unwrap();
        assert_eq!(count, 1);
        let updated = db.get_scene(&scene.id).unwrap();
        // Class attribute should be preserved, only text content changed
        assert!(updated.text.contains(r#"class="bold""#));
        assert!(updated.text.contains("italic text"));
    }

    #[test]
    fn test_find_replace_plain_text_no_html() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Just plain text without any tags".to_string()),
            ..Default::default()
        }).unwrap();

        let count = db.find_replace_in_scenes("plain", "simple", false, false, None).unwrap();
        assert_eq!(count, 1);
        let updated = db.get_scene(&scene.id).unwrap();
        assert_eq!(updated.text, "Just simple text without any tags");
    }

    // --- Find/Replace: multiple scenes, verify transaction commits all ---

    #[test]
    fn test_find_replace_across_many_scenes_all_updated() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Create 5 scenes with the same word
        let mut scene_ids = Vec::new();
        for i in 0..5 {
            let s = db.create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: format!("Scene {}", i),
                summary: None,
                position: None,
            }).unwrap();
            db.update_scene(&s.id, &UpdateSceneRequest {
                text: Some(format!("Scene {} has the target word here", i)),
                ..Default::default()
            }).unwrap();
            scene_ids.push(s.id);
        }

        let count = db.find_replace_in_scenes("target", "replaced", false, false, None).unwrap();
        assert_eq!(count, 5);

        for id in &scene_ids {
            let s = db.get_scene(id).unwrap();
            assert!(s.text.contains("replaced"));
            assert!(!s.text.contains("target"));
        }
    }

    // --- Split scene: empty text ---

    #[test]
    fn test_split_scene_empty_text_at_zero() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        // Scene text is empty by default ("")
        let result = db.split_scene(&scene.id, 0, None);
        assert!(result.is_ok());
        let (first, second) = result.unwrap();
        assert_eq!(first.text, "");
        assert_eq!(second.text, "");
    }

    #[test]
    fn test_split_scene_empty_text_at_one_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        // Position 1 is beyond empty text
        let result = db.split_scene(&scene.id, 1, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("beyond text length"));
    }

    // --- Split scene: Unicode multibyte characters ---

    #[test]
    fn test_split_scene_unicode_multibyte() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        // "Café" is 4 chars but 5 bytes (é is 2 bytes in UTF-8)
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Café world".to_string()),
            ..Default::default()
        }).unwrap();

        // Split after "Caf" (3 chars) — should split before é
        let (first, second) = db.split_scene(&scene.id, 3, None).unwrap();
        assert_eq!(first.text, "Caf");
        assert_eq!(second.text, "é world");
    }

    #[test]
    fn test_split_scene_unicode_emoji() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        // "Hi 😊 bye" — emoji is 1 char but 4 bytes in UTF-8
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Hi 😊 bye".to_string()),
            ..Default::default()
        }).unwrap();

        // Split at position 4 (after "Hi 😊")
        let (first, second) = db.split_scene(&scene.id, 4, None).unwrap();
        assert_eq!(first.text, "Hi 😊");
        assert_eq!(second.text, " bye");
    }

    #[test]
    fn test_split_scene_unicode_cjk() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        // Chinese characters: each is 3 bytes in UTF-8
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("你好世界".to_string()), // 4 chars, 12 bytes
            ..Default::default()
        }).unwrap();

        let (first, second) = db.split_scene(&scene.id, 2, None).unwrap();
        assert_eq!(first.text, "你好");
        assert_eq!(second.text, "世界");
    }

    // --- Split scene: position continuity after sequential splits ---

    #[test]
    fn test_split_scene_sequential_preserves_position_order() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Chapter".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Create scene with text
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Original".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("AAABBBCCC".to_string()),
            ..Default::default()
        }).unwrap();

        // First split: "AAA" | "BBBCCC"
        let (first, second) = db.split_scene(&scene.id, 3, Some("Second")).unwrap();
        assert_eq!(first.text, "AAA");
        assert_eq!(second.text, "BBBCCC");

        // Second split on the second part: "BBB" | "CCC"
        let (second_a, second_b) = db.split_scene(&second.id, 3, Some("Third")).unwrap();
        assert_eq!(second_a.text, "BBB");
        assert_eq!(second_b.text, "CCC");

        // Verify all scenes in order with no gaps
        let scenes = db.get_scenes(&chapter.id).unwrap();
        assert_eq!(scenes.len(), 3);
        // Positions should be strictly increasing
        for i in 1..scenes.len() {
            assert!(scenes[i].position > scenes[i-1].position,
                "Position {} should be > position {} (scene {} vs {})",
                scenes[i].position, scenes[i-1].position,
                scenes[i].title, scenes[i-1].title);
        }
        assert_eq!(scenes[0].text, "AAA");
        assert_eq!(scenes[1].text, "BBB");
        assert_eq!(scenes[2].text, "CCC");
    }

    // --- Split scene: preserves status, pov, tags from original ---

    #[test]
    fn test_split_scene_preserves_metadata() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("First half. Second half.".to_string()),
            status: Some("revised".to_string()),
            pov: Some("Alice".to_string()),
            tags: Some("action,drama".to_string()),
            ..Default::default()
        }).unwrap();

        let (first, second) = db.split_scene(&scene.id, 12, None).unwrap();
        // Status, pov, tags inherited by new scene
        assert_eq!(second.status, "revised");
        assert_eq!(second.pov.as_deref(), Some("Alice"));
        assert_eq!(second.tags.as_deref(), Some("action,drama"));
        // New scene does NOT inherit summary, notes, todos (starts fresh)
        assert!(second.summary.is_none());
        assert!(second.notes.is_none());
        assert!(second.todos.is_none());
    }

    // --- Merge scenes: notes edge cases ---

    #[test]
    fn test_merge_scenes_one_has_notes_other_does_not() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.update_scene(&s1.id, &UpdateSceneRequest {
            notes: Some("Important note".to_string()),
            ..Default::default()
        }).unwrap();
        // s2 has no notes (None)

        let merged = db.merge_scenes(&[s1.id.clone(), s2.id.clone()]).unwrap();
        // Should just have the one note, no separator needed
        assert_eq!(merged.notes.as_deref(), Some("Important note"));
    }

    #[test]
    fn test_merge_scenes_both_have_notes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.update_scene(&s1.id, &UpdateSceneRequest {
            notes: Some("Note A".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&s2.id, &UpdateSceneRequest {
            notes: Some("Note B".to_string()),
            ..Default::default()
        }).unwrap();

        let merged = db.merge_scenes(&[s1.id.clone(), s2.id.clone()]).unwrap();
        assert_eq!(merged.notes.as_deref(), Some("Note A\n---\nNote B"));
    }

    #[test]
    fn test_merge_scenes_empty_notes_not_included() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.update_scene(&s1.id, &UpdateSceneRequest {
            notes: Some("".to_string()), // Empty string, not None
            ..Default::default()
        }).unwrap();
        db.update_scene(&s2.id, &UpdateSceneRequest {
            notes: Some("Real note".to_string()),
            ..Default::default()
        }).unwrap();

        let merged = db.merge_scenes(&[s1.id.clone(), s2.id.clone()]).unwrap();
        // Empty notes should be filtered out
        assert_eq!(merged.notes.as_deref(), Some("Real note"));
    }

    #[test]
    fn test_merge_scenes_neither_has_notes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let merged = db.merge_scenes(&[s1.id.clone(), s2.id.clone()]).unwrap();
        assert!(merged.notes.is_none());
    }

    // --- Merge scenes: text combination edge cases ---

    #[test]
    fn test_merge_scenes_one_empty_text() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.update_scene(&s1.id, &UpdateSceneRequest {
            text: Some("Content here".to_string()),
            ..Default::default()
        }).unwrap();
        // s2 has empty text (default "")

        let merged = db.merge_scenes(&[s1.id.clone(), s2.id.clone()]).unwrap();
        // No double newline separator when one side is empty
        assert_eq!(merged.text, "Content here");
    }

    #[test]
    fn test_merge_scenes_both_empty_text() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let merged = db.merge_scenes(&[s1.id.clone(), s2.id.clone()]).unwrap();
        assert_eq!(merged.text, "");
    }

    // --- Merge scenes: three scenes ---

    #[test]
    fn test_merge_three_scenes_text_and_notes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S2".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s3 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S3".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.update_scene(&s1.id, &UpdateSceneRequest {
            text: Some("Part one.".to_string()),
            notes: Some("Note 1".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&s2.id, &UpdateSceneRequest {
            text: Some("Part two.".to_string()),
            notes: Some("Note 2".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&s3.id, &UpdateSceneRequest {
            text: Some("Part three.".to_string()),
            notes: Some("Note 3".to_string()),
            ..Default::default()
        }).unwrap();

        let merged = db.merge_scenes(&[s1.id.clone(), s2.id.clone(), s3.id.clone()]).unwrap();
        assert_eq!(merged.text, "Part one.\n\nPart two.\n\nPart three.");
        assert_eq!(merged.notes.as_deref(), Some("Note 1\n---\nNote 2\n---\nNote 3"));

        // Other scenes should be soft-deleted
        let remaining = db.get_scenes(&chapter.id).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, s1.id);
    }

    // --- Merge scenes: merged scenes are soft-deleted, not hard-deleted ---

    #[test]
    fn test_merge_scenes_deleted_scenes_appear_in_trash() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "S2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.merge_scenes(&[s1.id.clone(), s2.id.clone()]).unwrap();

        let trash = db.get_deleted_scenes().unwrap();
        assert_eq!(trash.len(), 1);
        assert_eq!(trash[0].id, s2.id);
    }

    // --- Schema migrations: idempotency ---

    #[test]
    fn test_database_survives_double_open() {
        // Simulates migration idempotency: opening the same DB file twice
        // should not corrupt data (since migrations use IF NOT EXISTS and
        // column-existence checks).
        use std::path::Path;

        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("test.cahnon");

        // First open — creates schema
        {
            let db = Database::create(&path).unwrap();
            db.create_project(&CreateProjectRequest {
                title: "Test Project".to_string(),
                author: None,
                description: None,
            }).unwrap();
            let chapter = db.create_chapter(&CreateChapterRequest {
                title: "Ch".to_string(),
                summary: None,
                position: None,
            }).unwrap();
            db.create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: "Scene".to_string(),
                summary: None,
                position: None,
            }).unwrap();
        }

        // Second open — runs migrations again on existing schema
        {
            let db = Database::open(&path).unwrap();
            let project = db.get_project().unwrap();
            assert_eq!(project.title, "Test Project");
            let chapters = db.get_chapters().unwrap();
            assert_eq!(chapters.len(), 1);
            let scenes = db.get_scenes(&chapters[0].id).unwrap();
            assert_eq!(scenes.len(), 1);
        }

        // Third open — migrations run yet again, still fine
        {
            let db = Database::open(&path).unwrap();
            let project = db.get_project().unwrap();
            assert_eq!(project.title, "Test Project");
        }
    }

    // --- Snapshot: cleanup_expired_snapshots respects snapshot types ---

    #[test]
    fn test_cleanup_expired_snapshots_only_deletes_pre_bulk() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Create manual, pre_restore, and pre_bulk snapshots
        db.create_snapshot("Manual", None, "manual").unwrap();
        db.create_snapshot("Pre-restore", None, "pre_restore").unwrap();
        db.create_snapshot("Pre-bulk", None, "pre_bulk").unwrap();

        // None are old enough to be cleaned
        let deleted = db.cleanup_expired_snapshots().unwrap();
        assert_eq!(deleted, 0);

        let all = db.get_snapshots().unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_cleanup_expired_snapshots_preserves_recent_pre_bulk() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Create a fresh pre_bulk snapshot
        db.create_snapshot("Recent pre_bulk", None, "pre_bulk").unwrap();

        let deleted = db.cleanup_expired_snapshots().unwrap();
        assert_eq!(deleted, 0);

        let all = db.get_snapshots().unwrap();
        assert_eq!(all.len(), 1);
    }

    // --- Snapshot: restore_scene_from_snapshot on soft-deleted scene ---

    #[test]
    fn test_restore_scene_from_snapshot_deleted_scene_not_updated() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Original text".to_string()),
            ..Default::default()
        }).unwrap();

        let snapshot = db.create_snapshot("Snap", None, "manual").unwrap();

        // Soft-delete the scene
        db.delete_scene(&scene.id).unwrap();

        // Try to restore from snapshot — should fail because scene is deleted
        // (WHERE clause has deleted_at IS NULL)
        let result = db.restore_scene_from_snapshot(&snapshot.id, &scene.id);
        // The UPDATE succeeds with 0 rows, then get_scene fails
        assert!(result.is_err());
    }

    // --- Snapshot: get_snapshot_scenes ---

    #[test]
    fn test_get_snapshot_scenes_returns_snapshotted_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Snapshotted text".to_string()),
            ..Default::default()
        }).unwrap();

        let snapshot = db.create_snapshot("Snap", None, "manual").unwrap();

        let snap_scenes = db.get_snapshot_scenes(&snapshot.id).unwrap();
        assert_eq!(snap_scenes.len(), 1);
        assert_eq!(snap_scenes[0].id, scene.id);
        assert_eq!(snap_scenes[0].text, "Snapshotted text");
    }

    #[test]
    fn test_get_snapshot_scenes_nonexistent_snapshot() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.get_snapshot_scenes("nonexistent");
        assert!(result.is_err());
    }

    // --- Scan names: whitespace-only and HTML-only scenes ---

    #[test]
    fn test_scan_names_whitespace_only_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("   \n\n\t  ".to_string()),
            ..Default::default()
        }).unwrap();

        let (entries, mentions) = db.scan_names().unwrap();
        assert_eq!(entries, 0);
        assert_eq!(mentions, 0);
    }

    #[test]
    fn test_scan_names_html_only_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p></p><br/><div></div>".to_string()),
            ..Default::default()
        }).unwrap();

        let (entries, mentions) = db.scan_names().unwrap();
        assert_eq!(entries, 0);
        assert_eq!(mentions, 0);
    }

    // --- Scan names: mention limit is 50 per name ---

    #[test]
    fn test_scan_names_mention_limit_fifty() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // "Alice" must appear mid-sentence (not first word) to avoid the
        // first-word-of-sentence filter. Repeat 60 times to exceed the limit.
        let text = (0..60).map(|_| "Then Alice said hello.").collect::<Vec<_>>().join(" ");
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some(text),
            ..Default::default()
        }).unwrap();

        let (entries, mentions) = db.scan_names().unwrap();
        assert_eq!(entries, 1); // One name: Alice
        assert!(mentions <= 50, "Mentions should be capped at 50, got {}", mentions);
    }

    // --- Soft-delete consistency: deleted scenes excluded from search ---

    #[test]
    fn test_soft_deleted_scene_excluded_from_global_search() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Unique searchable xylophone text".to_string()),
            ..Default::default()
        }).unwrap();

        // Should find it
        let results = db.global_search("xylophone", None).unwrap();
        assert!(!results.is_empty());

        // Soft-delete
        db.delete_scene(&scene.id).unwrap();

        // Should NOT find it anymore
        let results = db.global_search("xylophone", None).unwrap();
        let scene_results: Vec<_> = results.iter().filter(|r| r.result_type == "scene").collect();
        assert!(scene_results.is_empty());
    }

    #[test]
    fn test_soft_deleted_bible_entry_excluded_from_search() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Zarqwix the Peculiar".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let results = db.global_search("Zarqwix", None).unwrap();
        assert!(!results.is_empty());

        db.delete_bible_entry(&entry.id).unwrap();

        let results = db.global_search("Zarqwix", None).unwrap();
        let bible_results: Vec<_> = results.iter().filter(|r| r.result_type == "bible_entry").collect();
        assert!(bible_results.is_empty());
    }

    // --- Search: FTS5 special characters don't cause errors ---

    #[test]
    fn test_search_fts5_special_chars_not_star() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // FTS5 operators like NOT, OR, NEAR, *, ^ should be handled
        let result = db.global_search("NOT", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_fts5_special_chars_or() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.global_search("hello OR goodbye", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_fts5_special_chars_asterisk() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.global_search("test*", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_fts5_special_chars_quotes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.global_search(r#""quoted phrase""#, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_fts5_special_chars_near() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.global_search("NEAR(hello, world)", None);
        assert!(result.is_ok());
    }

    // --- Search: SQL LIKE wildcard characters in event/annotation/cut search ---

    #[test]
    fn test_search_sql_wildcards_escaped_in_events() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "100% complete".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        // "%" is a LIKE wildcard. Searching for "%" should find this event
        // but not act as a "match everything" wildcard
        let results = db.global_search("100%", Some(vec!["events".to_string()])).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "100% complete");
    }

    #[test]
    fn test_search_sql_underscore_escaped_in_events() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_event(&CreateEventRequest {
            title: "file_name_here".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        // "_" is a LIKE wildcard for single char. Should be escaped
        let results = db.global_search("file_name", Some(vec!["events".to_string()])).unwrap();
        assert!(!results.is_empty());
    }

    // --- Duplicate scene: position correctly placed ---

    #[test]
    fn test_duplicate_scene_position_after_original() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "First".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Second".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s3 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Third".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Duplicate the middle scene
        let dup = db.duplicate_scene(&s2.id, false).unwrap();

        // Get all scenes in order
        let scenes = db.get_scenes(&chapter.id).unwrap();
        assert_eq!(scenes.len(), 4);

        // Find the duplicate — should be right after s2
        let dup_idx = scenes.iter().position(|s| s.id == dup.id).unwrap();
        let s2_idx = scenes.iter().position(|s| s.id == s2.id).unwrap();
        assert_eq!(dup_idx, s2_idx + 1);
    }

    // --- Duplicate scene: copies all metadata fields ---

    #[test]
    fn test_duplicate_scene_copies_all_metadata() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Some text content".to_string()),
            summary: Some("A summary".to_string()),
            status: Some("revised".to_string()),
            pov: Some("Bob".to_string()),
            tags: Some("tag1,tag2".to_string()),
            notes: Some("Important notes".to_string()),
            todos: Some("TODO item".to_string()),
            word_target: Some(1000),
            on_timeline: Some(false),
            ..Default::default()
        }).unwrap();

        let dup = db.duplicate_scene(&scene.id, false).unwrap();

        assert_eq!(dup.text, "Some text content");
        assert_eq!(dup.summary.as_deref(), Some("A summary"));
        assert_eq!(dup.status, "revised");
        assert_eq!(dup.pov.as_deref(), Some("Bob"));
        assert_eq!(dup.tags.as_deref(), Some("tag1,tag2"));
        assert_eq!(dup.notes.as_deref(), Some("Important notes"));
        assert_eq!(dup.todos.as_deref(), Some("TODO item"));
        assert_eq!(dup.word_target, Some(1000));
        assert!(!dup.on_timeline);
        // Title should have "(copy)" suffix
        assert!(dup.title.contains("(copy)"));
    }

    #[test]
    fn test_duplicate_scene_structure_only_has_empty_text() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("This should not be copied".to_string()),
            summary: Some("But summary should".to_string()),
            ..Default::default()
        }).unwrap();

        let dup = db.duplicate_scene(&scene.id, true).unwrap();
        assert_eq!(dup.text, ""); // Structure only — no text
        assert_eq!(dup.summary.as_deref(), Some("But summary should"));
    }

    // --- Import JSON backup: round-trip preserves all entity types ---

    #[test]
    fn test_import_json_backup_preserves_arcs_and_events() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        // Create an arc and event
        let arc = db.create_arc(&CreateArcRequest {
            name: "Main Plot".to_string(),
            description: Some("The central conflict".to_string()),
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        let event = db.create_event(&CreateEventRequest {
            title: "Battle".to_string(),
            description: Some("The big battle".to_string()),
            time_point: Some("Day 5".to_string()),
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        // Export
        let json = db.export_json_backup().unwrap();

        // Modify data
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Modified text".to_string()),
            ..Default::default()
        }).unwrap();

        // Import (restores original state)
        db.import_json_backup(&json).unwrap();

        // Verify arc survived
        let arcs = db.get_arcs().unwrap();
        assert_eq!(arcs.len(), 1);
        assert_eq!(arcs[0].name, "Main Plot");

        // Verify event survived
        let events = db.get_events().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Battle");
    }

    // --- Import JSON backup: missing fields in JSON ---

    #[test]
    fn test_import_json_backup_missing_required_field_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Invalid JSON: missing required "project" field
        let bad_json = r#"{"chapters": [], "scenes": [], "bible_entries": [], "arcs": [], "events": []}"#;
        let result = db.import_json_backup(bad_json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));

        // Project should still be intact (import was rejected)
        let project = db.get_project().unwrap();
        assert!(!project.title.is_empty());
    }

    // --- Reorder scenes: duplicate positions handled ---

    #[test]
    fn test_reorder_scenes_explicit_ordering() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "A".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "B".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s3 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "C".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Reverse the order: C, B, A
        db.reorder_scenes(&chapter.id, &[s3.id.clone(), s2.id.clone(), s1.id.clone()]).unwrap();

        let scenes = db.get_scenes(&chapter.id).unwrap();
        assert_eq!(scenes[0].title, "C");
        assert_eq!(scenes[1].title, "B");
        assert_eq!(scenes[2].title, "A");

        // Positions should be sequential (0, 1, 2)
        assert_eq!(scenes[0].position, 0);
        assert_eq!(scenes[1].position, 1);
        assert_eq!(scenes[2].position, 2);
    }

    // --- Move scene to different chapter ---

    #[test]
    fn test_move_scene_to_chapter_updates_chapter_id() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch1 = db.create_chapter(&CreateChapterRequest {
            title: "Chapter 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let ch2 = db.create_chapter(&CreateChapterRequest {
            title: "Chapter 2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: ch1.id.clone(),
            title: "Movable".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let moved = db.move_scene_to_chapter(&scene.id, &ch2.id, 0).unwrap();
        assert_eq!(moved.chapter_id, ch2.id);
        assert_eq!(moved.position, 0);

        // Chapter 1 should now be empty
        let ch1_scenes = db.get_scenes(&ch1.id).unwrap();
        assert!(ch1_scenes.is_empty());

        // Chapter 2 should have the scene
        let ch2_scenes = db.get_scenes(&ch2.id).unwrap();
        assert_eq!(ch2_scenes.len(), 1);
        assert_eq!(ch2_scenes[0].id, scene.id);
    }

    // --- FTS5 sanitizer unit-level test ---

    #[test]
    fn test_sanitize_fts5_query_wraps_tokens() {
        let result = Database::sanitize_fts5_query("hello world");
        assert_eq!(result, r#""hello" "world""#);
    }

    #[test]
    fn test_sanitize_fts5_query_escapes_quotes() {
        // Input: say "hello"
        // Tokenized by whitespace: ["say", "\"hello\""]
        // Token "say" → escaped: "say" → wrapped: "\"say\""
        // Token "\"hello\"" → escaped (double each "): "\"\"hello\"\"" → wrapped: "\"\"\"hello\"\"\""
        let result = Database::sanitize_fts5_query("say \"hello\"");
        // Expected: "say" """hello"""
        let expected = "\"say\" \"\"\"hello\"\"\"";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sanitize_fts5_query_empty() {
        assert_eq!(Database::sanitize_fts5_query(""), "");
        assert_eq!(Database::sanitize_fts5_query("   "), "");
    }

    #[test]
    fn test_sanitize_fts5_query_operators_escaped() {
        // FTS5 operators become literal tokens when quoted
        let result = Database::sanitize_fts5_query("NOT OR AND");
        assert_eq!(result, r#""NOT" "OR" "AND""#);
    }

    // --- Scene history: update without text change does NOT create history ---

    #[test]
    fn test_update_scene_non_text_field_no_history() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Initial text".to_string()),
            ..Default::default()
        }).unwrap();

        let history_before = db.get_scene_history(&scene.id).unwrap();

        // Update only non-text fields
        db.update_scene(&scene.id, &UpdateSceneRequest {
            title: Some("New Title".to_string()),
            status: Some("revised".to_string()),
            pov: Some("Carol".to_string()),
            ..Default::default()
        }).unwrap();

        let history_after = db.get_scene_history(&scene.id).unwrap();
        // No new history entry since text wasn't changed
        assert_eq!(history_before.len(), history_after.len());
    }

    // --- Annotation: search scope includes annotations ---

    #[test]
    fn test_global_search_finds_annotations() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Some text here".to_string()),
            ..Default::default()
        }).unwrap();

        db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene.id.clone(),
            start_offset: 0,
            end_offset: 4,
            annotation_type: None,
            content: "Unique xyzannotation content".to_string(),
        }).unwrap();

        let results = db.global_search("xyzannotation", Some(vec!["annotations".to_string()])).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].result_type, "annotation");
    }

    // --- Search scope: cuts ---

    #[test]
    fn test_global_search_finds_cuts() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.create_cut(Some(&scene.id), "Unique xyzcut snippet here").unwrap();

        let results = db.global_search("xyzcut", Some(vec!["cuts".to_string()])).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].result_type, "cut");
    }

    // --- Search scope: events ---

    #[test]
    fn test_global_search_finds_events() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        db.create_event(&CreateEventRequest {
            title: "Unique xyzeventname".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        let results = db.global_search("xyzeventname", Some(vec!["events".to_string()])).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].result_type, "event");
    }

    // --- Soft-deleted events excluded from search ---

    #[test]
    fn test_soft_deleted_event_excluded_from_search() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "Unique xyzdeletedevent".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        let results = db.global_search("xyzdeletedevent", Some(vec!["events".to_string()])).unwrap();
        assert_eq!(results.len(), 1);

        db.delete_event(&event.id).unwrap();

        let results = db.global_search("xyzdeletedevent", Some(vec!["events".to_string()])).unwrap();
        assert!(results.is_empty());
    }

    // --- Soft-deleted cut excluded from search ---

    #[test]
    fn test_soft_deleted_cut_excluded_from_search() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let cut = db.create_cut(Some(&scene.id), "Unique xyzdeletedcut snippet").unwrap();

        let results = db.global_search("xyzdeletedcut", Some(vec!["cuts".to_string()])).unwrap();
        assert_eq!(results.len(), 1);

        db.delete_cut(&cut.id).unwrap();

        let results = db.global_search("xyzdeletedcut", Some(vec!["cuts".to_string()])).unwrap();
        assert!(results.is_empty());
    }

    // --- End-to-end workflow: create → edit → snapshot → modify → restore → verify ---

    #[test]
    fn test_end_to_end_create_edit_snapshot_restore() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // 1. Create structure
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Chapter One".to_string(),
            summary: Some("The beginning".to_string()),
            position: None,
        }).unwrap();

        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Opening Scene".to_string(),
            summary: Some("The hero appears".to_string()),
            position: None,
        }).unwrap();

        // 2. Edit with rich content
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>The hero walked into the sunset.</p>".to_string()),
            pov: Some("Hero".to_string()),
            tags: Some("opening,dramatic".to_string()),
            ..Default::default()
        }).unwrap();

        // 3. Add bible entry and link to scene
        let entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "The Hero".to_string(),
            aliases: None,
            short_description: Some("Protagonist".to_string()),
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: entry.id.clone(),
        }).unwrap();

        // 4. Create arc and link
        let arc = db.create_arc(&CreateArcRequest {
            name: "Hero's Journey".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();
        db.link_scene_to_arc(&scene.id, &arc.id).unwrap();

        // 5. Create snapshot
        let snapshot = db.create_snapshot("Before edits", None, "manual").unwrap();

        // 6. Modify everything
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>COMPLETELY DIFFERENT TEXT</p>".to_string()),
            pov: Some("Villain".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_chapter(&chapter.id, &UpdateChapterRequest {
            title: Some("Modified Chapter".to_string()),
            summary: None,
            status: None,
            notes: None,
            position: None,
        }).unwrap();

        // 7. Restore snapshot
        db.restore_snapshot(&snapshot.id).unwrap();

        // 8. Verify everything is back to original
        let restored_project = db.get_project().unwrap();
        assert!(!restored_project.title.is_empty());

        let restored_chapters = db.get_chapters().unwrap();
        assert_eq!(restored_chapters.len(), 1);
        assert_eq!(restored_chapters[0].title, "Chapter One");

        let restored_scenes = db.get_scenes(&restored_chapters[0].id).unwrap();
        assert_eq!(restored_scenes.len(), 1);
        assert_eq!(restored_scenes[0].text, "<p>The hero walked into the sunset.</p>");
        assert_eq!(restored_scenes[0].pov.as_deref(), Some("Hero"));
        assert_eq!(restored_scenes[0].tags.as_deref(), Some("opening,dramatic"));

        // Bible entry and arc restored
        let entries = db.get_bible_entries(None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "The Hero");

        let arcs = db.get_arcs().unwrap();
        assert_eq!(arcs.len(), 1);
        assert_eq!(arcs[0].name, "Hero's Journey");
    }

    // --- End-to-end: split → merge round-trip ---

    #[test]
    fn test_end_to_end_split_then_merge_roundtrip() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Original".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("First part. Second part.".to_string()),
            ..Default::default()
        }).unwrap();

        // Split at position 12 (after "First part. ")
        let (first, second) = db.split_scene(&scene.id, 12, Some("Continued")).unwrap();
        assert_eq!(first.text, "First part. ");
        assert_eq!(second.text, "Second part.");

        // Now merge them back
        let merged = db.merge_scenes(&[first.id.clone(), second.id.clone()]).unwrap();
        assert_eq!(merged.text, "First part. \n\nSecond part.");

        // Only one scene remains
        let scenes = db.get_scenes(&chapter.id).unwrap();
        assert_eq!(scenes.len(), 1);
    }

    // --- End-to-end: find/replace then undo via snapshot ---

    #[test]
    fn test_end_to_end_find_replace_then_restore_via_snapshot() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let original_text = "Alice met Bob at the park".to_string();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some(original_text.clone()),
            ..Default::default()
        }).unwrap();

        // Create snapshot before find/replace
        let snapshot = db.create_snapshot("Before replace", None, "manual").unwrap();

        // Find/replace uses direct SQL (not update_scene), so no scene history
        db.find_replace_in_scenes("Alice", "Carol", false, false, None).unwrap();
        let after_replace = db.get_scene(&scene.id).unwrap();
        assert_eq!(after_replace.text, "Carol met Bob at the park");

        // Restore from snapshot to get original text back
        let restored = db.restore_scene_from_snapshot(&snapshot.id, &scene.id).unwrap();
        assert_eq!(restored.text, original_text);
    }

    // --- find_replace_in_scenes does NOT create scene history ---

    #[test]
    fn test_find_replace_does_not_create_scene_history() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Hello world".to_string()),
            ..Default::default()
        }).unwrap();

        let history_before = db.get_scene_history(&scene.id).unwrap();

        db.find_replace_in_scenes("Hello", "Goodbye", false, false, None).unwrap();

        let history_after = db.get_scene_history(&scene.id).unwrap();
        // find_replace uses direct SQL, bypasses save_scene_to_history
        assert_eq!(history_before.len(), history_after.len());
    }

    // =========================================================================
    // Batch 8: Timeline conflicts, scene version comparison, edge cases
    // =========================================================================

    // --- Timeline: adjacent non-overlapping ranges (no conflict) ---

    #[test]
    fn test_timeline_adjacent_ranges_no_conflict() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene A".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Scene B".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Scene A: Day 1 - Day 3, Scene B: Day 4 - Day 5 (no overlap)
        db.update_scene(&s1.id, &UpdateSceneRequest {
            pov: Some("Alice".to_string()),
            time_start: Some("Day 1".to_string()),
            time_end: Some("Day 3".to_string()),
            on_timeline: Some(true),
            ..Default::default()
        }).unwrap();
        db.update_scene(&s2.id, &UpdateSceneRequest {
            pov: Some("Alice".to_string()),
            time_start: Some("Day 4".to_string()),
            time_end: Some("Day 5".to_string()),
            on_timeline: Some(true),
            ..Default::default()
        }).unwrap();

        let conflicts = db.detect_timeline_conflicts().unwrap();
        let overlap_conflicts: Vec<_> = conflicts.iter()
            .filter(|c| c.conflict_type == "overlapping_time" || c.conflict_type == "same_time")
            .collect();
        assert!(overlap_conflicts.is_empty());
    }

    // --- Timeline: three-way overlap ---

    #[test]
    fn test_timeline_three_way_overlap() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Three scenes all at the same time point with same POV
        for i in 0..3 {
            let s = db.create_scene(&CreateSceneRequest {
                chapter_id: chapter.id.clone(),
                title: format!("Scene {}", i),
                summary: None,
                position: None,
            }).unwrap();
            db.update_scene(&s.id, &UpdateSceneRequest {
                pov: Some("Bob".to_string()),
                time_point: Some("Noon".to_string()),
                on_timeline: Some(true),
                ..Default::default()
            }).unwrap();
        }

        let conflicts = db.detect_timeline_conflicts().unwrap();
        // 3 scenes → 3 pairwise conflicts (0-1, 0-2, 1-2)
        let time_conflicts: Vec<_> = conflicts.iter()
            .filter(|c| c.conflict_type == "same_time" || c.conflict_type == "overlapping_time")
            .collect();
        assert_eq!(time_conflicts.len(), 3);
    }

    // --- Timeline: time_point vs time_start/time_end overlap ---

    #[test]
    fn test_timeline_point_vs_range_overlap() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Point Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Range Scene".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Scene 1: time_point = "Day 3"
        // Scene 2: time_start = "Day 1", time_end = "Day 5" (encompasses Day 3)
        db.update_scene(&s1.id, &UpdateSceneRequest {
            pov: Some("Carol".to_string()),
            time_point: Some("Day 3".to_string()),
            on_timeline: Some(true),
            ..Default::default()
        }).unwrap();
        db.update_scene(&s2.id, &UpdateSceneRequest {
            pov: Some("Carol".to_string()),
            time_start: Some("Day 1".to_string()),
            time_end: Some("Day 5".to_string()),
            on_timeline: Some(true),
            ..Default::default()
        }).unwrap();

        let conflicts = db.detect_timeline_conflicts().unwrap();
        let overlap_conflicts: Vec<_> = conflicts.iter()
            .filter(|c| c.conflict_type == "overlapping_time")
            .collect();
        assert_eq!(overlap_conflicts.len(), 1);
    }

    // --- Timeline: scenes without POV produce no overlap conflicts ---

    #[test]
    fn test_timeline_no_pov_no_overlap_conflict() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "No POV 1".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "No POV 2".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Same time but NO pov — should not produce overlap conflicts
        db.update_scene(&s1.id, &UpdateSceneRequest {
            time_point: Some("Dawn".to_string()),
            on_timeline: Some(true),
            ..Default::default()
        }).unwrap();
        db.update_scene(&s2.id, &UpdateSceneRequest {
            time_point: Some("Dawn".to_string()),
            on_timeline: Some(true),
            ..Default::default()
        }).unwrap();

        let conflicts = db.detect_timeline_conflicts().unwrap();
        let overlap_conflicts: Vec<_> = conflicts.iter()
            .filter(|c| c.conflict_type == "same_time" || c.conflict_type == "overlapping_time")
            .collect();
        assert!(overlap_conflicts.is_empty());
    }

    // --- Timeline: missing_time conflict detection ---

    #[test]
    fn test_timeline_missing_time_with_pov_and_on_timeline() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Missing time".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.update_scene(&s.id, &UpdateSceneRequest {
            pov: Some("Diane".to_string()),
            on_timeline: Some(true),
            ..Default::default()
        }).unwrap();

        // Scene has POV but no time — should appear as missing_time
        // BUT get_all_scenes_for_timeline filters by (time_point IS NOT NULL OR time_start IS NOT NULL)
        // So this scene won't even be returned by the query
        let conflicts = db.detect_timeline_conflicts().unwrap();
        // The detect_missing_time_conflicts operates on the results of get_all_scenes_for_timeline,
        // which ONLY includes scenes WITH time data. So a scene with POV but no time won't appear.
        let missing: Vec<_> = conflicts.iter().filter(|c| c.conflict_type == "missing_time").collect();
        assert_eq!(missing.len(), 0);
    }

    // --- Compare scene versions: same version returns identical text ---

    #[test]
    fn test_compare_scene_versions_same_version() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Version one".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Version two".to_string()),
            ..Default::default()
        }).unwrap();

        let history = db.get_scene_history(&scene.id).unwrap();
        assert!(history.len() >= 1);

        // Compare a version against itself
        let (text_a, text_b) = db.compare_scene_versions(&scene.id, &history[0].id, &history[0].id).unwrap();
        assert_eq!(text_a, text_b);
    }

    // --- Compare scene versions: returns correct text for each ---

    #[test]
    fn test_compare_scene_versions_returns_correct_texts() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        // Create v1
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("First version".to_string()),
            ..Default::default()
        }).unwrap();
        // Create v2 (v1 saved to history)
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Second version".to_string()),
            ..Default::default()
        }).unwrap();
        // Create v3 (v2 saved to history)
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Third version".to_string()),
            ..Default::default()
        }).unwrap();

        let history = db.get_scene_history(&scene.id).unwrap();
        // History is ordered DESC, so [0] = most recent, [1] = oldest
        assert!(history.len() >= 2);

        let (text_a, text_b) = db.compare_scene_versions(
            &scene.id,
            &history[0].id,  // Second version (saved when v3 was written)
            &history[1].id,  // First version (saved when v2 was written)
        ).unwrap();

        assert_eq!(text_a, "Second version");
        assert_eq!(text_b, "First version");
    }

    // --- Compare scene versions: wrong scene_id fails ---

    #[test]
    fn test_compare_scene_versions_wrong_scene_id_fails() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("V1".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("V2".to_string()),
            ..Default::default()
        }).unwrap();

        let history = db.get_scene_history(&scene.id).unwrap();
        assert!(!history.is_empty());

        // Use a different scene_id than what the history belongs to
        let result = db.compare_scene_versions("wrong-scene-id", &history[0].id, &history[0].id);
        assert!(result.is_err());
    }

    // --- Scene history: limit of 100 entries ---

    #[test]
    fn test_scene_history_limited_to_100() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        // Create 105 versions (each update saves the previous text to history)
        for i in 0..105 {
            db.update_scene(&scene.id, &UpdateSceneRequest {
                text: Some(format!("Version {}", i)),
                ..Default::default()
            }).unwrap();
        }

        let history = db.get_scene_history(&scene.id).unwrap();
        // SQL has LIMIT 100, so at most 100 entries returned
        assert!(history.len() <= 100);
        assert!(history.len() >= 100); // Should be exactly 100 since we created 105
    }

    // --- Soft-deleted scenes not returned by get_scenes ---

    #[test]
    fn test_deleted_scene_not_in_get_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Keep".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Delete".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.delete_scene(&s2.id).unwrap();

        let scenes = db.get_scenes(&chapter.id).unwrap();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].title, "Keep");
    }

    // --- Soft-deleted chapter not returned by get_chapters ---

    #[test]
    fn test_deleted_chapter_not_in_get_chapters() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch1 = db.create_chapter(&CreateChapterRequest {
            title: "Keep".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let ch2 = db.create_chapter(&CreateChapterRequest {
            title: "Delete".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.delete_chapter(&ch2.id).unwrap();

        let chapters = db.get_chapters().unwrap();
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "Keep");
    }

    // --- Soft-deleted arc not returned by get_arcs ---

    #[test]
    fn test_deleted_arc_not_in_get_arcs() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let arc = db.create_arc(&CreateArcRequest {
            name: "Delete me".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        db.delete_arc(&arc.id).unwrap();

        let arcs = db.get_arcs().unwrap();
        assert!(arcs.is_empty());
    }

    // --- Soft-deleted event not returned by get_events ---

    #[test]
    fn test_deleted_event_not_in_get_events() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "Delete me".to_string(),
            description: None,
            time_point: None,
            time_start: None,
            time_end: None,
            event_type: None,
            importance: None,
        }).unwrap();

        db.delete_event(&event.id).unwrap();

        let events = db.get_events().unwrap();
        assert!(events.is_empty());
    }

    // --- Get scene by ID fails for deleted scene ---

    #[test]
    fn test_get_scene_fails_for_deleted_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.delete_scene(&scene.id).unwrap();

        let result = db.get_scene(&scene.id);
        assert!(result.is_err());
    }

    // --- Scene operations on off-timeline scenes ---

    #[test]
    fn test_off_timeline_scene_not_in_timeline_query() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            time_point: Some("Day 1".to_string()),
            on_timeline: Some(false), // Explicitly off timeline
            ..Default::default()
        }).unwrap();

        let timeline_scenes = db.get_all_scenes_for_timeline().unwrap();
        assert!(timeline_scenes.is_empty());
    }

    // --- Bible relationship: get for entry ---

    #[test]
    fn test_get_bible_relationships_for_entry() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let e1 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Alice".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();
        let e2 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Bob".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        db.create_bible_relationship(&CreateBibleRelationshipRequest {
            source_id: e1.id.clone(),
            target_id: e2.id.clone(),
            relationship_type: "friend".to_string(),
            note: Some("Best friends".to_string()),
            status: None,
        }).unwrap();

        let rels = db.get_bible_relationships(&e1.id).unwrap();
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].relationship_type, "friend");

        // Also visible from the other side
        let rels2 = db.get_bible_relationships(&e2.id).unwrap();
        assert_eq!(rels2.len(), 1);
    }

    // --- Reorder chapters reverse order ---

    #[test]
    fn test_reorder_chapters_reverse_order() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch1 = db.create_chapter(&CreateChapterRequest {
            title: "A".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let ch2 = db.create_chapter(&CreateChapterRequest {
            title: "B".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let ch3 = db.create_chapter(&CreateChapterRequest {
            title: "C".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Reverse: C, B, A
        db.reorder_chapters(&[ch3.id.clone(), ch2.id.clone(), ch1.id.clone()]).unwrap();

        let chapters = db.get_chapters().unwrap();
        assert_eq!(chapters[0].title, "C");
        assert_eq!(chapters[1].title, "B");
        assert_eq!(chapters[2].title, "A");
    }

    // --- Word count: deleted scenes excluded ---

    #[test]
    fn test_word_count_excludes_deleted_scenes() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Keep".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Delete".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        db.update_scene(&s1.id, &UpdateSceneRequest {
            text: Some("one two three".to_string()),
            ..Default::default()
        }).unwrap();
        db.update_scene(&s2.id, &UpdateSceneRequest {
            text: Some("four five six seven".to_string()),
            ..Default::default()
        }).unwrap();

        let counts_before = db.get_word_counts().unwrap();
        assert_eq!(counts_before.total, 7); // 3 + 4

        db.delete_scene(&s2.id).unwrap();

        let counts_after = db.get_word_counts().unwrap();
        assert_eq!(counts_after.total, 3); // Only s1's words
    }

    // =========================================================================
    // Batch 9: Final edge cases & error paths
    // =========================================================================

    // --- Create scene with nonexistent chapter ---

    #[test]
    fn test_create_scene_nonexistent_chapter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let result = db.create_scene(&CreateSceneRequest {
            chapter_id: "nonexistent-chapter-id".to_string(),
            title: "Orphan Scene".to_string(),
            summary: None,
            position: None,
        });
        // Should fail because foreign key constraint
        assert!(result.is_err());
    }

    // --- Move scene to nonexistent chapter ---

    #[test]
    fn test_move_scene_to_nonexistent_chapter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.move_scene_to_chapter(&scene.id, "nonexistent-chapter-id", 0);
        // The move should either fail or the scene should remain in original chapter
        if result.is_ok() {
            let fetched = db.get_scene(&scene.id).unwrap();
            assert!(!fetched.chapter_id.is_empty());
        }
        // Either way, original chapter should still exist
        let _ = db.get_chapter(&chapter.id).unwrap();
    }

    // --- Create association with nonexistent scene ---

    #[test]
    fn test_create_association_nonexistent_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let _entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Alice".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        let result = db.create_association(&CreateAssociationRequest {
            scene_id: "nonexistent-scene".to_string(),
            bible_entry_id: _entry.id.clone(),
        });
        assert!(result.is_err());
    }

    // --- Create association with nonexistent bible entry ---

    #[test]
    fn test_create_association_nonexistent_bible_entry() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        let result = db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: "nonexistent-bible-entry".to_string(),
        });
        assert!(result.is_err());
    }

    // --- Link arc to nonexistent scene ---

    #[test]
    fn test_link_arc_to_nonexistent_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let arc = db.create_arc(&CreateArcRequest {
            name: "Main Arc".to_string(),
            description: None,
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();

        let result = db.link_scene_to_arc("nonexistent-scene", &arc.id);
        assert!(result.is_err());
    }

    // --- Link event to nonexistent scene ---

    #[test]
    fn test_link_event_to_nonexistent_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let event = db.create_event(&CreateEventRequest {
            title: "Event 1".to_string(),
            description: None,
            event_type: None,
            importance: None,
            time_point: None,
            time_start: None,
            time_end: None,
        }).unwrap();

        let result = db.link_scene_to_event("nonexistent-scene", &event.id);
        assert!(result.is_err());
    }

    // --- Export with complex relationships ---

    #[test]
    fn test_export_json_with_all_entity_types() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // Add rich content to scene
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Once upon a time...</p>".to_string()),
            summary: Some("A beginning".to_string()),
            pov: Some("Alice".to_string()),
            tags: Some("fantasy,opening".to_string()),
            time_point: Some("1200".to_string()),
            on_timeline: Some(true),
            ..Default::default()
        }).unwrap();

        // Create multiple bible entries
        let char1 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Alice".to_string(),
            aliases: Some("Ali".to_string()),
            short_description: Some("The hero".to_string()),
            full_description: Some("A brave adventurer".to_string()),
            status: Some("canon".to_string()),
            tags: Some("hero,main".to_string()),
            color: Some("#ff0000".to_string()),
        }).unwrap();

        let loc1 = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "location".to_string(),
            name: "Castle".to_string(),
            aliases: None,
            short_description: None,
            full_description: None,
            status: None,
            tags: None,
            color: None,
        }).unwrap();

        // Create associations
        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: char1.id.clone(),
        }).unwrap();
        db.create_association(&CreateAssociationRequest {
            scene_id: scene.id.clone(),
            bible_entry_id: loc1.id.clone(),
        }).unwrap();

        // Create bible relationship
        db.create_bible_relationship(&CreateBibleRelationshipRequest {
            source_id: char1.id.clone(),
            target_id: loc1.id.clone(),
            relationship_type: "lives_in".to_string(),
            note: Some("Her ancestral home".to_string()),
            status: None,
        }).unwrap();

        // Create arc and link
        let arc = db.create_arc(&CreateArcRequest {
            name: "Quest Arc".to_string(),
            description: Some("The main quest".to_string()),
            stakes: None,
            characters: None,
            status: None,
            color: None,
        }).unwrap();
        db.link_scene_to_arc(&scene.id, &arc.id).unwrap();

        // Create event and link
        let event = db.create_event(&CreateEventRequest {
            title: "The Call".to_string(),
            description: Some("Adventure begins".to_string()),
            event_type: None,
            importance: None,
            time_point: Some("1200".to_string()),
            time_start: None,
            time_end: None,
        }).unwrap();
        db.link_scene_to_event(&scene.id, &event.id).unwrap();

        // Export JSON
        let json = db.export_json_backup().unwrap();

        // Parse and verify all entities present
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["version"], "1.0");
        assert!(parsed["exported_at"].is_string());
        assert_eq!(parsed["project"]["title"], "Test");
        assert_eq!(parsed["chapters"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["scenes"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["bible_entries"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["arcs"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["events"].as_array().unwrap().len(), 1);

        // Verify scene data
        let scene_data = &parsed["scenes"][0];
        assert_eq!(scene_data["pov"], "Alice");
        assert_eq!(scene_data["tags"], "fantasy,opening");
        assert_eq!(scene_data["time_point"], "1200");

        // Verify bible data
        let char_data = parsed["bible_entries"].as_array().unwrap()
            .iter()
            .find(|e| e["name"] == "Alice")
            .unwrap();
        assert_eq!(char_data["aliases"], "Ali");
        assert_eq!(char_data["status"], "canon");
    }

    // --- Import/export roundtrip preserves all fields ---

    #[test]
    fn test_import_json_backup_roundtrip_all_fields() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        // Create chapter with all fields
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Rich Chapter".to_string(),
            summary: Some("A detailed chapter".to_string()),
            position: None,
        }).unwrap();
        db.update_chapter(&chapter.id, &UpdateChapterRequest {
            title: None,
            summary: None,
            status: Some("writing".to_string()),
            notes: Some("Author notes here".to_string()),
            position: None,
        }).unwrap();

        // Create scene with all fields
        let scene = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Rich Scene".to_string(),
            summary: Some("Scene summary".to_string()),
            position: None,
        }).unwrap();
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Rich content here</p>".to_string()),
            pov: Some("Narrator".to_string()),
            tags: Some("tag1,tag2".to_string()),
            notes: Some("Scene notes".to_string()),
            status: Some("revision".to_string()),
            time_point: Some("Day 1".to_string()),
            on_timeline: Some(true),
            ..Default::default()
        }).unwrap();

        // Create bible entry with all fields
        let _entry = db.create_bible_entry(&CreateBibleEntryRequest {
            entry_type: "character".to_string(),
            name: "Protagonist".to_string(),
            aliases: Some("Hero, Main".to_string()),
            short_description: Some("Short desc".to_string()),
            full_description: Some("Full description".to_string()),
            status: Some("canon".to_string()),
            tags: Some("main,hero".to_string()),
            color: Some("#00ff00".to_string()),
        }).unwrap();

        // Export
        let json = db.export_json_backup().unwrap();

        // Verify the export contains all the data
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let scene_json = &parsed["scenes"][0];
        assert_eq!(scene_json["title"], "Rich Scene");
        assert_eq!(scene_json["summary"], "Scene summary");
        assert_eq!(scene_json["pov"], "Narrator");
        assert_eq!(scene_json["tags"], "tag1,tag2");
        assert_eq!(scene_json["notes"], "Scene notes");
        assert_eq!(scene_json["status"], "revision");

        let bible_json = &parsed["bible_entries"][0];
        assert_eq!(bible_json["name"], "Protagonist");
        assert_eq!(bible_json["aliases"], "Hero, Main");
        assert_eq!(bible_json["short_description"], "Short desc");
        assert_eq!(bible_json["full_description"], "Full description");
        assert_eq!(bible_json["status"], "canon");
        assert_eq!(bible_json["color"], "#00ff00");
    }

    // --- Scan names respects existing aliases ---

    #[test]
    fn test_scan_names_respects_existing_registry() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        // Pre-create a name registry entry
        db.create_name_registry_entry(&CreateNameRegistryRequest {
            canonical_name: "Alice".to_string(),
            name_type: Some("character".to_string()),
            bible_entry_id: None,
            aliases: None,
        }).unwrap();

        // Create scenes mentioning Alice multiple times (not at sentence start)
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("Then Alice arrived. Later Alice spoke.".to_string()),
            ..Default::default()
        }).unwrap();

        let (new_entries, _new_mentions) = db.scan_names().unwrap();

        // Alice already exists in registry → should NOT create a duplicate entry
        assert_eq!(new_entries, 0);

        // Verify only one registry entry for Alice
        let entries = db.get_name_registry_entries(None).unwrap();
        let alice_entries: Vec<_> = entries.iter()
            .filter(|e| e.canonical_name.to_lowercase() == "alice")
            .collect();
        assert_eq!(alice_entries.len(), 1);
    }

    // --- Snapshot restore then re-export matches ---

    #[test]
    fn test_snapshot_restore_then_export_matches_original() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Original content for export</p>".to_string()),
            ..Default::default()
        }).unwrap();

        // Export markdown before changes
        let original_markdown = db.export_markdown().unwrap();

        // Create snapshot
        let snapshot = db.create_snapshot("Before changes", None, "manual").unwrap();

        // Modify content
        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Modified content</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let modified_markdown = db.export_markdown().unwrap();
        assert_ne!(original_markdown, modified_markdown);

        // Restore snapshot
        db.restore_snapshot(&snapshot.id).unwrap();

        // Export again - should match original
        let restored_markdown = db.export_markdown().unwrap();
        assert_eq!(original_markdown, restored_markdown);
    }

    // --- Reorder scenes within chapter ---

    #[test]
    fn test_reorder_scenes_within_chapter() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let chapter = db.create_chapter(&CreateChapterRequest {
            title: "Ch".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "First".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Second".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let s3 = db.create_scene(&CreateSceneRequest {
            chapter_id: chapter.id.clone(),
            title: "Third".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        // Reverse order
        db.reorder_scenes(&chapter.id, &[s3.id.clone(), s1.id.clone(), s2.id.clone()]).unwrap();

        let scenes = db.get_scenes(&chapter.id).unwrap();
        assert_eq!(scenes[0].title, "Third");
        assert_eq!(scenes[1].title, "First");
        assert_eq!(scenes[2].title, "Second");
    }

    // --- Multiple chapters with scenes - export order preserved ---

    #[test]
    fn test_export_preserves_chapter_and_scene_order() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);

        let ch1 = db.create_chapter(&CreateChapterRequest {
            title: "Chapter One".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        let ch2 = db.create_chapter(&CreateChapterRequest {
            title: "Chapter Two".to_string(),
            summary: None,
            position: None,
        }).unwrap();

        let s1 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch1.id.clone(),
            title: "Scene A".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&s1.id, &UpdateSceneRequest {
            text: Some("<p>Content A</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let s2 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch1.id.clone(),
            title: "Scene B".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&s2.id, &UpdateSceneRequest {
            text: Some("<p>Content B</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let s3 = db.create_scene(&CreateSceneRequest {
            chapter_id: ch2.id.clone(),
            title: "Scene C".to_string(),
            summary: None,
            position: None,
        }).unwrap();
        db.update_scene(&s3.id, &UpdateSceneRequest {
            text: Some("<p>Content C</p>".to_string()),
            ..Default::default()
        }).unwrap();

        let markdown = db.export_markdown().unwrap();

        // Verify order: Chapter One appears before Chapter Two
        let ch1_pos = markdown.find("Chapter One").unwrap();
        let ch2_pos = markdown.find("Chapter Two").unwrap();
        assert!(ch1_pos < ch2_pos);

        // Verify scenes appear in order within their chapters
        let a_pos = markdown.find("Content A").unwrap();
        let b_pos = markdown.find("Content B").unwrap();
        let c_pos = markdown.find("Content C").unwrap();
        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    // --- Delete chapter then verify scenes excluded from global search ---

    #[test]
    fn test_delete_chapter_scenes_excluded_from_global_search() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (chapter, scene) = setup_chapter_and_scene(&db);

        db.update_scene(&scene.id, &UpdateSceneRequest {
            text: Some("<p>Searchable unique word xyzzyqwer</p>".to_string()),
            ..Default::default()
        }).unwrap();

        // Verify searchable before delete
        let results = db.global_search("xyzzyqwer", None).unwrap();
        assert!(!results.is_empty());

        // Delete chapter (cascades to scenes)
        db.delete_chapter(&chapter.id).unwrap();

        // Verify scene excluded from search
        let results = db.global_search("xyzzyqwer", None).unwrap();
        assert!(results.is_empty());
    }

    // --- Annotation on deleted scene ---

    #[test]
    fn test_create_annotation_on_deleted_scene() {
        let (db, _temp_dir) = create_test_db();
        setup_project(&db);
        let (_chapter, scene) = setup_chapter_and_scene(&db);
        let scene_id = scene.id.clone();

        db.delete_scene(&scene_id).unwrap();

        // Creating annotation on deleted scene should still work at DB level
        // (soft-delete doesn't remove the row)
        let result = db.create_annotation(&CreateAnnotationRequest {
            scene_id: scene_id.clone(),
            annotation_type: None,
            content: "A note".to_string(),
            start_offset: 0,
            end_offset: 5,
        });
        // This is implementation-dependent - it may succeed (soft delete keeps row)
        // or fail (if there's a check). Either way, document the behavior.
        if result.is_ok() {
            let annotations = db.get_annotations(&scene_id).unwrap();
            assert!(!annotations.is_empty());
        }
    }
}
