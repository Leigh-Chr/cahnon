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
	import { appState } from '$lib/stores';
	import { relationshipApi, type BibleEntry, type BibleRelationshipWithEntry } from '$lib/api';
	import { bibleEntryTypes, bibleStatuses } from '$lib/utils';
	import { Icon, Button, FormGroup, FormActions } from './ui';

	let isCreating = $state(false);
	let newEntryType = $state('character');
	let newEntryName = $state('');

	// Relationships
	let relationships = $state<BibleRelationshipWithEntry[]>([]);
	let showRelationshipForm = $state(false);
	let newRelationshipTarget = $state('');
	let newRelationshipType = $state('related_to');

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

	// Side effect: load relationships when selection changes
	$effect(() => {
		if (appState.selectedBibleEntryId) {
			loadRelationships(appState.selectedBibleEntryId);
		}
	});

	async function loadRelationships(entryId: string) {
		try {
			relationships = await relationshipApi.getByEntry(entryId);
		} catch (e) {
			console.error('Failed to load relationships:', e);
			relationships = [];
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
		}
	}

	async function deleteRelationship(relationshipId: string) {
		if (!selectedEntry) return;
		try {
			await relationshipApi.delete(relationshipId);
			await loadRelationships(selectedEntry.id);
		} catch (e) {
			console.error('Failed to delete relationship:', e);
		}
	}

	function getRelationshipLabel(type: string): string {
		return relationshipTypes.find((t) => t.value === type)?.label || type;
	}

	function selectEntry(entry: BibleEntry) {
		appState.selectedBibleEntryId = entry.id;
	}

	async function createEntry() {
		if (!newEntryName.trim()) return;

		await appState.createBibleEntry({
			entry_type: newEntryType,
			name: newEntryName.trim(),
		});

		newEntryName = '';
		isCreating = false;
	}

	async function updateEntry(field: string, value: string) {
		if (selectedEntry) {
			await appState.updateBibleEntry(selectedEntry.id, { [field]: value });
		}
	}

	async function deleteEntry() {
		if (selectedEntry && confirm(`Delete "${selectedEntry.name}"? This cannot be undone.`)) {
			await appState.deleteBibleEntry(selectedEntry.id);
			appState.selectedBibleEntryId = null;
		}
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
	let lastSyncedEntryId = $state<string | null>(null);

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
				<button
					class="entry-item"
					class:selected={selectedEntry?.id === entry.id}
					onclick={() => selectEntry(entry)}
					style="--entry-color: {entry.color || 'var(--color-accent)'}"
				>
					<span class="entry-icon">{typeInfo.icon}</span>
					<span class="entry-name truncate">{entry.name}</span>
					<span class="entry-status">{entry.status}</span>
				</button>
			{:else}
				<div class="empty-list">
					<p>No entries found</p>
				</div>
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
						value={selectedEntry.short_description || ''}
						onblur={(e) => updateEntry('short_description', e.currentTarget.value)}
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

				<div class="relationships-section">
					<div class="section-header">
						<h4>Relationships</h4>
						<Button variant="secondary" size="sm" onclick={() => (showRelationshipForm = true)}>
							<Icon name="plus" size={14} />
							Add
						</Button>
					</div>

					{#if showRelationshipForm}
						<div class="relationship-form">
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
								<div class="relationship-item">
									<span class="relationship-type"
										>{getRelationshipLabel(rel.relationship_type)}</span
									>
									<button
										class="relationship-target"
										onclick={() => (appState.selectedBibleEntryId = rel.related_entry_id)}
									>
										{rel.related_entry_name}
									</button>
									<Button
										variant="icon"
										size="sm"
										onclick={() => deleteRelationship(rel.id)}
										title="Remove relationship"
									>
										<Icon name="close" size={12} />
									</Button>
								</div>
							{/each}
						</div>
					{:else if !showRelationshipForm}
						<p class="no-relationships">No relationships defined</p>
					{/if}
				</div>

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
								<div class="custom-field">
									<input
										type="text"
										class="field-name"
										placeholder="Field name"
										value={field.name}
										onblur={(e) => updateCustomFieldName(index, e.currentTarget.value)}
									/>
									<input
										type="text"
										class="field-value"
										placeholder="Value"
										value={field.value}
										onblur={(e) => updateCustomFieldValue(index, e.currentTarget.value)}
									/>
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
		{:else}
			<div class="no-selection">
				<Icon name="info" size={48} strokeWidth={1.5} />
				<h3>No entry selected</h3>
				<p>Select an entry from the list or create a new one.</p>
			</div>
		{/if}
	</div>
</div>

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
		transition: background-color var(--transition-fast);
		border-left: 3px solid transparent;
	}

	.entry-item:hover {
		background-color: var(--color-bg-hover);
	}

	.entry-item.selected {
		background-color: var(--color-accent-light);
		border-left-color: var(--entry-color);
	}

	.entry-icon {
		flex-shrink: 0;
	}

	.entry-name {
		flex: 1;
		min-width: 0;
	}

	.entry-status {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.empty-list {
		padding: var(--spacing-lg);
		text-align: center;
		color: var(--color-text-muted);
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
		background: none;
		padding: 0;
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

	.no-selection {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		text-align: center;
		color: var(--color-text-muted);
		padding: var(--spacing-xl);
	}

	.no-selection :global(.icon) {
		margin-bottom: var(--spacing-md);
		opacity: 0.5;
	}

	.no-selection h3 {
		font-size: var(--font-size-lg);
		font-weight: 500;
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-sm);
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

	.custom-field .field-value {
		flex: 1;
		padding: var(--spacing-xs) var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.no-custom-fields {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		margin-top: var(--spacing-sm);
	}
</style>
