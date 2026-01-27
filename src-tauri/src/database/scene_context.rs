//! Scene Context ("Previously On...").
//!
//! Provides immediate narrative context when selecting a scene:
//! previous scenes in the same chapter, present characters,
//! nearby issues, TODOs, and last writing session info.

use crate::models::{
    LastSessionInfo, NearbyIssue, PresentCharacter, PreviousSceneSummary, SceneContext,
};
use rusqlite::params;

use super::Database;

impl Database {
    /// Returns the narrative context for a scene.
    pub fn get_scene_context(&self, scene_id: &str) -> Result<SceneContext, String> {
        let previous_scenes = self.sc_previous_scenes(scene_id)?;
        let present_characters = self.sc_present_characters(scene_id)?;
        let nearby_issues = self.sc_nearby_issues(scene_id)?;
        let todos = self.sc_todos(scene_id)?;
        let last_session = self.sc_last_session()?;

        Ok(SceneContext {
            scene_id: scene_id.to_string(),
            previous_scenes,
            present_characters,
            nearby_issues,
            todos,
            last_session,
        })
    }

    /// Up to 5 previous scenes in the same chapter.
    fn sc_previous_scenes(&self, scene_id: &str) -> Result<Vec<PreviousSceneSummary>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s2.id, s2.title, s2.summary, s2.pov, s2.word_count
                 FROM scenes s2
                 JOIN scenes s1 ON s2.chapter_id = s1.chapter_id
                 WHERE s1.id = ?1
                   AND s2.id != ?1
                   AND s2.position < s1.position
                   AND s2.deleted_at IS NULL
                 ORDER BY s2.position DESC
                 LIMIT 5",
            )
            .map_err(|e| e.to_string())?;

        let results: Vec<PreviousSceneSummary> = stmt
            .query_map(params![scene_id], |row| {
                Ok(PreviousSceneSummary {
                    scene_id: row.get(0)?,
                    title: row.get(1)?,
                    summary: row.get(2)?,
                    pov: row.get(3)?,
                    word_count: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        // Reverse so they're in manuscript order (earliest first)
        Ok(results.into_iter().rev().collect())
    }

    /// Characters and other bible entries present in this scene.
    fn sc_present_characters(&self, scene_id: &str) -> Result<Vec<PresentCharacter>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT be.id, be.name, be.short_description, be.entry_type
                 FROM canonical_associations ca
                 JOIN bible_entries be ON ca.bible_entry_id = be.id
                 WHERE ca.scene_id = ?1
                   AND be.deleted_at IS NULL
                 ORDER BY be.entry_type, be.name",
            )
            .map_err(|e| e.to_string())?;

        let results = stmt
            .query_map(params![scene_id], |row| {
                Ok(PresentCharacter {
                    bible_entry_id: row.get(0)?,
                    name: row.get(1)?,
                    short_description: row.get(2)?,
                    entry_type: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(results)
    }

    /// Open issues linked to this scene or its 2 adjacent scenes.
    fn sc_nearby_issues(&self, scene_id: &str) -> Result<Vec<NearbyIssue>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT i.id, i.title, i.severity, i.status, isc.scene_id
                 FROM issues i
                 JOIN issue_scenes isc ON i.id = isc.issue_id
                 WHERE i.status = 'open'
                   AND isc.scene_id IN (
                     -- This scene
                     ?1,
                     -- Previous scene in same chapter
                     (SELECT s2.id FROM scenes s2
                      JOIN scenes s1 ON s2.chapter_id = s1.chapter_id
                      WHERE s1.id = ?1 AND s2.position < s1.position
                        AND s2.deleted_at IS NULL
                      ORDER BY s2.position DESC LIMIT 1),
                     -- Next scene in same chapter
                     (SELECT s3.id FROM scenes s3
                      JOIN scenes s1 ON s3.chapter_id = s1.chapter_id
                      WHERE s1.id = ?1 AND s3.position > s1.position
                        AND s3.deleted_at IS NULL
                      ORDER BY s3.position ASC LIMIT 1)
                   )
                 ORDER BY i.severity DESC, i.created_at",
            )
            .map_err(|e| e.to_string())?;

        let results = stmt
            .query_map(params![scene_id], |row| {
                Ok(NearbyIssue {
                    issue_id: row.get(0)?,
                    title: row.get(1)?,
                    severity: row.get(2)?,
                    status: row.get(3)?,
                    linked_scene_id: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(results)
    }

    /// Todos from the scene.
    fn sc_todos(&self, scene_id: &str) -> Result<Vec<String>, String> {
        let todos: Option<String> = self
            .conn
            .query_row(
                "SELECT todos FROM scenes WHERE id = ?1 AND deleted_at IS NULL",
                params![scene_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        Ok(todos
            .map(|t| {
                t.lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Most recent writing session.
    fn sc_last_session(&self) -> Result<Option<LastSessionInfo>, String> {
        self.conn
            .query_row(
                "SELECT date, words_end - words_start, duration_minutes
                 FROM writing_sessions
                 ORDER BY date DESC
                 LIMIT 1",
                [],
                |row| {
                    Ok(LastSessionInfo {
                        date: row.get(0)?,
                        words_written: row.get(1)?,
                        duration_minutes: row.get(2)?,
                    })
                },
            )
            .map_or_else(|_| Ok(None), |s| Ok(Some(s)))
    }
}
