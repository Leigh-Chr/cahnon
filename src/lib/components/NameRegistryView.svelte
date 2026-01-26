<!--
  Name Registry view for tracking proper nouns across the manuscript.

  Features:
  - Centralized list of all proper nouns
  - Scan manuscript to detect names automatically
  - Detect similar spellings (fuzzy match) and case inconsistencies
  - Actions: merge entries, add alias, create bible entry, mark as intentional
  - Filter by type and confirmation status
  - View mentions per entry
-->
<script lang="ts">
	import {
		type NameMention,
		nameMentionApi,
		nameRegistryApi,
		type NameRegistryEntry,
	} from '$lib/api';
	import { appState } from '$lib/stores';

	import { Button, EmptyState, FormActions, FormGroup } from './ui';

	let entries = $state<NameRegistryEntry[]>([]);
	let isLoading = $state(false);
	let isScanning = $state(false);
	let scanResult = $state<string | null>(null);

	// Filters
	let filterType = $state('');
	let filterConfirmed = $state('');
	let searchText = $state('');

	// Selected entry
	let selectedEntryId = $state<string | null>(null);
	let selectedMentions = $state<NameMention[]>([]);

	// Create form
	let showCreateForm = $state(false);
	let newName = $state('');
	let newType = $state('character');
	let newAliases = $state('');

	// Merge mode
	let mergeMode = $state(false);
	let mergeTargetId = $state<string | null>(null);

	// Edit mode
	let isEditingEntry = $state(false);
	let editedCanonicalName = $state('');
	let editedNameType = $state('character');
	let editedAliases = $state('');

	// Fuzzy match results
	let similarGroups = $state<Array<{ entries: NameRegistryEntry[]; similarity: string }>>([]);

	const nameTypes = [
		{ value: 'character', label: 'Character' },
		{ value: 'location', label: 'Location' },
		{ value: 'object', label: 'Object' },
		{ value: 'faction', label: 'Faction' },
		{ value: 'other', label: 'Other' },
	];

	let selectedEntry = $derived(
		selectedEntryId ? entries.find((e) => e.id === selectedEntryId) || null : null
	);

	let filteredEntries = $derived.by(() => {
		let result = entries;
		if (filterType) result = result.filter((e) => e.name_type === filterType);
		if (filterConfirmed === 'confirmed') result = result.filter((e) => e.is_confirmed);
		if (filterConfirmed === 'unconfirmed') result = result.filter((e) => !e.is_confirmed);
		if (searchText) {
			const lower = searchText.toLowerCase();
			result = result.filter(
				(e) =>
					e.canonical_name.toLowerCase().includes(lower) ||
					(e.aliases && e.aliases.toLowerCase().includes(lower))
			);
		}
		return result;
	});

	$effect(() => {
		if (appState.project) {
			loadEntries();
		}
	});

	$effect(() => {
		if (selectedEntryId) {
			loadMentions(selectedEntryId);
		} else {
			selectedMentions = [];
		}
	});

	async function loadEntries() {
		isLoading = true;
		try {
			entries = await nameRegistryApi.getAll();
			detectSimilarNames();
		} catch (e) {
			console.error('Failed to load name registry:', e);
		} finally {
			isLoading = false;
		}
	}

	async function loadMentions(entryId: string) {
		try {
			selectedMentions = await nameMentionApi.getByRegistry(entryId);
		} catch (e) {
			console.error('Failed to load mentions:', e);
		}
	}

	async function scanManuscript() {
		isScanning = true;
		scanResult = null;
		try {
			const [newEntries, newMentions] = await nameRegistryApi.scan();
			scanResult = `Scan complete: ${newEntries} new names found, ${newMentions} mentions recorded.`;
			await loadEntries();
		} catch (e) {
			console.error('Failed to scan names:', e);
			scanResult = `Scan failed: ${e}`;
		} finally {
			isScanning = false;
		}
	}

	async function createEntry() {
		if (!newName.trim()) return;
		try {
			const entry = await nameRegistryApi.create({
				canonical_name: newName.trim(),
				name_type: newType,
				aliases: newAliases.trim() || undefined,
			});
			entries = [...entries, entry];
			showCreateForm = false;
			newName = '';
			newAliases = '';
			selectedEntryId = entry.id;
			detectSimilarNames();
		} catch (e) {
			console.error('Failed to create entry:', e);
		}
	}

	async function confirmEntry(id: string) {
		try {
			const updated = await nameRegistryApi.update(id, { is_confirmed: true });
			entries = entries.map((e) => (e.id === updated.id ? updated : e));
		} catch (e) {
			console.error('Failed to confirm entry:', e);
		}
	}

	async function deleteEntry(id: string) {
		if (!confirm('Delete this name registry entry and all its mentions?')) return;
		try {
			await nameRegistryApi.delete(id);
			entries = entries.filter((e) => e.id !== id);
			if (selectedEntryId === id) {
				selectedEntryId = null;
			}
			detectSimilarNames();
		} catch (e) {
			console.error('Failed to delete entry:', e);
		}
	}

	async function mergeEntries(keepId: string, mergeId: string) {
		try {
			const merged = await nameRegistryApi.merge(keepId, mergeId);
			entries = entries.filter((e) => e.id !== mergeId).map((e) => (e.id === keepId ? merged : e));
			mergeMode = false;
			mergeTargetId = null;
			selectedEntryId = keepId;
			detectSimilarNames();
			await loadMentions(keepId);
		} catch (e) {
			console.error('Failed to merge entries:', e);
		}
	}

	async function createBibleEntry(entry: NameRegistryEntry) {
		try {
			const bibleEntry = await appState.createBibleEntry({
				name: entry.canonical_name,
				entry_type: entry.name_type === 'character' ? 'character' : entry.name_type,
			});
			const updated = await nameRegistryApi.update(entry.id, {
				bible_entry_id: bibleEntry.id,
				is_confirmed: true,
			});
			entries = entries.map((e) => (e.id === updated.id ? updated : e));
		} catch (e) {
			console.error('Failed to create bible entry:', e);
		}
	}

	function startEditingEntry() {
		if (!selectedEntry) return;
		editedCanonicalName = selectedEntry.canonical_name;
		editedNameType = selectedEntry.name_type;
		editedAliases = selectedEntry.aliases || '';
		isEditingEntry = true;
	}

	function cancelEditingEntry() {
		isEditingEntry = false;
	}

	async function saveEditedEntry() {
		if (!selectedEntry || !editedCanonicalName.trim()) return;
		try {
			const updated = await nameRegistryApi.update(selectedEntry.id, {
				canonical_name: editedCanonicalName.trim(),
				name_type: editedNameType,
				aliases: editedAliases.trim() || undefined,
			});
			entries = entries.map((e) => (e.id === updated.id ? updated : e));
			isEditingEntry = false;
			detectSimilarNames();
		} catch (e) {
			console.error('Failed to update entry:', e);
		}
	}

	async function updateMentionStatus(mentionId: string, status: string) {
		try {
			const updated = await nameMentionApi.update(mentionId, { status });
			selectedMentions = selectedMentions.map((m) => (m.id === updated.id ? updated : m));
		} catch (e) {
			console.error('Failed to update mention:', e);
		}
	}

	function levenshtein(a: string, b: string): number {
		const matrix: number[][] = [];
		for (let i = 0; i <= b.length; i++) matrix[i] = [i];
		for (let j = 0; j <= a.length; j++) matrix[0][j] = j;

		for (let i = 1; i <= b.length; i++) {
			for (let j = 1; j <= a.length; j++) {
				if (b[i - 1] === a[j - 1]) {
					matrix[i][j] = matrix[i - 1][j - 1];
				} else {
					matrix[i][j] = Math.min(
						matrix[i - 1][j - 1] + 1,
						matrix[i][j - 1] + 1,
						matrix[i - 1][j] + 1
					);
				}
			}
		}
		return matrix[b.length][a.length];
	}

	function detectSimilarNames() {
		const groups: Array<{ entries: NameRegistryEntry[]; similarity: string }> = [];
		const used: Record<string, boolean> = {};

		for (let i = 0; i < entries.length; i++) {
			if (used[entries[i].id]) continue;

			const group: NameRegistryEntry[] = [entries[i]];
			const nameA = entries[i].canonical_name.toLowerCase();

			for (let j = i + 1; j < entries.length; j++) {
				if (used[entries[j].id]) continue;

				const nameB = entries[j].canonical_name.toLowerCase();

				// Case inconsistency: same letters, different case
				if (nameA === nameB && entries[i].canonical_name !== entries[j].canonical_name) {
					group.push(entries[j]);
					used[entries[j].id] = true;
					continue;
				}

				// Fuzzy match: Levenshtein distance <= 2 for names > 3 chars
				if (nameA.length > 3 && nameB.length > 3) {
					const dist = levenshtein(nameA, nameB);
					if (dist > 0 && dist <= 2) {
						group.push(entries[j]);
						used[entries[j].id] = true;
					}
				}
			}

			if (group.length > 1) {
				used[entries[i].id] = true;
				const similarity = group.some(
					(e) =>
						e.canonical_name.toLowerCase() === nameA &&
						e.canonical_name !== entries[i].canonical_name
				)
					? 'Case inconsistency'
					: 'Similar spelling';
				groups.push({ entries: group, similarity });
			}
		}

		similarGroups = groups;
	}

	function getSceneTitle(sceneId: string): string {
		for (const [, scenes] of appState.scenes) {
			const scene = scenes.find((s) => s.id === sceneId);
			if (scene) return scene.title;
		}
		return 'Unknown scene';
	}

	function navigateToScene(sceneId: string) {
		for (const [chapterId, scenes] of appState.scenes) {
			const scene = scenes.find((s) => s.id === sceneId);
			if (scene) {
				appState.selectedChapterId = chapterId;
				appState.selectedSceneId = sceneId;
				appState.setViewMode('editor');
				return;
			}
		}
	}

	function getBibleEntryName(id: string): string {
		const entry = appState.bibleEntries.find((e) => e.id === id);
		return entry?.name || 'Unknown';
	}
</script>

<div class="name-registry">
	<div class="registry-sidebar">
		<div class="sidebar-header">
			<h2>Name Registry</h2>
			<div class="header-actions">
				<Button size="sm" onclick={scanManuscript} disabled={isScanning}>
					{isScanning ? 'Scanning...' : 'Scan Manuscript'}
				</Button>
				<Button size="sm" variant="primary" onclick={() => (showCreateForm = true)}>
					+ Add Name
				</Button>
			</div>
			{#if scanResult}
				<p class="scan-result">{scanResult}</p>
			{/if}
		</div>

		<div class="filters">
			<input
				type="text"
				bind:value={searchText}
				placeholder="Search names..."
				class="search-input"
			/>
			<select bind:value={filterType}>
				<option value="">All Types</option>
				{#each nameTypes as type (type.value)}
					<option value={type.value}>{type.label}</option>
				{/each}
			</select>
			<select bind:value={filterConfirmed}>
				<option value="">All</option>
				<option value="confirmed">Confirmed</option>
				<option value="unconfirmed">Unconfirmed</option>
			</select>
		</div>

		{#if similarGroups.length > 0}
			<div class="similar-names-banner">
				<strong>{similarGroups.length} potential issue{similarGroups.length > 1 ? 's' : ''}</strong>
				<span class="banner-detail">Similar or inconsistent names detected</span>
			</div>
		{/if}

		<div class="entries-list">
			{#if isLoading}
				<div class="loading">Loading names...</div>
			{:else if filteredEntries.length === 0}
				<EmptyState title="No names found" />
			{:else}
				{#each filteredEntries as entry (entry.id)}
					<button
						class="entry-item"
						class:selected={selectedEntryId === entry.id}
						class:unconfirmed={!entry.is_confirmed}
						class:merge-target={mergeMode && mergeTargetId !== entry.id}
						onclick={() => {
							if (mergeMode && mergeTargetId && mergeTargetId !== entry.id) {
								mergeEntries(mergeTargetId, entry.id);
							} else {
								selectedEntryId = entry.id;
							}
						}}
					>
						<div class="entry-content">
							<span class="entry-name">{entry.canonical_name}</span>
							<span class="entry-meta">
								{nameTypes.find((t) => t.value === entry.name_type)?.label || entry.name_type}
								{#if entry.aliases}
									&bull; {entry.aliases.split(',').length} alias{entry.aliases.split(',').length > 1
										? 'es'
										: ''}
								{/if}
								{#if !entry.is_confirmed}
									&bull; <span class="unconfirmed-badge">Unconfirmed</span>
								{/if}
							</span>
						</div>
						{#if entry.bible_entry_id}
							<span class="bible-link-indicator" title="Linked to Bible entry">B</span>
						{/if}
					</button>
				{/each}
			{/if}
		</div>
	</div>

	<div class="registry-detail">
		{#if showCreateForm}
			<div class="detail-content">
				<h3>Add New Name</h3>
				<form
					onsubmit={(e) => {
						e.preventDefault();
						createEntry();
					}}
				>
					<FormGroup label="Canonical Name">
						<input type="text" bind:value={newName} placeholder="e.g. John Smith" />
					</FormGroup>

					<FormGroup label="Type">
						<select bind:value={newType}>
							{#each nameTypes as type (type.value)}
								<option value={type.value}>{type.label}</option>
							{/each}
						</select>
					</FormGroup>

					<FormGroup label="Aliases (comma-separated)">
						<input type="text" bind:value={newAliases} placeholder="e.g. Johnny, J. Smith" />
					</FormGroup>

					<FormActions>
						<Button onclick={() => (showCreateForm = false)}>Cancel</Button>
						<Button variant="primary" type="submit">Add Name</Button>
					</FormActions>
				</form>
			</div>
		{:else if mergeMode}
			<div class="detail-content">
				<h3>Merge Names</h3>
				<p>
					Keeping: <strong>{entries.find((e) => e.id === mergeTargetId)?.canonical_name}</strong>
				</p>
				<p>Click another name in the list to merge it into the kept entry.</p>
				<p class="merge-info">
					The merged entry's name will become an alias, and all its mentions will be transferred.
				</p>
				<Button
					onclick={() => {
						mergeMode = false;
						mergeTargetId = null;
					}}>Cancel Merge</Button
				>
			</div>
		{:else if selectedEntry}
			<div class="detail-content">
				{#if isEditingEntry}
					<h3>Edit Name</h3>
					<form
						onsubmit={(e) => {
							e.preventDefault();
							saveEditedEntry();
						}}
					>
						<FormGroup label="Canonical Name">
							<input type="text" bind:value={editedCanonicalName} placeholder="e.g. John Smith" />
						</FormGroup>

						<FormGroup label="Type">
							<select bind:value={editedNameType}>
								{#each nameTypes as type (type.value)}
									<option value={type.value}>{type.label}</option>
								{/each}
							</select>
						</FormGroup>

						<FormGroup label="Aliases (comma-separated)">
							<input type="text" bind:value={editedAliases} placeholder="e.g. Johnny, J. Smith" />
						</FormGroup>

						<FormActions>
							<Button onclick={cancelEditingEntry}>Cancel</Button>
							<Button variant="primary" type="submit">Save Changes</Button>
						</FormActions>
					</form>
				{:else}
					<div class="detail-header">
						<h3>{selectedEntry.canonical_name}</h3>
						<span class="type-badge">
							{nameTypes.find((t) => t.value === selectedEntry.name_type)?.label ||
								selectedEntry.name_type}
						</span>
						{#if selectedEntry.is_confirmed}
							<span class="confirmed-badge">Confirmed</span>
						{:else}
							<span class="unconfirmed-detail-badge">Unconfirmed</span>
						{/if}
					</div>

					{#if selectedEntry.aliases}
						<div class="aliases-section">
							<strong>Aliases:</strong>
							<div class="alias-tags">
								{#each selectedEntry.aliases.split(',') as alias, idx (idx)}
									<span class="alias-tag">{alias.trim()}</span>
								{/each}
							</div>
						</div>
					{/if}

					{#if selectedEntry.bible_entry_id}
						<div class="bible-link">
							<strong>Bible Entry:</strong>
							<button
								class="link-btn"
								onclick={() => {
									if (selectedEntry?.bible_entry_id) {
										appState.selectedBibleEntryId = selectedEntry.bible_entry_id;
										appState.setViewMode('bible');
									}
								}}
							>
								{getBibleEntryName(selectedEntry.bible_entry_id)}
							</button>
						</div>
					{/if}

					<div class="actions">
						<Button size="sm" onclick={startEditingEntry}>Edit</Button>
						{#if !selectedEntry.is_confirmed}
							<Button size="sm" onclick={() => confirmEntry(selectedEntry!.id)}>Confirm</Button>
						{/if}
						{#if !selectedEntry.bible_entry_id}
							<Button size="sm" onclick={() => createBibleEntry(selectedEntry!)}>
								Create Bible Entry
							</Button>
						{/if}
						<Button
							size="sm"
							onclick={() => {
								mergeMode = true;
								mergeTargetId = selectedEntry!.id;
							}}
						>
							Merge Into This
						</Button>
						<Button size="sm" onclick={() => deleteEntry(selectedEntry!.id)}>Delete</Button>
					</div>
				{/if}

				<!-- Mentions section -->
				<div class="mentions-section">
					<h4>Mentions ({selectedMentions.length})</h4>
					{#if selectedMentions.length === 0}
						<p class="no-mentions">No mentions recorded.</p>
					{:else}
						<div class="mentions-list">
							{#each selectedMentions as mention (mention.id)}
								<div
									class="mention-item"
									class:mention-ignored={mention.status === 'ignored'}
									class:mention-accepted={mention.status === 'accepted'}
								>
									<div class="mention-info">
										<button class="link-btn" onclick={() => navigateToScene(mention.scene_id)}>
											{getSceneTitle(mention.scene_id)}
										</button>
										<span class="mention-text">"{mention.mention_text}"</span>
										<span class="mention-status status-{mention.status}">{mention.status}</span>
									</div>
									{#if mention.status === 'pending'}
										<div class="mention-actions">
											<button
												class="action-btn accept"
												onclick={() => updateMentionStatus(mention.id, 'accepted')}
												title="Accept"
											>
												&#10003;
											</button>
											<button
												class="action-btn ignore"
												onclick={() => updateMentionStatus(mention.id, 'ignored')}
												title="Ignore"
											>
												&#10005;
											</button>
										</div>
									{/if}
								</div>
							{/each}
						</div>
					{/if}
				</div>

				<div class="meta">
					<small>Created: {new Date(selectedEntry.created_at).toLocaleString()}</small>
					<small>Updated: {new Date(selectedEntry.updated_at).toLocaleString()}</small>
				</div>
			</div>
		{:else if similarGroups.length > 0}
			<!-- Show similar names overview when nothing is selected -->
			<div class="detail-content">
				<h3>Potential Issues</h3>
				<p class="issues-desc">
					The following groups of names look similar and may need attention:
				</p>
				{#each similarGroups as group, gIdx (gIdx)}
					<div class="similar-group">
						<span class="similarity-badge">{group.similarity}</span>
						<div class="similar-names">
							{#each group.entries as entry, j (entry.id)}
								{#if j > 0}<span class="similar-separator">&harr;</span>{/if}
								<button class="similar-name-btn" onclick={() => (selectedEntryId = entry.id)}>
									{entry.canonical_name}
								</button>
							{/each}
						</div>
						<div class="similar-actions">
							<Button
								size="sm"
								onclick={() => {
									mergeMode = true;
									mergeTargetId = group.entries[0].id;
								}}
							>
								Merge
							</Button>
							<Button
								size="sm"
								onclick={() => {
									for (const e of group.entries) confirmEntry(e.id);
								}}
							>
								Mark Intentional
							</Button>
						</div>
					</div>
				{/each}
			</div>
		{:else}
			<EmptyState title="Select a name to view details" />
		{/if}
	</div>
</div>

<style>
	.name-registry {
		display: flex;
		height: 100%;
		background-color: var(--color-bg-primary);
	}

	.registry-sidebar {
		width: 350px;
		border-right: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
	}

	.sidebar-header {
		padding: var(--spacing-md);
		border-bottom: 1px solid var(--color-border);
	}

	.sidebar-header h2 {
		margin: 0 0 var(--spacing-sm) 0;
		font-size: var(--font-size-lg);
	}

	.header-actions {
		display: flex;
		gap: var(--spacing-sm);
	}

	.scan-result {
		margin-top: var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		padding: var(--spacing-xs) var(--spacing-sm);
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
	}

	.filters {
		padding: var(--spacing-sm);
		border-bottom: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.search-input {
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
		font-size: var(--font-size-sm);
	}

	.filters select {
		padding: var(--spacing-xs);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
		font-size: var(--font-size-sm);
	}

	.similar-names-banner {
		padding: var(--spacing-sm) var(--spacing-md);
		background-color: var(--color-warning);
		color: white;
		font-size: var(--font-size-sm);
	}

	.banner-detail {
		display: block;
		font-size: var(--font-size-xs);
		opacity: 0.9;
	}

	.entries-list {
		flex: 1;
		overflow-y: auto;
	}

	.loading {
		padding: var(--spacing-lg);
		text-align: center;
		color: var(--color-text-secondary);
	}

	.entry-item {
		width: 100%;
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm) var(--spacing-md);
		border-bottom: 1px solid var(--color-border);
		text-align: left;
		background: none;
		cursor: pointer;
		transition: background-color var(--transition-fast);
	}

	.entry-item:hover {
		background-color: var(--color-bg-hover);
	}

	.entry-item.selected {
		background-color: var(--color-accent-light);
	}

	.entry-item.unconfirmed {
		border-left: 3px solid var(--color-warning);
	}

	.entry-item.merge-target {
		border-left: 3px solid var(--color-accent);
		cursor: crosshair;
	}

	.entry-content {
		flex: 1;
		min-width: 0;
	}

	.entry-name {
		display: block;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.entry-meta {
		display: block;
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
	}

	.unconfirmed-badge {
		color: var(--color-warning);
		font-weight: 500;
	}

	.bible-link-indicator {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background-color: var(--color-accent);
		color: white;
		font-size: var(--font-size-xs);
		font-weight: 700;
		flex-shrink: 0;
	}

	.registry-detail {
		flex: 1;
		overflow-y: auto;
	}

	.detail-content {
		padding: var(--spacing-lg);
		max-width: 600px;
	}

	.detail-header {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		margin-bottom: var(--spacing-md);
	}

	.detail-header h3 {
		margin: 0;
		font-size: var(--font-size-xl);
	}

	.type-badge {
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		font-weight: 500;
		background-color: var(--color-bg-tertiary);
		color: var(--color-text-secondary);
	}

	.confirmed-badge {
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		font-weight: 500;
		background-color: var(--color-success, #22c55e);
		color: white;
	}

	.unconfirmed-detail-badge {
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		font-weight: 500;
		background-color: var(--color-warning);
		color: white;
	}

	.aliases-section {
		margin-bottom: var(--spacing-md);
	}

	.alias-tags {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-xs);
		margin-top: var(--spacing-xs);
	}

	.alias-tag {
		padding: 2px var(--spacing-sm);
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
	}

	.bible-link {
		margin-bottom: var(--spacing-md);
	}

	.link-btn {
		color: var(--color-accent);
		background: none;
		border: none;
		padding: 0;
		cursor: pointer;
		text-decoration: underline;
	}

	.link-btn:hover {
		color: var(--color-accent-hover);
	}

	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-sm);
		margin: var(--spacing-lg) 0;
		padding-bottom: var(--spacing-md);
		border-bottom: 1px solid var(--color-border);
	}

	.mentions-section {
		margin-top: var(--spacing-md);
	}

	.mentions-section h4 {
		margin: 0 0 var(--spacing-sm) 0;
	}

	.no-mentions {
		color: var(--color-text-secondary);
		font-style: italic;
	}

	.mentions-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.mention-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
	}

	.mention-item.mention-ignored {
		opacity: 0.5;
	}

	.mention-item.mention-accepted {
		border-left: 3px solid var(--color-success, #22c55e);
	}

	.mention-info {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		min-width: 0;
		flex: 1;
	}

	.mention-text {
		color: var(--color-text-secondary);
		font-style: italic;
		font-size: var(--font-size-sm);
	}

	.mention-status {
		font-size: var(--font-size-xs);
		padding: 1px var(--spacing-xs);
		border-radius: var(--border-radius-sm);
		font-weight: 500;
	}

	.mention-status.status-pending {
		background-color: var(--color-warning);
		color: white;
	}

	.mention-status.status-accepted {
		background-color: var(--color-success, #22c55e);
		color: white;
	}

	.mention-status.status-ignored {
		background-color: var(--color-bg-tertiary);
		color: var(--color-text-secondary);
	}

	.mention-actions {
		display: flex;
		gap: var(--spacing-xs);
	}

	.action-btn {
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		border-radius: var(--border-radius-sm);
		cursor: pointer;
		font-size: var(--font-size-sm);
		background: none;
	}

	.action-btn.accept:hover {
		background-color: var(--color-success, #22c55e);
		color: white;
	}

	.action-btn.ignore:hover {
		background-color: var(--color-error);
		color: white;
	}

	.merge-info {
		color: var(--color-text-secondary);
		font-size: var(--font-size-sm);
		margin-bottom: var(--spacing-md);
	}

	.meta {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
		color: var(--color-text-tertiary);
		border-top: 1px solid var(--color-border);
		padding-top: var(--spacing-md);
		margin-top: var(--spacing-lg);
	}

	/* Similar names / issues overview */
	.issues-desc {
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-md);
	}

	.similar-group {
		padding: var(--spacing-md);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		margin-bottom: var(--spacing-sm);
	}

	.similarity-badge {
		display: inline-block;
		font-size: var(--font-size-xs);
		padding: 2px var(--spacing-xs);
		background-color: var(--color-warning);
		color: white;
		border-radius: var(--border-radius-sm);
		margin-bottom: var(--spacing-sm);
		font-weight: 500;
	}

	.similar-names {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		margin-bottom: var(--spacing-sm);
		flex-wrap: wrap;
	}

	.similar-separator {
		color: var(--color-text-secondary);
	}

	.similar-name-btn {
		padding: var(--spacing-xs) var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		cursor: pointer;
		font-weight: 500;
	}

	.similar-name-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.similar-actions {
		display: flex;
		gap: var(--spacing-sm);
	}

	/* Form styles */
	.detail-content form {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
	}

	.detail-content input,
	.detail-content select {
		width: 100%;
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
		font-size: var(--font-size-base);
	}
</style>
