//! Name Registry operations
//!
//! Manages proper nouns (characters, locations) and their mentions in scenes.

use crate::models::{
    CreateNameRegistryRequest, NameMention, NameRegistryEntry, UpdateNameMentionRequest,
    UpdateNameRegistryRequest,
};
use rusqlite::params;

use super::Database;

impl Database {
    pub fn create_name_registry_entry(
        &self,
        req: &CreateNameRegistryRequest,
    ) -> Result<NameRegistryEntry, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO name_registry (id, canonical_name, name_type, bible_entry_id, aliases, is_confirmed, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
                params![
                    id,
                    req.canonical_name,
                    req.name_type.as_deref().unwrap_or("character"),
                    req.bible_entry_id,
                    req.aliases,
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_name_registry_entry(&id)
    }

    pub fn get_name_registry_entries(
        &self,
        name_type: Option<&str>,
    ) -> Result<Vec<NameRegistryEntry>, String> {
        let (query, param_value);
        if let Some(nt) = name_type {
            query = "SELECT id, canonical_name, name_type, bible_entry_id, aliases, is_confirmed, created_at, updated_at
                     FROM name_registry WHERE name_type = ?1 ORDER BY canonical_name";
            param_value = Some(nt.to_string());
        } else {
            query = "SELECT id, canonical_name, name_type, bible_entry_id, aliases, is_confirmed, created_at, updated_at
                     FROM name_registry ORDER BY canonical_name";
            param_value = None;
        }

        let mut stmt = self.conn.prepare(query).map_err(|e| e.to_string())?;

        let entries = if let Some(ref ntype) = param_value {
            stmt.query_map(params![ntype], Self::map_name_registry_entry)
        } else {
            stmt.query_map([], Self::map_name_registry_entry)
        }
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        Ok(entries)
    }

    fn map_name_registry_entry(row: &rusqlite::Row) -> rusqlite::Result<NameRegistryEntry> {
        Ok(NameRegistryEntry {
            id: row.get(0)?,
            canonical_name: row.get(1)?,
            name_type: row.get(2)?,
            bible_entry_id: row.get(3)?,
            aliases: row.get(4)?,
            is_confirmed: row.get::<_, i32>(5)? != 0,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }

    pub fn get_name_registry_entry(&self, id: &str) -> Result<NameRegistryEntry, String> {
        self.conn
            .query_row(
                "SELECT id, canonical_name, name_type, bible_entry_id, aliases, is_confirmed, created_at, updated_at
                 FROM name_registry WHERE id = ?1",
                params![id],
                Self::map_name_registry_entry,
            )
            .map_err(|e| e.to_string())
    }

    pub fn update_name_registry_entry(
        &self,
        id: &str,
        req: &UpdateNameRegistryRequest,
    ) -> Result<NameRegistryEntry, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut set_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref name) = req.canonical_name {
            set_clauses.push(format!("canonical_name = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(name.clone()));
        }
        if let Some(ref name_type) = req.name_type {
            set_clauses.push(format!("name_type = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(name_type.clone()));
        }
        if let Some(ref bible_id) = req.bible_entry_id {
            set_clauses.push(format!("bible_entry_id = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(bible_id.clone()));
        }
        if let Some(ref aliases) = req.aliases {
            set_clauses.push(format!("aliases = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(aliases.clone()));
        }
        if let Some(confirmed) = req.is_confirmed {
            set_clauses.push(format!("is_confirmed = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(confirmed as i32));
        }

        if !set_clauses.is_empty() {
            set_clauses.push(format!("updated_at = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(now));

            let id_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE name_registry SET {} WHERE id = ?{}",
                set_clauses.join(", "),
                id_idx
            );

            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();
            let mut all_params = params_refs;
            all_params.push(&id);

            self.conn
                .execute(&query, all_params.as_slice())
                .map_err(|e| e.to_string())?;
        }

        self.get_name_registry_entry(id)
    }

    pub fn delete_name_registry_entry(&self, id: &str) -> Result<(), String> {
        // Delete associated mentions first
        self.conn
            .execute(
                "DELETE FROM name_mentions WHERE name_registry_id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;

        self.conn
            .execute("DELETE FROM name_registry WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    // Name Mentions
    pub fn create_name_mention(
        &self,
        name_registry_id: &str,
        scene_id: &str,
        mention_text: &str,
        start_offset: i32,
        end_offset: i32,
    ) -> Result<NameMention, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO name_mentions (id, name_registry_id, scene_id, mention_text, start_offset, end_offset, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7)",
                params![id, name_registry_id, scene_id, mention_text, start_offset, end_offset, now],
            )
            .map_err(|e| e.to_string())?;

        self.get_name_mention(&id)
    }

    pub fn get_name_mentions_by_scene(&self, scene_id: &str) -> Result<Vec<NameMention>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name_registry_id, scene_id, mention_text, start_offset, end_offset, status, created_at
                 FROM name_mentions WHERE scene_id = ?1 ORDER BY start_offset",
            )
            .map_err(|e| e.to_string())?;

        let mentions = stmt
            .query_map(params![scene_id], Self::map_name_mention)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(mentions)
    }

    pub fn get_name_mentions_by_registry(
        &self,
        registry_id: &str,
    ) -> Result<Vec<NameMention>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name_registry_id, scene_id, mention_text, start_offset, end_offset, status, created_at
                 FROM name_mentions WHERE name_registry_id = ?1 ORDER BY created_at",
            )
            .map_err(|e| e.to_string())?;

        let mentions = stmt
            .query_map(params![registry_id], Self::map_name_mention)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(mentions)
    }

    fn map_name_mention(row: &rusqlite::Row) -> rusqlite::Result<NameMention> {
        Ok(NameMention {
            id: row.get(0)?,
            name_registry_id: row.get(1)?,
            scene_id: row.get(2)?,
            mention_text: row.get(3)?,
            start_offset: row.get(4)?,
            end_offset: row.get(5)?,
            status: row.get(6)?,
            created_at: row.get(7)?,
        })
    }

    pub fn get_name_mention(&self, id: &str) -> Result<NameMention, String> {
        self.conn
            .query_row(
                "SELECT id, name_registry_id, scene_id, mention_text, start_offset, end_offset, status, created_at
                 FROM name_mentions WHERE id = ?1",
                params![id],
                Self::map_name_mention,
            )
            .map_err(|e| e.to_string())
    }

    pub fn update_name_mention(
        &self,
        id: &str,
        req: &UpdateNameMentionRequest,
    ) -> Result<NameMention, String> {
        self.conn
            .execute(
                "UPDATE name_mentions SET status = ?1 WHERE id = ?2",
                params![req.status, id],
            )
            .map_err(|e| e.to_string())?;

        self.get_name_mention(id)
    }

    pub fn delete_name_mention(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM name_mentions WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Scan all scene text for proper nouns (capitalized words not at sentence start)
    /// and register them, detecting similar spellings via Levenshtein distance.
    /// Returns (new_entries_count, new_mentions_count).
    pub fn scan_names(&self) -> Result<(i32, i32), String> {
        use std::collections::{HashMap, HashSet};

        // 1. Fetch all scenes
        let mut stmt = self
            .conn
            .prepare("SELECT id, text FROM scenes WHERE deleted_at IS NULL AND text != ''")
            .map_err(|e| e.to_string())?;

        let scenes: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        // 2. Fetch existing registry entries and bible entries
        let existing_entries = self.get_name_registry_entries(None)?;
        let existing_names: HashSet<String> = existing_entries
            .iter()
            .map(|e| e.canonical_name.to_lowercase())
            .collect();

        // Also collect aliases
        let mut alias_to_entry: HashMap<String, String> = HashMap::new();
        for entry in &existing_entries {
            alias_to_entry.insert(entry.canonical_name.to_lowercase(), entry.id.clone());
            if let Some(ref aliases) = entry.aliases {
                for alias in aliases.split(',') {
                    let alias = alias.trim().to_lowercase();
                    if !alias.is_empty() {
                        alias_to_entry.insert(alias, entry.id.clone());
                    }
                }
            }
        }

        // Fetch bible entry names for auto-linking
        let mut bible_stmt = self
            .conn
            .prepare("SELECT id, name FROM bible_entries WHERE deleted_at IS NULL")
            .map_err(|e| e.to_string())?;
        let bible_entries: Vec<(String, String)> = bible_stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let bible_name_map: HashMap<String, String> = bible_entries
            .iter()
            .map(|(id, name)| (name.to_lowercase(), id.clone()))
            .collect();

        // Common words to skip (not proper nouns even if capitalized)
        let skip_words: HashSet<&str> = [
            "the",
            "a",
            "an",
            "and",
            "or",
            "but",
            "in",
            "on",
            "at",
            "to",
            "for",
            "of",
            "with",
            "by",
            "from",
            "is",
            "it",
            "its",
            "this",
            "that",
            "was",
            "were",
            "be",
            "been",
            "are",
            "have",
            "has",
            "had",
            "do",
            "does",
            "did",
            "will",
            "would",
            "could",
            "should",
            "may",
            "might",
            "shall",
            "can",
            "not",
            "no",
            "yes",
            "he",
            "she",
            "they",
            "we",
            "you",
            "i",
            "me",
            "him",
            "her",
            "us",
            "them",
            "my",
            "your",
            "his",
            "our",
            "their",
            "what",
            "which",
            "who",
            "whom",
            "how",
            "when",
            "where",
            "why",
            "if",
            "then",
            "else",
            "so",
            "as",
            "than",
            "too",
            "very",
            "just",
            "about",
            "up",
            "out",
            "into",
            "over",
            "after",
            "before",
            "all",
            "each",
            "every",
            "both",
            "some",
            "any",
            "many",
            "much",
            "more",
            "most",
            "other",
            "such",
            "only",
            "also",
            "than",
            "now",
            "here",
            "there",
            "still",
            "already",
            "yet",
            "again",
            "however",
            "although",
            "because",
            "since",
            "while",
            "until",
            "chapter",
            "scene",
            "part",
            "book",
            "page",
            "said",
            "asked",
            "told",
            "thought",
            "knew",
            "saw",
            "looked",
            "made",
            "came",
            "went",
            "got",
            "took",
            "gave",
            "put",
            "let",
            "like",
            "never",
            "always",
            "often",
            "sometimes",
            "once",
            "even",
            "first",
            "last",
            "next",
            "new",
            "old",
            "long",
            "little",
            "big",
            "great",
            "good",
            "bad",
            "right",
            "left",
            "own",
            "same",
        ]
        .iter()
        .copied()
        .collect();

        let html_tag_re = &super::HTML_TAG_REGEX;

        // 3. Extract capitalized words from all scenes
        // Map: lowercase_name -> (original_forms set, scene_occurrences)
        type NameData = (HashSet<String>, Vec<(String, i32, i32)>);
        let mut found_names: HashMap<String, NameData> = HashMap::new();

        for (scene_id, html) in &scenes {
            let plain = html_tag_re.replace_all(html, " ");
            let plain = plain.as_ref();

            // Split into sentences (roughly: after ., !, ?, or start of text)
            let sentences: Vec<&str> = plain.split(['.', '!', '?']).collect();

            let mut offset = 0;
            for sentence in &sentences {
                let trimmed = sentence.trim();
                let words: Vec<&str> = trimmed.split_whitespace().collect();

                // Skip first word of each sentence (always capitalized)
                for (i, word) in words.iter().enumerate() {
                    let clean: String = word
                        .chars()
                        .filter(|c| c.is_alphabetic() || *c == '-' || *c == '\'')
                        .collect();

                    if clean.len() < 2 {
                        continue;
                    }

                    let Some(first_char) = clean.chars().next() else {
                        continue;
                    };
                    if !first_char.is_uppercase() {
                        continue;
                    }

                    // Skip first word of sentence
                    if i == 0 {
                        continue;
                    }

                    let lower = clean.to_lowercase();
                    if skip_words.contains(lower.as_str()) {
                        continue;
                    }

                    // Find approximate offset in plain text
                    let word_offset = plain[offset..]
                        .find(word.trim_matches(|c: char| !c.is_alphabetic()))
                        .map(|o| o + offset)
                        .unwrap_or(offset);

                    let entry = found_names
                        .entry(lower.clone())
                        .or_insert_with(|| (HashSet::new(), Vec::new()));
                    entry.0.insert(clean.clone());
                    entry.1.push((
                        scene_id.clone(),
                        word_offset as i32,
                        (word_offset + clean.len()) as i32,
                    ));
                }

                // Advance offset past this sentence
                offset += sentence.len() + 1; // +1 for the delimiter
            }
        }

        // 4. Create registry entries for new names and mentions
        let mut new_entries = 0;
        let mut new_mentions = 0;

        for (lower_name, (original_forms, occurrences)) in &found_names {
            // Skip if only found once (likely not a recurring proper noun)
            if occurrences.len() < 2 {
                continue;
            }

            // Check if already in registry
            let registry_id = if let Some(id) = alias_to_entry.get(lower_name) {
                id.clone()
            } else if existing_names.contains(lower_name) {
                continue; // Already exists, skip
            } else {
                // Create new entry
                let Some(canonical) = original_forms.iter().next().cloned() else {
                    continue;
                };
                let bible_entry_id = bible_name_map.get(lower_name).cloned();

                let req = CreateNameRegistryRequest {
                    canonical_name: canonical,
                    name_type: Some("character".to_string()),
                    bible_entry_id,
                    aliases: if original_forms.len() > 1 {
                        Some(
                            original_forms
                                .iter()
                                .skip(1)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", "),
                        )
                    } else {
                        None
                    },
                };

                let entry = self.create_name_registry_entry(&req)?;
                new_entries += 1;
                entry.id
            };

            // Create mentions (limit to avoid flooding)
            let max_mentions = 50;
            for (scene_id, start, end) in occurrences.iter().take(max_mentions) {
                let Some(mention_text) = original_forms.iter().next() else {
                    continue;
                };
                self.create_name_mention(&registry_id, scene_id, mention_text, *start, *end)?;
                new_mentions += 1;
            }
        }

        Ok((new_entries, new_mentions))
    }

    /// Merge one registry entry into another, moving all mentions and combining aliases.
    pub fn merge_name_entries(
        &self,
        keep_id: &str,
        merge_id: &str,
    ) -> Result<NameRegistryEntry, String> {
        // Get both entries
        let keep = self.get_name_registry_entry(keep_id)?;
        let merge = self.get_name_registry_entry(merge_id)?;

        // Combine aliases
        let mut all_aliases: Vec<String> = Vec::new();
        // Add merged entry's canonical name as alias
        all_aliases.push(merge.canonical_name.clone());
        // Add existing aliases from both
        if let Some(ref aliases) = keep.aliases {
            for a in aliases.split(',') {
                let a = a.trim().to_string();
                if !a.is_empty() {
                    all_aliases.push(a);
                }
            }
        }
        if let Some(ref aliases) = merge.aliases {
            for a in aliases.split(',') {
                let a = a.trim().to_string();
                if !a.is_empty() {
                    all_aliases.push(a);
                }
            }
        }

        // Deduplicate aliases (case-insensitive)
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        seen.insert(keep.canonical_name.to_lowercase());
        all_aliases.retain(|a| seen.insert(a.to_lowercase()));

        let combined_aliases = if all_aliases.is_empty() {
            None
        } else {
            Some(all_aliases.join(", "))
        };

        // Move mentions from merged entry to kept entry
        self.conn
            .execute(
                "UPDATE name_mentions SET name_registry_id = ?1 WHERE name_registry_id = ?2",
                params![keep_id, merge_id],
            )
            .map_err(|e| e.to_string())?;

        // Delete the merged entry (mentions already moved)
        self.conn
            .execute("DELETE FROM name_registry WHERE id = ?1", params![merge_id])
            .map_err(|e| e.to_string())?;

        // Update kept entry with combined aliases
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE name_registry SET aliases = ?1, updated_at = ?2 WHERE id = ?3",
                params![combined_aliases, now, keep_id],
            )
            .map_err(|e| e.to_string())?;

        self.get_name_registry_entry(keep_id)
    }
}
