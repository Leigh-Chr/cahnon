<!--
  Consistency issues and problems view.

  Features:
  - List all issues with filtering by type, status, severity
  - Create manual issues (bible contradiction, continuity error)
  - Resolve or ignore issues
  - Navigate to affected scenes/bible entries
  - Auto-detect timeline conflicts
-->
<script lang="ts">
	import { appState } from '$lib/stores';
	import { issueApi, timelineApi, type Issue } from '$lib/api';
	import { Button, FormGroup, FormActions, EmptyState } from './ui';

	let issues = $state<Issue[]>([]);
	let isLoading = $state(false);

	// Filters
	let filterType = $state<string | null>(null);
	let filterStatus = $state<string | null>(null);
	let filterSeverity = $state<string | null>(null);

	// Create issue form
	let showCreateForm = $state(false);
	let newIssueType = $state('continuity_error');
	let newIssueTitle = $state('');
	let newIssueDescription = $state('');
	let newIssueSeverity = $state('warning');

	// Selected issue
	let selectedIssueId = $state<string | null>(null);
	let linkedSceneIds = $state<string[]>([]);
	let linkedBibleIds = $state<string[]>([]);

	const issueTypes = [
		{ value: 'timeline_conflict', label: 'Timeline Conflict', icon: 'clock', auto: true },
		{ value: 'tbd_in_done', label: 'TBD in Done Scene', icon: 'alert-circle', auto: true },
		{ value: 'orphan_mention', label: 'Orphan Mention', icon: 'user-x', auto: true },
		{ value: 'bible_contradiction', label: 'Bible Contradiction', icon: 'book-x', auto: false },
		{ value: 'continuity_error', label: 'Continuity Error', icon: 'link-2-off', auto: false },
	];

	const severities = [
		{ value: 'info', label: 'Info', color: 'var(--color-info)' },
		{ value: 'warning', label: 'Warning', color: 'var(--color-warning)' },
		{ value: 'error', label: 'Error', color: 'var(--color-error)' },
	];

	const statuses = [
		{ value: 'open', label: 'Open' },
		{ value: 'resolved', label: 'Resolved' },
		{ value: 'ignored', label: 'Ignored' },
	];

	let selectedIssue = $derived(
		selectedIssueId ? issues.find((i) => i.id === selectedIssueId) || null : null
	);

	let filteredIssues = $derived(() => {
		let result = issues;
		if (filterType) result = result.filter((i) => i.issue_type === filterType);
		if (filterStatus) result = result.filter((i) => i.status === filterStatus);
		if (filterSeverity) result = result.filter((i) => i.severity === filterSeverity);
		return result;
	});

	$effect(() => {
		if (appState.project) {
			loadIssues();
		}
	});

	$effect(() => {
		if (selectedIssueId) {
			loadLinkedItems(selectedIssueId);
		}
	});

	async function loadIssues() {
		isLoading = true;
		try {
			issues = await issueApi.getAll();
		} catch (e) {
			console.error('Failed to load issues:', e);
		} finally {
			isLoading = false;
		}
	}

	async function loadLinkedItems(issueId: string) {
		try {
			linkedSceneIds = await issueApi.getIssueScenes(issueId);
			linkedBibleIds = await issueApi.getIssueBibleEntries(issueId);
		} catch (e) {
			console.error('Failed to load linked items:', e);
		}
	}

	async function detectTimelineConflicts() {
		isLoading = true;
		try {
			const conflicts = await timelineApi.detectConflicts();
			for (const conflict of conflicts) {
				// Check if issue already exists
				const existingIssue = issues.find(
					(i) =>
						i.issue_type === 'timeline_conflict' && i.title.includes(conflict.character_name || '')
				);
				if (!existingIssue) {
					await issueApi.create({
						issue_type: 'timeline_conflict',
						title: `Timeline conflict: ${conflict.character_name || 'Unknown'}`,
						description: conflict.description,
						severity: 'error',
					});
				}
			}
			await loadIssues();
		} catch (e) {
			console.error('Failed to detect conflicts:', e);
		} finally {
			isLoading = false;
		}
	}

	async function createIssue() {
		if (!newIssueTitle.trim()) return;

		try {
			const issue = await issueApi.create({
				issue_type: newIssueType,
				title: newIssueTitle.trim(),
				description: newIssueDescription.trim() || undefined,
				severity: newIssueSeverity,
			});
			issues = [...issues, issue];
			showCreateForm = false;
			newIssueTitle = '';
			newIssueDescription = '';
			selectedIssueId = issue.id;
		} catch (e) {
			console.error('Failed to create issue:', e);
		}
	}

	async function updateIssueStatus(status: string, resolutionNote?: string) {
		if (!selectedIssue) return;

		try {
			const updated = await issueApi.update(selectedIssue.id, {
				status,
				resolution_note: resolutionNote,
			});
			issues = issues.map((i) => (i.id === updated.id ? updated : i));
		} catch (e) {
			console.error('Failed to update issue:', e);
		}
	}

	function getTypeInfo(type: string) {
		return issueTypes.find((t) => t.value === type) || { value: type, label: type, icon: 'help' };
	}

	function getSeverityInfo(severity: string) {
		return (
			severities.find((s) => s.value === severity) || {
				value: severity,
				label: severity,
				color: 'var(--color-text-secondary)',
			}
		);
	}

	function getStatusInfo(status: string) {
		return statuses.find((s) => s.value === status) || { value: status, label: status };
	}

	function navigateToScene(sceneId: string) {
		// Find the chapter containing this scene
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

	function navigateToBibleEntry(entryId: string) {
		appState.selectedBibleEntryId = entryId;
		appState.setViewMode('bible');
	}

	function getSceneTitle(sceneId: string): string {
		for (const [, scenes] of appState.scenes) {
			const scene = scenes.find((s) => s.id === sceneId);
			if (scene) return scene.title;
		}
		return 'Unknown scene';
	}

	function getBibleEntryName(entryId: string): string {
		const entry = appState.bibleEntries.find((e) => e.id === entryId);
		return entry?.name || 'Unknown entry';
	}
</script>

<div class="issues-view">
	<div class="issues-sidebar">
		<div class="sidebar-header">
			<h2>Issues</h2>
			<div class="header-actions">
				<Button size="sm" onclick={() => detectTimelineConflicts()} disabled={isLoading}>
					Detect Conflicts
				</Button>
				<Button size="sm" variant="primary" onclick={() => (showCreateForm = true)}>
					+ New Issue
				</Button>
			</div>
		</div>

		<div class="filters">
			<select bind:value={filterType}>
				<option value={null}>All Types</option>
				{#each issueTypes as type}
					<option value={type.value}>{type.label}</option>
				{/each}
			</select>

			<select bind:value={filterStatus}>
				<option value={null}>All Statuses</option>
				{#each statuses as status}
					<option value={status.value}>{status.label}</option>
				{/each}
			</select>

			<select bind:value={filterSeverity}>
				<option value={null}>All Severities</option>
				{#each severities as severity}
					<option value={severity.value}>{severity.label}</option>
				{/each}
			</select>
		</div>

		<div class="issues-list">
			{#if isLoading}
				<div class="loading">Loading issues...</div>
			{:else if filteredIssues().length === 0}
				<EmptyState title="No issues found" />
			{:else}
				{#each filteredIssues() as issue (issue.id)}
					<button
						class="issue-item"
						class:selected={selectedIssueId === issue.id}
						class:resolved={issue.status === 'resolved'}
						class:ignored={issue.status === 'ignored'}
						onclick={() => (selectedIssueId = issue.id)}
					>
						<span
							class="severity-indicator"
							style="background-color: {getSeverityInfo(issue.severity).color}"
						></span>
						<div class="issue-content">
							<span class="issue-title">{issue.title}</span>
							<span class="issue-meta">
								{getTypeInfo(issue.issue_type).label} &bull; {getStatusInfo(issue.status).label}
							</span>
						</div>
					</button>
				{/each}
			{/if}
		</div>
	</div>

	<div class="issues-detail">
		{#if showCreateForm}
			<div class="detail-content">
				<h3>Create New Issue</h3>
				<form
					onsubmit={(e) => {
						e.preventDefault();
						createIssue();
					}}
				>
					<FormGroup label="Type">
						<select bind:value={newIssueType}>
							{#each issueTypes.filter((t) => !t.auto) as type}
								<option value={type.value}>{type.label}</option>
							{/each}
						</select>
					</FormGroup>

					<FormGroup label="Title">
						<input
							type="text"
							bind:value={newIssueTitle}
							placeholder="Brief description of the issue"
						/>
					</FormGroup>

					<FormGroup label="Description">
						<textarea
							bind:value={newIssueDescription}
							rows="4"
							placeholder="Detailed explanation..."
						></textarea>
					</FormGroup>

					<FormGroup label="Severity">
						<select bind:value={newIssueSeverity}>
							{#each severities as severity}
								<option value={severity.value}>{severity.label}</option>
							{/each}
						</select>
					</FormGroup>

					<FormActions>
						<Button onclick={() => (showCreateForm = false)}>Cancel</Button>
						<Button variant="primary" type="submit">Create Issue</Button>
					</FormActions>
				</form>
			</div>
		{:else if selectedIssue}
			<div class="detail-content">
				<div class="detail-header">
					<span
						class="severity-badge"
						style="background-color: {getSeverityInfo(selectedIssue.severity).color}"
					>
						{getSeverityInfo(selectedIssue.severity).label}
					</span>
					<span class="type-badge">{getTypeInfo(selectedIssue.issue_type).label}</span>
				</div>

				<h3>{selectedIssue.title}</h3>

				{#if selectedIssue.description}
					<p class="description">{selectedIssue.description}</p>
				{/if}

				<div class="status-section">
					<strong>Status:</strong>
					{getStatusInfo(selectedIssue.status).label}
					{#if selectedIssue.resolution_note}
						<p class="resolution-note">{selectedIssue.resolution_note}</p>
					{/if}
				</div>

				{#if linkedSceneIds.length > 0}
					<div class="linked-items">
						<strong>Linked Scenes:</strong>
						<ul>
							{#each linkedSceneIds as sceneId}
								<li>
									<button class="link-btn" onclick={() => navigateToScene(sceneId)}>
										{getSceneTitle(sceneId)}
									</button>
								</li>
							{/each}
						</ul>
					</div>
				{/if}

				{#if linkedBibleIds.length > 0}
					<div class="linked-items">
						<strong>Linked Bible Entries:</strong>
						<ul>
							{#each linkedBibleIds as entryId}
								<li>
									<button class="link-btn" onclick={() => navigateToBibleEntry(entryId)}>
										{getBibleEntryName(entryId)}
									</button>
								</li>
							{/each}
						</ul>
					</div>
				{/if}

				{#if selectedIssue.status === 'open'}
					<div class="actions">
						<Button onclick={() => updateIssueStatus('resolved', 'Marked as resolved')}>
							Mark Resolved
						</Button>
						<Button onclick={() => updateIssueStatus('ignored', 'Ignored by user')}>Ignore</Button>
					</div>
				{:else}
					<div class="actions">
						<Button onclick={() => updateIssueStatus('open')}>Reopen</Button>
					</div>
				{/if}

				<div class="meta">
					<small>Created: {new Date(selectedIssue.created_at).toLocaleString()}</small>
					<small>Updated: {new Date(selectedIssue.updated_at).toLocaleString()}</small>
				</div>
			</div>
		{:else}
			<EmptyState title="Select an issue to view details" />
		{/if}
	</div>
</div>

<style>
	.issues-view {
		display: flex;
		height: 100%;
		background-color: var(--color-bg-primary);
	}

	.issues-sidebar {
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

	.filters {
		padding: var(--spacing-sm);
		border-bottom: 1px solid var(--color-border);
		display: flex;
		gap: var(--spacing-xs);
	}

	.filters select {
		flex: 1;
		padding: var(--spacing-xs);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
		font-size: var(--font-size-sm);
	}

	.issues-list {
		flex: 1;
		overflow-y: auto;
	}

	.loading {
		padding: var(--spacing-lg);
		text-align: center;
		color: var(--color-text-secondary);
	}

	.issue-item {
		width: 100%;
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm) var(--spacing-md);
		border-bottom: 1px solid var(--color-border);
		text-align: left;
		background: none;
		cursor: pointer;
		transition: background-color var(--transition-fast);
	}

	.issue-item:hover {
		background-color: var(--color-bg-hover);
	}

	.issue-item.selected {
		background-color: var(--color-accent-light);
	}

	.issue-item.resolved,
	.issue-item.ignored {
		opacity: 0.6;
	}

	.severity-indicator {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
		margin-top: 6px;
	}

	.issue-content {
		flex: 1;
		min-width: 0;
	}

	.issue-title {
		display: block;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.issue-meta {
		display: block;
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
	}

	.issues-detail {
		flex: 1;
		overflow-y: auto;
	}

	.detail-content {
		padding: var(--spacing-lg);
		max-width: 600px;
	}

	.detail-header {
		display: flex;
		gap: var(--spacing-sm);
		margin-bottom: var(--spacing-md);
	}

	.severity-badge,
	.type-badge {
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		font-weight: 500;
	}

	.severity-badge {
		color: white;
	}

	.type-badge {
		background-color: var(--color-bg-tertiary);
		color: var(--color-text-secondary);
	}

	.detail-content h3 {
		margin: 0 0 var(--spacing-md) 0;
		font-size: var(--font-size-xl);
	}

	.description {
		color: var(--color-text-secondary);
		line-height: 1.6;
		margin-bottom: var(--spacing-lg);
	}

	.status-section {
		margin-bottom: var(--spacing-lg);
	}

	.resolution-note {
		margin-top: var(--spacing-xs);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-sm);
		font-style: italic;
	}

	.linked-items {
		margin-bottom: var(--spacing-md);
	}

	.linked-items ul {
		list-style: none;
		padding: 0;
		margin: var(--spacing-xs) 0 0 0;
	}

	.linked-items li {
		margin-bottom: var(--spacing-xs);
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
		gap: var(--spacing-sm);
		margin: var(--spacing-lg) 0;
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

	/* Form styles */
	.detail-content form {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
	}

	.detail-content input,
	.detail-content select,
	.detail-content textarea {
		width: 100%;
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
		font-size: var(--font-size-base);
	}

	.detail-content textarea {
		resize: vertical;
	}
</style>
