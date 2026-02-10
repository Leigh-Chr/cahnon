<!--
  Story bible (knowledge base) browser and editor.

  Features:
  - Browse entries by type (character, location, object, faction, concept, glossary)
  - Create and edit bible entries with rich fields
  - Define relationships between entries (parent/child, spouse, ally, etc.)
  - Custom fields as JSON
  - Color coding for visual identification
  - Aliases for alternate names
  - Status tracking (active, minor, mentioned, deceased)
-->
<script lang="ts">
	import { slide } from 'svelte/transition';

	import {
		type Arc,
		arcApi,
		associationApi,
		type BibleEntry,
		type BibleRelationshipWithEntry,
		eventApi,
		type Issue,
		issueApi,
		relationshipApi,
		type Scene,
		type TimelineEvent,
	} from '$lib/api';
	import { BIBLE_FIELD_TEMPLATES } from '$lib/bible-templates';

	import ImpactDialog from './ImpactDialog.svelte';

	// Impact dialog state
	let impactDialog = $state<{
		entityId: string;
		entityName: string;
	} | null>(null);
	import { appState } from '$lib/stores';
	import { showError } from '$lib/toast';
	import { bibleEntryTypes, bibleStatuses } from '$lib/utils';
	import { nativeConfirm } from '$lib/utils/native-dialog';

	import ContextMenu from './ui/ContextMenu.svelte';
	import ContextMenuItem from './ui/ContextMenuItem.svelte';
	import ContextMenuSeparator from './ui/ContextMenuSeparator.svelte';

	// Context menu for bible entries
	let entryContextMenu = $state<{ x: number; y: number; entryId: string } | null>(null);

	function handleEntryContextMenu(event: MouseEvent, entryId: string) {
		event.preventDefault();
		entryContextMenu = { x: event.clientX, y: event.clientY, entryId };
	}

	function closeEntryContextMenu() {
		entryContextMenu = null;
	}

	import RelationshipMap from './RelationshipMap.svelte';
	import { Button, EmptyState, FormActions, FormGroup, Icon, LoadingState } from './ui';

	let isCreating = $state(false);
	let bibleViewMode = $state<'detail' | 'graph'>('detail');
	let newEntryType = $state('character');
	let newEntryName = $state('');

	// Relationships
	let relationships = $state<BibleRelationshipWithEntry[]>([]);
	let linkedEvents = $state<TimelineEvent[]>([]);
	let linkedScenes = $state<Scene[]>([]);
	let linkedArcs = $state<Arc[]>([]);
	let linkedIssues = $state<Issue[]>([]);
	let loadRelatedError = $state(false);
	let isLoadingRelated = $state(false);
	let showRelationshipForm = $state(false);
	let newRelationshipTarget = $state('');
	let newRelationshipType = $state('related_to');

	// Relationship editing
	let editingRelationshipId = $state<string | null>(null);
	let editedRelType = $state('related_to');
	let editedRelNote = $state('');

	const entryColors = [
		'#ef4444', // red
		'#f97316', // orange
		'#eab308', // yellow
		'#22c55e', // green
		'#06b6d4', // cyan
		'#3b82f6', // blue
		'#6366f1', // indigo
		'#8b5cf6', // violet
		'#ec4899', // pink
	];

	const relationshipTypes = [
		// General
		{ value: 'related_to', label: 'Related to' },
		// Character ↔ Character
		{ value: 'parent_of', label: 'Parent of' },
		{ value: 'child_of', label: 'Child of' },
		{ value: 'sibling_of', label: 'Sibling of' },
		{ value: 'spouse_of', label: 'Spouse of' },
		{ value: 'friend_of', label: 'Friend of' },
		{ value: 'enemy_of', label: 'Enemy of' },
		{ value: 'mentor_of', label: 'Mentor of' },
		{ value: 'ally_of', label: 'Ally of' },
		{ value: 'knows', label: 'Knows' },
		{ value: 'killed', label: 'Killed' },
		{ value: 'saved', label: 'Saved' },
		// Character ↔ Location
		{ value: 'lives_in', label: 'Lives in' },
		{ value: 'born_in', label: 'Born in' },
		{ value: 'works_at', label: 'Works at' },
		// Character ↔ Faction
		{ value: 'member_of', label: 'Member of' },
		{ value: 'leader_of', label: 'Leader of' },
		{ value: 'founder_of', label: 'Founder of' },
		// Character ↔ Object
		{ value: 'owns', label: 'Owns' },
		{ value: 'created', label: 'Created' },
		{ value: 'seeks', label: 'Seeks' },
		// Location ↔ Location
		{ value: 'contains', label: 'Contains' },
		{ value: 'part_of', label: 'Part of' },
		{ value: 'near', label: 'Near' },
		{ value: 'located_in', label: 'Located in' },
	];

	// Use $derived for computed state instead of $effect
	let selectedEntry = $derived(
		appState.selectedBibleEntryId
			? appState.bibleEntries.find((e) => e.id === appState.selectedBibleEntryId) || null
			: null
	);

	// Side effect: load relationships and linked items when selection changes
	$effect(() => {
		if (appState.selectedBibleEntryId) {
			loadRelatedError = false;
			loadAllRelated(appState.selectedBibleEntryId);
		}
	});

	async function loadAllRelated(entryId: string) {
		isLoadingRelated = true;
		const results = await Promise.allSettled([
			loadRelationships(entryId),
			loadLinkedEvents(entryId),
			loadLinkedScenes(entryId),
			loadLinkedArcs(entryId),
			loadLinkedIssues(entryId),
		]);
		if (results.some((r) => r.status === 'rejected')) {
			loadRelatedError = true;
		}
		isLoadingRelated = false;
	}

	async function loadRelationships(entryId: string) {
		try {
			relationships = await relationshipApi.getByEntry(entryId);
		} catch (e) {
			console.error('Failed to load relationships:', e);
			relationships = [];
		}
	}

	async function loadLinkedEvents(entryId: string) {
		try {
			linkedEvents = await eventApi.getBibleEntryEvents(entryId);
		} catch (e) {
			console.error('Failed to load linked events:', e);
			linkedEvents = [];
		}
	}

	async function loadLinkedScenes(entryId: string) {
		try {
			linkedScenes = await associationApi.getByEntry(entryId);
		} catch (e) {
			console.error('Failed to load linked scenes:', e);
			linkedScenes = [];
		}
	}

	async function loadLinkedArcs(entryId: string) {
		try {
			linkedArcs = await arcApi.getCharacterArcs(entryId);
		} catch (e) {
			console.error('Failed to load linked arcs:', e);
			linkedArcs = [];
		}
	}

	async function loadLinkedIssues(entryId: string) {
		try {
			linkedIssues = await issueApi.getBibleEntryIssues(entryId);
		} catch (e) {
			console.error('Failed to load linked issues:', e);
			linkedIssues = [];
		}
	}

	function navigateToScene(sceneId: string) {
		const chapterId = appState.getChapterIdForScene(sceneId);
		if (chapterId) {
			appState.selectedChapterId = chapterId;
			appState.selectedSceneId = sceneId;
			appState.setViewMode('editor');
		}
	}

	async function createRelationship() {
		if (!selectedEntry || !newRelationshipTarget) return;
		try {
			await relationshipApi.create({
				source_id: selectedEntry.id,
				target_id: newRelationshipTarget,
				relationship_type: newRelationshipType,
			});
			await loadRelationships(selectedEntry.id);
			showRelationshipForm = false;
			newRelationshipTarget = '';
			newRelationshipType = 'related_to';
		} catch (e) {
			console.error('Failed to create relationship:', e);
			showError('Failed to create relationship');
		}
	}

	async function deleteRelationship(relationshipId: string) {
		if (!selectedEntry) return;
		if (!(await nativeConfirm('Remove this relationship?', 'Delete Relationship'))) return;
		try {
			await relationshipApi.delete(relationshipId);
			await loadRelationships(selectedEntry.id);
		} catch (e) {
			console.error('Failed to delete relationship:', e);
			showError('Failed to delete relationship');
		}
	}

	function startEditingRelationship(rel: BibleRelationshipWithEntry) {
		editingRelationshipId = rel.id;
		editedRelType = rel.relationship_type;
		editedRelNote = rel.note || '';
	}

	function cancelEditingRelationship() {
		editingRelationshipId = null;
	}

	async function saveRelationshipEdit() {
		if (!editingRelationshipId || !selectedEntry) return;
		try {
			await relationshipApi.update(editingRelationshipId, {
				relationship_type: editedRelType,
				note: editedRelNote.trim() || undefined,
			});
			await loadRelationships(selectedEntry.id);
			editingRelationshipId = null;
		} catch (e) {
			console.error('Failed to update relationship:', e);
			showError('Failed to update relationship');
		}
	}

	const relationshipLabelMap = new Map(relationshipTypes.map((t) => [t.value, t.label]));

	function getRelationshipLabel(type: string): string {
		return relationshipLabelMap.get(type) || type;
	}

	function selectEntry(entry: BibleEntry) {
		appState.selectedBibleEntryId = entry.id;
	}

	async function createEntry() {
		if (!newEntryName.trim()) return;

		try {
			const entry = await appState.createBibleEntry({
				entry_type: newEntryType,
				name: newEntryName.trim(),
			});

			// Pre-fill custom fields from template if available (Phase 6D)
			const template = BIBLE_FIELD_TEMPLATES[newEntryType];
			if (template && template.length > 0) {
				const templateFields: Record<string, string> = {};
				for (const field of template) {
					templateFields[field.key] = '';
				}
				await appState.updateBibleEntry(entry.id, {
					custom_fields: JSON.stringify(templateFields),
				});
			}

			newEntryName = '';
			isCreating = false;
		} catch (e) {
			console.error('Failed to create bible entry:', e);
			showError('Failed to create entry');
		}
	}

	async function updateEntry(field: string, value: string) {
		if (selectedEntry) {
			try {
				await appState.updateBibleEntry(selectedEntry.id, { [field]: value });
			} catch (e) {
				console.error('Failed to update bible entry:', e);
				showError('Failed to update entry');
			}
		}
	}

	function deleteEntry() {
		if (selectedEntry) {
			impactDialog = {
				entityId: selectedEntry.id,
				entityName: selectedEntry.name,
			};
		}
	}

	async function confirmDeleteEntry() {
		if (!impactDialog) return;
		try {
			await appState.deleteBibleEntry(impactDialog.entityId);
			appState.selectedBibleEntryId = null;
		} catch (e) {
			console.error('Failed to delete bible entry:', e);
			showError('Failed to delete entry');
		}
		impactDialog = null;
	}

	function getTypeInfo(type: string) {
		return bibleEntryTypes.find((t) => t.value === type) || { value: type, label: type, icon: '?' };
	}

	// Custom fields handling - mutable state that syncs from selectedEntry
	interface CustomField {
		name: string;
		value: string;
	}

	let customFields = $state<CustomField[]>([]);
	let lastSyncedEntryId: string | null = null; // Non-reactive to avoid effect loops

	// Sync custom fields when the selected entry changes (not when custom fields are edited)
	$effect(() => {
		const entryId = selectedEntry?.id ?? null;
		if (entryId !== lastSyncedEntryId) {
			lastSyncedEntryId = entryId;
			if (selectedEntry?.custom_fields) {
				try {
					const parsed = JSON.parse(selectedEntry.custom_fields);
					customFields = Object.entries(parsed).map(([name, value]) => ({
						name,
						value: String(value),
					}));
				} catch {
					customFields = [];
				}
			} else {
				customFields = [];
			}
		}
	});

	// Friendly labels for type-specific default fields
	const fieldLabels: Record<string, string> = {
		role: 'Role',
		voice_notes: 'Voice Notes',
		parent_location: 'Parent Location',
		faction_type: 'Type',
		members: 'Members',
		headquarters: 'Headquarters',
		pronunciation: 'Pronunciation',
		etymology: 'Etymology',
		language: 'Language',
	};

	function getFieldLabel(name: string): string {
		// Check template labels for current entry type first
		if (selectedEntry) {
			const template = BIBLE_FIELD_TEMPLATES[selectedEntry.entry_type];
			if (template) {
				const templateField = template.find((f) => f.key === name);
				if (templateField) return templateField.label;
			}
		}
		return fieldLabels[name] || name;
	}

	function isTemplateField(name: string): boolean {
		if (!selectedEntry) return false;
		const template = BIBLE_FIELD_TEMPLATES[selectedEntry.entry_type];
		if (!template) return false;
		return template.some((f) => f.key === name);
	}

	function getFieldType(name: string): 'text' | 'textarea' {
		if (!selectedEntry) return 'text';
		const template = BIBLE_FIELD_TEMPLATES[selectedEntry.entry_type];
		if (!template) return 'text';
		const field = template.find((f) => f.key === name);
		return field?.type || 'text';
	}

	function isKnownField(name: string): boolean {
		if (fieldLabels[name]) return true;
		return isTemplateField(name);
	}

	function addCustomField() {
		customFields = [...customFields, { name: '', value: '' }];
	}

	function removeCustomField(index: number) {
		customFields = customFields.filter((_, i) => i !== index);
		saveCustomFields();
	}

	function updateCustomFieldName(index: number, name: string) {
		customFields[index].name = name;
		saveCustomFields();
	}

	function updateCustomFieldValue(index: number, value: string) {
		customFields[index].value = value;
		saveCustomFields();
	}

	// CC2: Enhanced completion indicator with detailed breakdown
	interface CompletionDetails {
		total: number;
		breakdown: {
			name: boolean;
			description: boolean;
			aliases: boolean;
			tags: boolean;
			notes: boolean;
			customFields: string; // "X/Y filled"
		};
	}

	function computeCompletionDetails(entry: BibleEntry): CompletionDetails {
		let filled = 0;
		let total = 0;

		// Base fields: name is always filled (10%), plus 5 optional base fields at ~10% each
		// name always counts
		filled += 10;
		total += 10;

		// summary
		total += 10;
		if (entry.summary) filled += 10;

		// full_description
		total += 10;
		if (entry.full_description) filled += 10;

		// aliases
		total += 10;
		if (entry.aliases) filled += 10;

		// tags
		total += 10;
		if (entry.tags) filled += 10;

		// notes
		total += 10;
		if (entry.notes) filled += 10;

		// Custom fields: each filled custom field adds proportional %
		let customFieldCount = 0;
		let customFieldFilled = 0;
		if (entry.custom_fields) {
			try {
				const parsed = JSON.parse(entry.custom_fields);
				const entries = Object.entries(parsed);
				customFieldCount = entries.length;
				customFieldFilled = entries.filter(
					([, v]) => typeof v === 'string' && v.trim().length > 0
				).length;
			} catch {
				// ignore parse errors
			}
		}
		if (customFieldCount > 0) {
			// Custom fields share 20% of total
			total += 20;
			filled += Math.round((customFieldFilled / customFieldCount) * 20);
		}

		return {
			total: Math.min(100, Math.round((filled / total) * 100)),
			breakdown: {
				name: true, // always filled
				description: !!(entry.summary || entry.full_description),
				aliases: !!entry.aliases,
				tags: !!entry.tags,
				notes: !!entry.notes,
				customFields: customFieldCount > 0 ? `${customFieldFilled}/${customFieldCount}` : 'none',
			},
		};
	}

	// Backwards-compatible wrapper (used in stats computation)
	function computeCompletion(entry: BibleEntry): number {
		return computeCompletionDetails(entry).total;
	}

	// Average completion for stats
	let avgCompletion = $derived.by(() => {
		if (appState.bibleEntries.length === 0) return 0;
		const total = appState.bibleEntries.reduce((sum, e) => sum + computeCompletion(e), 0);
		return Math.round(total / appState.bibleEntries.length);
	});

	// CC2: Generate detailed tooltip text for completion
	function getCompletionTooltip(details: CompletionDetails): string {
		const { total, breakdown } = details;
		const check = (ok: boolean) => (ok ? '✓' : '○');
		return `${total}% complete
Name: ${check(breakdown.name)}
Description: ${check(breakdown.description)}
Aliases: ${check(breakdown.aliases)}
Tags: ${check(breakdown.tags)}
Notes: ${check(breakdown.notes)}
Custom Fields: ${breakdown.customFields}`;
	}

	// CC4: Bible stats computed from entries
	let bibleStats = $derived.by(() => {
		const entries = appState.bibleEntries;
		const byType: Record<string, number> = {};
		let relationshipCount = 0;

		for (const entry of entries) {
			byType[entry.entry_type] = (byType[entry.entry_type] || 0) + 1;
		}

		// Relationship count is only accurate for selected entry
		if (selectedEntry) {
			relationshipCount = relationships.length;
		}

		return {
			total: entries.length,
			byType,
			characters: byType['character'] || 0,
			locations: byType['location'] || 0,
			objects: byType['object'] || 0,
			factions: byType['faction'] || 0,
			concepts: byType['concept'] || 0,
			glossary: byType['glossary'] || 0,
			relationships: relationshipCount,
		};
	});

	function saveCustomFields() {
		if (!selectedEntry) return;
		const fieldsObj: Record<string, string> = {};
		for (const field of customFields) {
			if (field.name.trim()) {
				fieldsObj[field.name.trim()] = field.value;
			}
		}
		const json = Object.keys(fieldsObj).length > 0 ? JSON.stringify(fieldsObj) : '';
		updateEntry('custom_fields', json);
	}
</script>

<div class="bible-view">
	<div class="bible-sidebar">
		<div class="sidebar-header">
			<h2>Bible</h2>
			<Button variant="icon" onclick={() => (isCreating = true)} title="Add new entry">
				<Icon name="plus" size={16} />
			</Button>
		</div>

		<div class="filter-tabs">
			<button
				class="filter-tab"
				class:active={appState.bibleFilter === null}
				onclick={() => (appState.bibleFilter = null)}
			>
				All
			</button>
			{#each bibleEntryTypes as type (type.value)}
				<button
					class="filter-tab"
					class:active={appState.bibleFilter === type.value}
					onclick={() => (appState.bibleFilter = type.value)}
					title={type.label}
				>
					{type.icon}
				</button>
			{/each}
		</div>

		<div class="bible-search-wrapper">
			<input
				type="text"
				placeholder="Search entries..."
				bind:value={appState.bibleSearchQuery}
				class="bible-search"
			/>
			{#if appState.bibleSearchQuery}
				<button
					class="clear-search"
					onclick={() => (appState.bibleSearchQuery = '')}
					title="Clear search"
				>
					<Icon name="close" size={14} />
				</button>
			{/if}
		</div>

		<!-- CC4: Bible stats panel -->
		<div class="bible-stats-panel">
			<span class="stat-item" title="Total entries">{bibleStats.total} entries</span>
			<span class="stat-item" title="Average completion">{avgCompletion}% avg</span>
			{#if bibleStats.characters > 0}
				<span class="stat-item" title="Characters">👤 {bibleStats.characters}</span>
			{/if}
			{#if bibleStats.locations > 0}
				<span class="stat-item" title="Locations">📍 {bibleStats.locations}</span>
			{/if}
			{#if bibleStats.objects > 0}
				<span class="stat-item" title="Objects">🔮 {bibleStats.objects}</span>
			{/if}
			{#if bibleStats.factions > 0}
				<span class="stat-item" title="Factions">⚔️ {bibleStats.factions}</span>
			{/if}
		</div>

		{#if isCreating}
			<div class="new-entry-form">
				<select bind:value={newEntryType}>
					{#each bibleEntryTypes as type (type.value)}
						<option value={type.value}>{type.icon} {type.label}</option>
					{/each}
				</select>
				<input
					type="text"
					placeholder="Name..."
					maxlength={100}
					bind:value={newEntryName}
					onkeydown={(e) => e.key === 'Enter' && createEntry()}
				/>
				<FormActions>
					<Button variant="ghost" onclick={() => (isCreating = false)}>Cancel</Button>
					<Button variant="primary" onclick={createEntry}>Create</Button>
				</FormActions>
			</div>
		{/if}

		<div class="entries-list">
			{#each appState.filteredBibleEntries as entry (entry.id)}
				{@const typeInfo = getTypeInfo(entry.entry_type)}
				{@const completionDetails = computeCompletionDetails(entry)}
				<button
					class="entry-item"
					class:selected={selectedEntry?.id === entry.id}
					onclick={() => selectEntry(entry)}
					oncontextmenu={(e) => handleEntryContextMenu(e, entry.id)}
					style="--entry-color: {entry.color || 'var(--color-accent)'}"
				>
					<span class="entry-icon">{typeInfo.icon}</span>
					<div class="entry-info">
						<span class="entry-name truncate">{entry.name}</span>
						<!-- CC2: Enhanced tooltip with detailed breakdown -->
						<div class="entry-completion-bar" title={getCompletionTooltip(completionDetails)}>
							<div class="entry-completion-fill" style="width: {completionDetails.total}%"></div>
						</div>
					</div>
					<span class="entry-status">{entry.status}</span>
				</button>
			{:else}
				<EmptyState
					compact
					icon="book"
					title="No entries found"
					description="Create a new entry to start building your story bible."
					actionLabel="Create Entry"
					onaction={() => (isCreating = true)}
				/>
			{/each}
		</div>
	</div>

	<div class="bible-content">
		{#if selectedEntry}
			{@const typeInfo = getTypeInfo(selectedEntry.entry_type)}
			<div class="entry-header">
				<div class="entry-title-section">
					<span class="type-badge">{typeInfo.icon} {typeInfo.label}</span>
					<input
						type="text"
						class="entry-name-input"
						value={selectedEntry.name}
						onblur={(e) => updateEntry('name', e.currentTarget.value)}
					/>
				</div>
				<div class="entry-actions">
					<div class="view-toggle">
						<button
							class="toggle-btn"
							class:active={bibleViewMode === 'detail'}
							onclick={() => (bibleViewMode = 'detail')}>Detail</button
						>
						<button
							class="toggle-btn"
							class:active={bibleViewMode === 'graph'}
							onclick={() => (bibleViewMode = 'graph')}>Graph</button
						>
					</div>
					<select
						value={selectedEntry.status}
						onchange={(e) => updateEntry('status', e.currentTarget.value)}
					>
						{#each bibleStatuses as status (status.value)}
							<option value={status.value}>{status.label}</option>
						{/each}
					</select>
					<Button variant="icon" class="danger" onclick={deleteEntry} title="Delete entry">
						<Icon name="delete" size={16} />
					</Button>
				</div>
			</div>

			{#if bibleViewMode === 'graph'}
				<RelationshipMap entryId={selectedEntry.id} entryName={selectedEntry.name} />
			{:else}
				<div class="entry-fields">
					<FormGroup label="Aliases" id="entry-aliases">
						<input
							id="entry-aliases"
							type="text"
							placeholder="Alternative names (comma-separated)"
							value={selectedEntry.aliases || ''}
							onblur={(e) => updateEntry('aliases', e.currentTarget.value)}
						/>
					</FormGroup>

					<FormGroup label="Short Description" id="entry-short-desc">
						<input
							id="entry-short-desc"
							type="text"
							placeholder="Brief description for tooltips"
							value={selectedEntry.summary || ''}
							onblur={(e) => updateEntry('summary', e.currentTarget.value)}
						/>
					</FormGroup>

					<FormGroup label="Full Description" id="entry-full-desc">
						<textarea
							id="entry-full-desc"
							rows="6"
							placeholder="Detailed description..."
							value={selectedEntry.full_description || ''}
							onblur={(e) => updateEntry('full_description', e.currentTarget.value)}
						></textarea>
					</FormGroup>

					<FormGroup label="Tags" id="entry-tags">
						<input
							id="entry-tags"
							type="text"
							placeholder="Tags (comma-separated)"
							value={selectedEntry.tags || ''}
							onblur={(e) => updateEntry('tags', e.currentTarget.value)}
						/>
					</FormGroup>

					<FormGroup label="Notes" id="entry-notes">
						<textarea
							id="entry-notes"
							rows="4"
							placeholder="Private notes..."
							value={selectedEntry.notes || ''}
							onblur={(e) => updateEntry('notes', e.currentTarget.value)}
						></textarea>
					</FormGroup>

					{#if loadRelatedError}
						<div class="load-error-banner">
							<span>Some related data couldn't be loaded.</span>
							<button
								onclick={() => {
									if (appState.selectedBibleEntryId) {
										loadRelatedError = false;
										loadAllRelated(appState.selectedBibleEntryId);
									}
								}}
							>
								Retry
							</button>
						</div>
					{/if}

					{#if isLoadingRelated}
						<div class="related-loading">
							<LoadingState message="Loading related data..." />
						</div>
					{:else}
						<div class="relationships-section">
							<div class="section-header">
								<h4>Relationships</h4>
								<Button variant="secondary" size="sm" onclick={() => (showRelationshipForm = true)}>
									<Icon name="plus" size={14} />
									Add
								</Button>
							</div>

							{#if showRelationshipForm}
								<div class="relationship-form" transition:slide={{ duration: 150 }}>
									<select bind:value={newRelationshipType}>
										{#each relationshipTypes as type (type.value)}
											<option value={type.value}>{type.label}</option>
										{/each}
									</select>
									<select bind:value={newRelationshipTarget}>
										<option value="">Select entry...</option>
										{#each appState.bibleEntries.filter((e) => e.id !== selectedEntry?.id) as entry (entry.id)}
											{@const typeInfo = getTypeInfo(entry.entry_type)}
											<option value={entry.id}>{typeInfo.icon} {entry.name}</option>
										{/each}
									</select>
									<FormActions>
										<Button variant="ghost" onclick={() => (showRelationshipForm = false)}
											>Cancel</Button
										>
										<Button
											variant="primary"
											onclick={createRelationship}
											disabled={!newRelationshipTarget}>Add</Button
										>
									</FormActions>
								</div>
							{/if}

							{#if relationships.length > 0}
								<div class="relationships-list">
									{#each relationships as rel (rel.id)}
										{#if editingRelationshipId === rel.id}
											<div class="relationship-edit-form">
												<select bind:value={editedRelType}>
													{#each relationshipTypes as type (type.value)}
														<option value={type.value}>{type.label}</option>
													{/each}
												</select>
												<input
													type="text"
													class="rel-note-input"
													placeholder="Note (optional)"
													bind:value={editedRelNote}
												/>
												<FormActions>
													<Button variant="ghost" size="sm" onclick={cancelEditingRelationship}
														>Cancel</Button
													>
													<Button variant="primary" size="sm" onclick={saveRelationshipEdit}
														>Save</Button
													>
												</FormActions>
											</div>
										{:else}
											<div class="relationship-item">
												<span class="relationship-type"
													>{getRelationshipLabel(rel.relationship_type)}</span
												>
												<!-- AO4: Use selectBibleEntry for navigation history -->
												<button
													class="relationship-target"
													onclick={() => appState.selectBibleEntry(rel.related_entry_id)}
												>
													{rel.related_entry_name}
												</button>
												{#if rel.note}
													<span class="relationship-note">{rel.note}</span>
												{/if}
												<Button
													variant="icon"
													size="sm"
													onclick={() => startEditingRelationship(rel)}
													title="Edit relationship"
												>
													<Icon name="edit" size={12} />
												</Button>
												<Button
													variant="icon"
													size="sm"
													onclick={() => deleteRelationship(rel.id)}
													title="Remove relationship"
												>
													<Icon name="close" size={12} />
												</Button>
											</div>
										{/if}
									{/each}
								</div>
							{:else if !showRelationshipForm}
								<p class="no-relationships">No relationships defined</p>
							{/if}
						</div>

						<!-- Linked Events Section -->
						{#if linkedEvents.length > 0}
							<div class="events-section">
								<div class="section-header">
									<h4>Linked Events ({linkedEvents.length})</h4>
								</div>
								<div class="linked-events-list">
									{#each linkedEvents as event (event.id)}
										<div class="linked-event-item">
											<span class="linked-event-type">{event.event_type}</span>
											<span class="linked-event-title">{event.title}</span>
											{#if event.time_point}
												<span class="linked-event-time">{event.time_point}</span>
											{:else if event.time_start}
												<span class="linked-event-time">
													{event.time_start}{event.time_end ? ` - ${event.time_end}` : ''}
												</span>
											{/if}
										</div>
									{/each}
								</div>
							</div>
						{/if}

						<!-- Appears in Scenes -->
						{#if linkedScenes.length > 0}
							<div class="scenes-section">
								<div class="section-header">
									<h4>Appears in Scenes ({linkedScenes.length})</h4>
								</div>
								<div class="linked-scenes-list">
									{#each linkedScenes as scene (scene.id)}
										<button class="linked-scene-item" onclick={() => navigateToScene(scene.id)}>
											<span class="linked-scene-title">{scene.title}</span>
											<span class="linked-scene-status">{scene.status}</span>
										</button>
									{/each}
								</div>
							</div>
						{/if}

						<!-- Story Arcs (for characters) -->
						{#if linkedArcs.length > 0}
							<div class="arcs-section">
								<div class="section-header">
									<h4>Story Arcs ({linkedArcs.length})</h4>
								</div>
								<div class="linked-arcs-list">
									{#each linkedArcs as arc (arc.id)}
										<div class="linked-arc-item">
											<span class="arc-color-dot" style="background-color: {arc.color || '#6366f1'}"
											></span>
											<span class="linked-arc-name">{arc.name}</span>
											<span class="linked-arc-status">{arc.status}</span>
										</div>
									{/each}
								</div>
							</div>
						{/if}

						<!-- Related Issues -->
						{#if linkedIssues.length > 0}
							<div class="issues-section">
								<div class="section-header">
									<h4>Related Issues ({linkedIssues.length})</h4>
								</div>
								<div class="linked-issues-list">
									{#each linkedIssues as issue (issue.id)}
										<div class="linked-issue-item" data-severity={issue.severity}>
											<span class="linked-issue-severity">{issue.severity}</span>
											<span class="linked-issue-title">{issue.title}</span>
											<span class="linked-issue-status">{issue.status}</span>
										</div>
									{/each}
								</div>
							</div>
						{/if}
					{/if}

					<!-- Image Section -->
					<div class="image-section">
						<div class="section-header">
							<h4>Image</h4>
						</div>
						{#if selectedEntry.image_path}
							<div class="image-preview">
								<img src={selectedEntry.image_path} alt={selectedEntry.name} />
								<Button
									variant="icon"
									size="sm"
									onclick={() => updateEntry('image_path', '')}
									title="Remove image"
								>
									<Icon name="close" size={14} />
								</Button>
							</div>
						{:else}
							<div class="image-upload">
								<input
									type="text"
									placeholder="Enter image path or URL..."
									onblur={(e) => updateEntry('image_path', e.currentTarget.value)}
								/>
							</div>
						{/if}
					</div>

					<!-- Color Section -->
					<div class="color-section">
						<div class="section-header">
							<h4>Color</h4>
						</div>
						<div class="color-palette">
							{#each entryColors as color (color)}
								<button
									class="color-swatch"
									class:selected={selectedEntry.color === color}
									style="background-color: {color}"
									onclick={() => updateEntry('color', color)}
									title={color}
								></button>
							{/each}
							{#if selectedEntry.color}
								<button
									class="color-swatch clear-color"
									onclick={() => updateEntry('color', '')}
									title="Clear color"
								>
									<Icon name="close" size={10} />
								</button>
							{/if}
						</div>
					</div>

					<!-- Custom Fields Section -->
					<div class="custom-fields-section">
						<div class="section-header">
							<h4>Custom Fields</h4>
							<Button variant="secondary" size="sm" onclick={addCustomField}>
								<Icon name="plus" size={14} />
								Add
							</Button>
						</div>
						{#if customFields.length > 0}
							<div class="custom-fields-list">
								{#each customFields as field, index (index)}
									<div
										class="custom-field"
										class:textarea-field={getFieldType(field.name) === 'textarea'}
									>
										{#if isKnownField(field.name)}
											<span class="field-label">{getFieldLabel(field.name)}</span>
										{:else}
											<input
												type="text"
												class="field-name"
												placeholder="Field name"
												value={field.name}
												onblur={(e) => updateCustomFieldName(index, e.currentTarget.value)}
											/>
										{/if}
										{#if getFieldType(field.name) === 'textarea'}
											<textarea
												class="field-value"
												rows="3"
												placeholder="Value"
												value={field.value}
												onblur={(e) => updateCustomFieldValue(index, e.currentTarget.value)}
											></textarea>
										{:else}
											<input
												type="text"
												class="field-value"
												placeholder="Value"
												value={field.value}
												onblur={(e) => updateCustomFieldValue(index, e.currentTarget.value)}
											/>
										{/if}
										<Button
											variant="icon"
											size="sm"
											onclick={() => removeCustomField(index)}
											title="Remove field"
										>
											<Icon name="close" size={14} />
										</Button>
									</div>
								{/each}
							</div>
						{:else}
							<p class="no-custom-fields">No custom fields defined</p>
						{/if}
					</div>
				</div>
			{/if}
		{:else}
			<EmptyState
				icon="info"
				title="No entry selected"
				description="Select an entry from the list or create a new one."
			/>
		{/if}
	</div>
</div>

{#if impactDialog}
	<ImpactDialog
		entityType="bible_entry"
		entityId={impactDialog.entityId}
		entityName={impactDialog.entityName}
		onconfirm={confirmDeleteEntry}
		oncancel={() => (impactDialog = null)}
	/>
{/if}

{#if entryContextMenu}
	<ContextMenu x={entryContextMenu.x} y={entryContextMenu.y} onclose={closeEntryContextMenu}>
		<ContextMenuItem
			label="Edit"
			onclick={() => {
				const entry = appState.bibleEntries.find((e) => e.id === entryContextMenu!.entryId);
				if (entry) selectEntry(entry);
				closeEntryContextMenu();
			}}
		/>
		<ContextMenuSeparator />
		<ContextMenuItem
			label="Delete"
			danger
			onclick={() => {
				const entry = appState.bibleEntries.find((e) => e.id === entryContextMenu!.entryId);
				if (entry) {
					selectEntry(entry);
					impactDialog = { entityId: entry.id, entityName: entry.name };
				}
				closeEntryContextMenu();
			}}
		/>
	</ContextMenu>
{/if}

<style>
	.bible-view {
		display: flex;
		height: 100%;
	}

	.bible-sidebar {
		width: 280px;
		background-color: var(--color-bg-secondary);
		border-right: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
	}

	.sidebar-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md);
		border-bottom: 1px solid var(--color-border-light);
	}

	.sidebar-header h2 {
		font-size: var(--font-size-md);
		font-weight: 600;
	}

	.filter-tabs {
		display: flex;
		gap: 2px;
		padding: var(--spacing-sm);
		background-color: var(--color-bg-tertiary);
		overflow-x: auto;
	}

	.filter-tab {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
		white-space: nowrap;
		transition: all var(--transition-fast);
	}

	.filter-tab:hover {
		background-color: var(--color-bg-hover);
	}

	.filter-tab.active {
		background-color: var(--color-bg-primary);
		color: var(--color-text-primary);
	}

	.bible-search-wrapper {
		position: relative;
		padding: 0 var(--spacing-sm) var(--spacing-sm);
	}

	.bible-search {
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-sm);
		padding-right: 28px;
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background: var(--color-bg-primary);
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.bible-search:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.clear-search {
		position: absolute;
		right: var(--spacing-md);
		top: 50%;
		transform: translateY(-50%);
		color: var(--color-text-muted);
		cursor: pointer;
		display: flex;
		align-items: center;
		padding: 2px;
	}

	.clear-search:hover {
		color: var(--color-text-primary);
	}

	/* CC4: Bible stats panel */
	.bible-stats-panel {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-sm) var(--spacing-sm);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		border-bottom: 1px solid var(--color-border-light);
	}

	.stat-item {
		padding: 2px var(--spacing-xs);
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		white-space: nowrap;
	}

	.new-entry-form {
		padding: var(--spacing-md);
		border-bottom: 1px solid var(--color-border-light);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.new-entry-form select,
	.new-entry-form input {
		width: 100%;
		font-size: var(--font-size-sm);
		padding: var(--spacing-sm);
	}

	.entries-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-sm) 0;
	}

	.entry-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		width: 100%;
		padding: var(--spacing-sm) var(--spacing-md);
		text-align: left;
		font-size: var(--font-size-sm);
		border-left: 3px solid transparent;
		transition:
			background-color var(--transition-fast),
			border-color var(--transition-fast);
	}

	.entry-item:hover {
		background-color: var(--color-bg-hover);
		border-left-color: var(--border-strong);
	}

	.entry-item.selected {
		background-color: var(--color-accent-light);
		border-left-color: var(--entry-color);
	}

	.entry-icon {
		flex-shrink: 0;
	}

	.entry-info {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.entry-name {
		flex: 1;
		min-width: 0;
	}

	.entry-completion-bar {
		width: 100%;
		height: 3px;
		background-color: var(--color-bg-tertiary);
		border-radius: 2px;
		overflow: hidden;
	}

	.entry-completion-fill {
		height: 100%;
		background-color: var(--color-accent);
		border-radius: 2px;
		transition: width var(--transition-fast);
	}

	.entry-status {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.bible-content {
		flex: 1;
		overflow-y: auto;
		background-color: var(--color-bg-primary);
	}

	.entry-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-lg);
		border-bottom: 1px solid var(--color-border-light);
		gap: var(--spacing-md);
	}

	.entry-title-section {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.type-badge {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.entry-name-input {
		font-size: var(--font-size-xl);
		font-weight: 600;
		border: none;
		border-bottom: 1px dashed transparent;
		background: none;
		padding: 0;
		cursor: text;
		transition: border-color var(--transition-fast);
	}

	.entry-name-input:hover {
		border-bottom-color: var(--color-border);
	}

	.entry-name-input:focus {
		outline: none;
		border-bottom: 2px solid var(--color-accent);
	}

	.entry-actions {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.entry-actions select {
		font-size: var(--font-size-sm);
		padding: var(--spacing-xs) var(--spacing-sm);
	}

	.view-toggle {
		display: flex;
		gap: 1px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		padding: 2px;
	}

	.toggle-btn {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-xs);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
		transition: all var(--transition-fast);
	}

	.toggle-btn:hover {
		color: var(--color-text-primary);
	}

	.toggle-btn.active {
		background-color: var(--color-bg-primary);
		color: var(--color-text-primary);
		box-shadow: var(--shadow-sm);
	}

	/* Danger variant for icon buttons */
	.entry-actions :global(.btn-icon.danger:hover) {
		color: var(--color-error);
	}

	.entry-fields {
		padding: var(--spacing-lg);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-lg);
		max-width: 700px;
	}

	.relationships-section {
		margin-top: var(--spacing-lg);
		padding-top: var(--spacing-lg);
		border-top: 1px solid var(--color-border-light);
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--spacing-md);
	}

	.section-header h4 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
	}

	.relationship-form {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-md);
		margin-bottom: var(--spacing-md);
	}

	.relationship-form select {
		width: 100%;
		font-size: var(--font-size-sm);
		padding: var(--spacing-sm);
	}

	.relationships-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.relationship-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-sm);
	}

	.relationship-type {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.relationship-target {
		flex: 1;
		font-size: var(--font-size-sm);
		color: var(--color-accent);
		text-align: left;
	}

	.relationship-target:hover {
		text-decoration: underline;
	}

	.no-relationships {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.relationship-edit-form {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
	}

	.relationship-edit-form select,
	.rel-note-input {
		width: 100%;
		font-size: var(--font-size-sm);
		padding: var(--spacing-xs) var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
		color: var(--color-text-primary);
	}

	.relationship-note {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		font-style: italic;
		flex-shrink: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.relationship-item :global(.btn-icon) {
		opacity: 0;
		transition: opacity var(--transition-fast);
		flex-shrink: 0;
	}

	.relationship-item:hover :global(.btn-icon) {
		opacity: 1;
	}

	/* Linked Events Section */
	.events-section {
		margin-top: var(--spacing-lg);
		padding-top: var(--spacing-lg);
		border-top: 1px solid var(--color-border-light);
	}

	.linked-events-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.linked-event-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-sm);
	}

	.linked-event-type {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		flex-shrink: 0;
		text-transform: capitalize;
	}

	.linked-event-title {
		flex: 1;
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.linked-event-time {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		flex-shrink: 0;
	}

	/* Image Section */
	.image-section {
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-md);
	}

	.image-preview {
		position: relative;
		margin-top: var(--spacing-sm);
	}

	.image-preview img {
		max-width: 100%;
		max-height: 200px;
		border-radius: var(--border-radius-sm);
		object-fit: contain;
	}

	.image-preview :global(.btn) {
		position: absolute;
		top: var(--spacing-xs);
		right: var(--spacing-xs);
		background-color: var(--color-bg-primary);
		opacity: 0.8;
	}

	.image-preview :global(.btn:hover) {
		opacity: 1;
	}

	.image-upload input {
		width: 100%;
		padding: var(--spacing-sm);
		border: 1px dashed var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	/* Color Section */
	.color-section {
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-md);
	}

	.color-palette {
		display: flex;
		gap: var(--spacing-sm);
		flex-wrap: wrap;
		margin-top: var(--spacing-sm);
	}

	.color-swatch {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		border: 2px solid transparent;
		transition: all var(--transition-fast);
	}

	.color-swatch:hover {
		transform: scale(1.15);
	}

	.color-swatch.selected {
		border-color: var(--color-text-primary);
		box-shadow: 0 0 0 2px var(--color-bg-primary);
	}

	.color-swatch.clear-color {
		background-color: var(--color-bg-tertiary);
		border: 1px dashed var(--color-border);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-muted);
	}

	/* Custom Fields Section */
	.custom-fields-section {
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-md);
	}

	.custom-fields-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
		margin-top: var(--spacing-sm);
	}

	.custom-field {
		display: flex;
		gap: var(--spacing-xs);
		align-items: center;
	}

	.custom-field .field-name {
		width: 120px;
		flex-shrink: 0;
		padding: var(--spacing-xs) var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.custom-field .field-label {
		width: 120px;
		flex-shrink: 0;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-secondary);
	}

	.custom-field .field-value {
		flex: 1;
		padding: var(--spacing-xs) var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.custom-field.textarea-field {
		align-items: flex-start;
	}

	.custom-field textarea.field-value {
		resize: vertical;
		min-height: 60px;
		font-family: inherit;
	}

	.no-custom-fields {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		margin-top: var(--spacing-sm);
	}

	/* Appears in Scenes */
	.scenes-section,
	.arcs-section,
	.issues-section {
		margin-top: var(--spacing-lg);
		padding-top: var(--spacing-lg);
		border-top: 1px solid var(--color-border-light);
	}

	.linked-scenes-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.linked-scene-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-sm);
		text-align: left;
		transition: background-color var(--transition-fast);
	}

	.linked-scene-item:hover {
		background-color: var(--color-bg-hover);
	}

	.linked-scene-title {
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-accent);
	}

	.linked-scene-status {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-transform: capitalize;
	}

	/* Story Arcs */
	.linked-arcs-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.linked-arc-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-sm);
	}

	.arc-color-dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.linked-arc-name {
		flex: 1;
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.linked-arc-status {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-transform: capitalize;
	}

	/* Related Issues */
	.linked-issues-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.linked-issue-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-sm);
	}

	.linked-issue-severity {
		font-size: var(--font-size-xs);
		font-weight: 500;
		text-transform: capitalize;
		flex-shrink: 0;
	}

	.linked-issue-item[data-severity='error'] .linked-issue-severity {
		color: var(--color-error);
	}

	.linked-issue-item[data-severity='warning'] .linked-issue-severity {
		color: var(--color-warning);
	}

	.linked-issue-item[data-severity='info'] .linked-issue-severity {
		color: var(--color-info);
	}

	.linked-issue-title {
		flex: 1;
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.linked-issue-status {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-transform: capitalize;
	}

	.load-error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-sm) var(--spacing-md);
		background-color: var(--color-warning-bg, light-dark(#fef3c7, #422006));
		border: 1px solid var(--color-warning);
		border-radius: var(--border-radius-sm);
		margin: var(--spacing-sm) 0;
		font-size: var(--font-size-sm);
		color: var(--color-warning);
	}

	.load-error-banner button {
		padding: 2px var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		font-weight: 500;
		color: var(--color-warning);
		text-decoration: underline;
		cursor: pointer;
	}

	.related-loading {
		padding: var(--spacing-lg) 0;
	}

	/* Responsive adjustments */
	@media (max-width: 900px) {
		.bible-sidebar {
			width: 240px;
		}
	}

	@media (max-width: 800px) {
		.bible-sidebar {
			width: 200px;
		}
		.filter-tabs {
			flex-wrap: nowrap;
			-webkit-overflow-scrolling: touch;
		}
		.filter-tab {
			font-size: var(--font-size-xs);
			padding: var(--spacing-xs);
		}
	}
</style>
