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
	import { slide } from 'svelte/transition';

	import { type BibleEntry, type Issue, issueApi, type Scene, timelineApi } from '$lib/api';
	import { appState } from '$lib/stores';
	import { showError } from '$lib/toast';
	import { formatDateTime } from '$lib/utils';
	import { nativeConfirm } from '$lib/utils/native-dialog';

	import { Button, EmptyState, FormActions, FormGroup, LoadingState } from './ui';
	import ContextMenu from './ui/ContextMenu.svelte';
	import ContextMenuItem from './ui/ContextMenuItem.svelte';
	import ContextMenuSeparator from './ui/ContextMenuSeparator.svelte';

	// Context menu for issues
	let issueContextMenu = $state<{ x: number; y: number; issueId: string } | null>(null);

	function handleIssueContextMenu(event: MouseEvent, issueId: string) {
		event.preventDefault();
		issueContextMenu = { x: event.clientX, y: event.clientY, issueId };
	}

	function closeIssueContextMenu() {
		issueContextMenu = null;
	}

	let issues = $state<Issue[]>([]);
	let isLoading = $state(false);

	// Filters (empty string = no filter; using string instead of null for HTML select compatibility)
	let filterType = $state('');
	let filterStatus = $state('');
	let filterSeverity = $state('');

	// Create issue form
	let showCreateForm = $state(false);
	let newIssueType = $state('continuity_error');
	let newIssueTitle = $state('');
	let newIssueDescription = $state('');
	let newIssueSeverity = $state('warning');

	// Selected issue
	let selectedIssueId = $state<string | null>(null);
	let linkedScenes = $state<Scene[]>([]);
	let linkedBibleEntries = $state<BibleEntry[]>([]);

	// Edit mode
	let isEditing = $state(false);
	let editedTitle = $state('');
	let editedDescription = $state('');
	let editedSeverity = $state('warning');

	// Linking
	let isAddingScene = $state(false);
	let selectedSceneToLink = $state('');
	let isAddingBibleEntry = $state(false);
	let bibleSearchQuery = $state('');

	let allScenes = $derived.by(() => {
		const result: Array<{ id: string; title: string }> = [];
		for (const chapter of appState.chapters) {
			const chapterScenes = appState.scenes.get(chapter.id) || [];
			for (const scene of chapterScenes) {
				result.push({ id: scene.id, title: scene.title });
			}
		}
		return result;
	});

	let linkedSceneIds = $derived(linkedScenes.map((s) => s.id));
	let linkedBibleIds = $derived(linkedBibleEntries.map((e) => e.id));

	let availableScenesToLink = $derived(allScenes.filter((s) => !linkedSceneIds.includes(s.id)));

	let filteredBibleEntriesToLink = $derived(
		bibleSearchQuery
			? appState.bibleEntries
					.filter(
						(e) =>
							!linkedBibleIds.includes(e.id) &&
							(e.name.toLowerCase().includes(bibleSearchQuery.toLowerCase()) ||
								(e.aliases && e.aliases.toLowerCase().includes(bibleSearchQuery.toLowerCase())))
					)
					.slice(0, 10)
			: []
	);

	const issueTypes = [
		{ value: 'timeline_conflict', label: 'Timeline Conflict', icon: 'clock', auto: true },
		{ value: 'tbd_in_done', label: 'TBD in Done Scene', icon: 'alert-circle', auto: true },
		{ value: 'orphan_mention', label: 'Orphan Mention', icon: 'user-x', auto: true },
		{ value: 'bible_contradiction', label: 'Bible Contradiction', icon: 'book-x', auto: false },
		{ value: 'continuity_error', label: 'Continuity Error', icon: 'link-2-off', auto: false },
		{ value: 'plot_hole', label: 'Plot Hole', icon: 'alert-triangle', auto: false },
		{ value: 'research_needed', label: 'Research Needed', icon: 'search', auto: false },
	];

	// CD5: Issue templates
	const issueTemplates = [
		{
			name: 'Timeline Inconsistency',
			type: 'timeline_conflict',
			severity: 'warning',
			description: 'Events occur in an impossible order. Scene X references Y before it happens.',
		},
		{
			name: 'Character Contradiction',
			type: 'bible_contradiction',
			severity: 'warning',
			description: 'Character behavior/appearance contradicts established facts.',
		},
		{
			name: 'Plot Hole',
			type: 'plot_hole',
			severity: 'error',
			description: 'Logical gap in the story that needs resolution.',
		},
		{
			name: 'Research Needed',
			type: 'research_needed',
			severity: 'info',
			description: 'Factual accuracy needs verification.',
		},
	];

	function applyTemplate(template: (typeof issueTemplates)[0]) {
		newIssueType = template.type;
		newIssueSeverity = template.severity;
		newIssueTitle = template.name;
		newIssueDescription = template.description;
		showCreateForm = true;
	}

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

	let filteredIssues = $derived.by(() => {
		let result = issues;
		// In writing mode, only show critical (error) issues per spec 14.4
		if (appState.workMode === 'writing') {
			result = result.filter((i) => i.severity === 'error');
		}
		if (filterType) result = result.filter((i) => i.issue_type === filterType);
		if (filterStatus) result = result.filter((i) => i.status === filterStatus);
		if (filterSeverity) result = result.filter((i) => i.severity === filterSeverity);
		return result;
	});

	// CD6: Dashboard metrics
	let issueStats = $derived.by(() => {
		const stats = {
			total: issues.length,
			open: 0,
			resolved: 0,
			ignored: 0,
			critical: 0,
			warnings: 0,
			info: 0,
		};

		for (const issue of issues) {
			// By status
			if (issue.status === 'open') stats.open++;
			else if (issue.status === 'resolved') stats.resolved++;
			else if (issue.status === 'ignored') stats.ignored++;

			// By severity (only count open issues)
			if (issue.status === 'open') {
				if (issue.severity === 'error') stats.critical++;
				else if (issue.severity === 'warning') stats.warnings++;
				else stats.info++;
			}
		}

		return stats;
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
			linkedScenes = await issueApi.getIssueScenes(issueId);
			linkedBibleEntries = await issueApi.getIssueBibleEntries(issueId);
		} catch (e) {
			console.error('Failed to load linked items:', e);
		}
	}

	async function runProjectAnalysis() {
		isLoading = true;
		try {
			const detected = await issueApi.runDetections();
			issues = await issueApi.getAll();
			if (detected.length > 0) {
				selectedIssueId = detected[0].id;
			}
		} catch (e) {
			console.error('Failed to run detections:', e);
			showError('Failed to analyze project');
		} finally {
			isLoading = false;
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
			showError('Failed to detect timeline conflicts');
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
			showError('Failed to create issue');
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
			showError('Failed to update issue');
		}
	}

	async function deleteIssue() {
		if (!selectedIssue) return;
		const issueId = selectedIssue.id;

		try {
			await issueApi.delete(issueId);
			issues = issues.filter((i) => i.id !== issueId);
			selectedIssueId = null;
		} catch (e) {
			console.error('Failed to delete issue:', e);
			showError('Failed to delete issue');
		}
	}

	function startEditing() {
		if (!selectedIssue) return;
		editedTitle = selectedIssue.title;
		editedDescription = selectedIssue.description || '';
		editedSeverity = selectedIssue.severity;
		isEditing = true;
	}

	function cancelEditing() {
		isEditing = false;
	}

	async function saveEdit() {
		if (!selectedIssue || !editedTitle.trim()) return;
		try {
			const updated = await issueApi.update(selectedIssue.id, {
				title: editedTitle.trim(),
				description: editedDescription.trim() || undefined,
				severity: editedSeverity,
			});
			issues = issues.map((i) => (i.id === updated.id ? updated : i));
			isEditing = false;
		} catch (e) {
			console.error('Failed to update issue:', e);
			showError('Failed to update issue');
		}
	}

	async function linkSceneToIssue() {
		if (!selectedIssue || !selectedSceneToLink) return;
		try {
			await issueApi.linkScene(selectedSceneToLink, selectedIssue.id);
			await loadLinkedItems(selectedIssue.id);
			selectedSceneToLink = '';
			isAddingScene = false;
		} catch (e) {
			console.error('Failed to link scene:', e);
			showError('Failed to link scene');
		}
	}

	async function unlinkSceneFromIssue(sceneId: string) {
		if (!selectedIssue) return;
		try {
			await issueApi.unlinkScene(sceneId, selectedIssue.id);
			await loadLinkedItems(selectedIssue.id);
		} catch (e) {
			console.error('Failed to unlink scene:', e);
			showError('Failed to unlink scene');
		}
	}

	async function linkBibleEntryToIssue(entryId: string) {
		if (!selectedIssue) return;
		try {
			await issueApi.linkBibleEntry(entryId, selectedIssue.id);
			await loadLinkedItems(selectedIssue.id);
			bibleSearchQuery = '';
			isAddingBibleEntry = false;
		} catch (e) {
			console.error('Failed to link bible entry:', e);
			showError('Failed to link bible entry');
		}
	}

	async function unlinkBibleEntryFromIssue(entryId: string) {
		if (!selectedIssue) return;
		try {
			await issueApi.unlinkBibleEntry(entryId, selectedIssue.id);
			await loadLinkedItems(selectedIssue.id);
		} catch (e) {
			console.error('Failed to unlink bible entry:', e);
			showError('Failed to unlink bible entry');
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
</script>

<div class="issues-view">
	<div class="issues-sidebar">
		<div class="sidebar-header">
			<h2>Issues</h2>
			{#if appState.workMode === 'writing'}
				<span class="mode-badge">Critical only</span>
			{/if}
			<div class="header-actions">
				<Button size="sm" onclick={runProjectAnalysis} disabled={isLoading}>Analyze Project</Button>
				<Button size="sm" onclick={() => detectTimelineConflicts()} disabled={isLoading}>
					Detect Conflicts
				</Button>
				<Button size="sm" variant="primary" onclick={() => (showCreateForm = true)}>
					+ New Issue
				</Button>
			</div>
		</div>

		<!-- CD6: Dashboard metrics -->
		<div class="issues-dashboard">
			<div class="stat-card" class:has-issues={issueStats.critical > 0}>
				<span class="stat-value stat-critical">{issueStats.critical}</span>
				<span class="stat-label">Critical</span>
			</div>
			<div class="stat-card" class:has-issues={issueStats.warnings > 0}>
				<span class="stat-value stat-warning">{issueStats.warnings}</span>
				<span class="stat-label">Warnings</span>
			</div>
			<div class="stat-card">
				<span class="stat-value stat-info">{issueStats.info}</span>
				<span class="stat-label">Info</span>
			</div>
			<div class="stat-card">
				<span class="stat-value stat-resolved">{issueStats.resolved}</span>
				<span class="stat-label">Resolved</span>
			</div>
		</div>

		<!-- CD5: Quick templates -->
		<div class="quick-templates">
			<span class="templates-label">Quick add:</span>
			{#each issueTemplates as template (template.name)}
				<button class="template-chip" onclick={() => applyTemplate(template)}>
					{template.name}
				</button>
			{/each}
		</div>

		<div class="filters">
			<select bind:value={filterType}>
				<option value="">All Types</option>
				{#each issueTypes as type (type.value)}
					<option value={type.value}>{type.label}</option>
				{/each}
			</select>

			<select bind:value={filterStatus}>
				<option value="">All Statuses</option>
				{#each statuses as status (status.value)}
					<option value={status.value}>{status.label}</option>
				{/each}
			</select>

			<select bind:value={filterSeverity}>
				<option value="">All Severities</option>
				{#each severities as severity (severity.value)}
					<option value={severity.value}>{severity.label}</option>
				{/each}
			</select>
		</div>

		<div class="issues-list">
			{#if isLoading}
				<LoadingState message="Loading issues..." />
			{:else if filteredIssues.length === 0}
				<EmptyState
					icon="check"
					title="No issues found"
					description="Your manuscript is looking good! Issues will appear here when continuity checks detect problems."
				/>
			{:else}
				{#each filteredIssues as issue (issue.id)}
					<button
						class="issue-item"
						class:selected={selectedIssueId === issue.id}
						class:resolved={issue.status === 'resolved'}
						class:ignored={issue.status === 'ignored'}
						onclick={() => (selectedIssueId = issue.id)}
						oncontextmenu={(e) => handleIssueContextMenu(e, issue.id)}
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
							{#each issueTypes.filter((t) => !t.auto) as type (type.value)}
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
							{#each severities as severity (severity.value)}
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
				{#if isEditing}
					<h3>Edit Issue</h3>
					<form
						onsubmit={(e) => {
							e.preventDefault();
							saveEdit();
						}}
					>
						<FormGroup label="Title">
							<input type="text" bind:value={editedTitle} placeholder="Issue title" />
						</FormGroup>

						<FormGroup label="Description">
							<textarea
								bind:value={editedDescription}
								rows="4"
								placeholder="Detailed explanation..."
							></textarea>
						</FormGroup>

						<FormGroup label="Severity">
							<select bind:value={editedSeverity}>
								{#each severities as severity (severity.value)}
									<option value={severity.value}>{severity.label}</option>
								{/each}
							</select>
						</FormGroup>

						<FormActions>
							<Button onclick={cancelEditing}>Cancel</Button>
							<Button variant="primary" type="submit">Save Changes</Button>
						</FormActions>
					</form>
				{:else}
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

					<div class="linked-items">
						<div class="linked-header">
							<strong>Linked Scenes:</strong>
							<button class="add-link-btn" onclick={() => (isAddingScene = !isAddingScene)}>
								{isAddingScene ? '×' : '+'}
							</button>
						</div>
						{#if isAddingScene}
							<div transition:slide={{ duration: 150 }}>
								<select
									class="link-select"
									bind:value={selectedSceneToLink}
									onchange={linkSceneToIssue}
								>
									<option value="">Select a scene...</option>
									{#each availableScenesToLink as scene (scene.id)}
										<option value={scene.id}>{scene.title}</option>
									{/each}
								</select>
							</div>
						{/if}
						<ul>
							{#each linkedScenes as scene (scene.id)}
								<li>
									<button class="link-btn" onclick={() => navigateToScene(scene.id)}>
										{scene.title}
									</button>
									<button
										class="unlink-btn"
										onclick={() => unlinkSceneFromIssue(scene.id)}
										title="Unlink">&times;</button
									>
								</li>
							{:else}
								<li class="empty-link">No scenes linked.</li>
							{/each}
						</ul>
					</div>

					<div class="linked-items">
						<div class="linked-header">
							<strong>Linked Bible Entries:</strong>
							<button
								class="add-link-btn"
								onclick={() => (isAddingBibleEntry = !isAddingBibleEntry)}
							>
								{isAddingBibleEntry ? '×' : '+'}
							</button>
						</div>
						{#if isAddingBibleEntry}
							<div transition:slide={{ duration: 150 }}>
								<input
									type="text"
									class="link-select"
									placeholder="Search bible entries..."
									bind:value={bibleSearchQuery}
								/>
								{#if filteredBibleEntriesToLink.length > 0}
									<div class="link-search-results">
										{#each filteredBibleEntriesToLink as entry (entry.id)}
											<button
												class="link-search-result"
												onclick={() => linkBibleEntryToIssue(entry.id)}
											>
												{entry.name}
											</button>
										{/each}
									</div>
								{/if}
							</div>
						{/if}
						<ul>
							{#each linkedBibleEntries as entry (entry.id)}
								<li>
									<button class="link-btn" onclick={() => navigateToBibleEntry(entry.id)}>
										{entry.name}
									</button>
									<button
										class="unlink-btn"
										onclick={() => unlinkBibleEntryFromIssue(entry.id)}
										title="Unlink">&times;</button
									>
								</li>
							{:else}
								<li class="empty-link">No bible entries linked.</li>
							{/each}
						</ul>
					</div>

					{#if selectedIssue.status === 'open'}
						<div class="actions">
							<Button onclick={startEditing}>Edit</Button>
							<Button onclick={() => updateIssueStatus('resolved', 'Marked as resolved')}>
								Mark Resolved
							</Button>
							<Button onclick={() => updateIssueStatus('ignored', 'Ignored by user')}>Ignore</Button
							>
							<Button onclick={deleteIssue} variant="danger">Delete</Button>
						</div>
					{:else}
						<div class="actions">
							<Button onclick={startEditing}>Edit</Button>
							<Button onclick={() => updateIssueStatus('open')}>Reopen</Button>
							<Button onclick={deleteIssue} variant="danger">Delete</Button>
						</div>
					{/if}

					<div class="meta">
						<small>Created: {formatDateTime(selectedIssue.created_at)}</small>
						<small>Updated: {formatDateTime(selectedIssue.updated_at)}</small>
					</div>
				{/if}
			</div>
		{:else}
			<EmptyState
				icon="info"
				title="Select an issue to view details"
				description="Click on an issue from the list to see its details and linked scenes."
			/>
		{/if}
	</div>
</div>

{#if issueContextMenu}
	{@const issue = issues.find((i) => i.id === issueContextMenu!.issueId)}
	<ContextMenu x={issueContextMenu.x} y={issueContextMenu.y} onclose={closeIssueContextMenu}>
		<ContextMenuItem
			label="View Details"
			onclick={() => {
				selectedIssueId = issueContextMenu!.issueId;
				closeIssueContextMenu();
			}}
		/>
		{#if issue && issue.status === 'open'}
			<ContextMenuItem
				label="Resolve"
				onclick={async () => {
					await issueApi.update(issueContextMenu!.issueId, { status: 'resolved' });
					await loadIssues();
					closeIssueContextMenu();
				}}
			/>
		{/if}
		<ContextMenuSeparator />
		<ContextMenuItem
			label="Delete"
			danger
			onclick={async () => {
				if (
					await nativeConfirm('Delete this issue? This action cannot be undone.', 'Delete Issue')
				) {
					await issueApi.delete(issueContextMenu!.issueId);
					await loadIssues();
				}
				closeIssueContextMenu();
			}}
		/>
	</ContextMenu>
{/if}

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

	.mode-badge {
		display: inline-block;
		font-size: var(--font-size-xs);
		padding: 2px var(--spacing-xs);
		background-color: var(--color-warning);
		color: white;
		border-radius: var(--border-radius-sm);
		margin-bottom: var(--spacing-sm);
		font-weight: 500;
	}

	.header-actions {
		display: flex;
		gap: var(--spacing-sm);
	}

	/* CD6: Dashboard metrics */
	.issues-dashboard {
		display: flex;
		gap: var(--spacing-xs);
		padding: var(--spacing-sm);
		border-bottom: 1px solid var(--color-border);
	}

	.stat-card {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: var(--spacing-xs);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-tertiary);
	}

	.stat-card.has-issues {
		background-color: var(--color-error-light, rgba(239, 68, 68, 0.1));
	}

	.stat-value {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.stat-value.stat-critical {
		color: var(--color-error);
	}

	.stat-value.stat-warning {
		color: var(--color-warning);
	}

	.stat-value.stat-info {
		color: var(--color-info);
	}

	.stat-value.stat-resolved {
		color: var(--color-success);
	}

	.stat-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	/* CD5: Quick templates */
	.quick-templates {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-sm);
		border-bottom: 1px solid var(--color-border);
	}

	.templates-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.template-chip {
		font-size: var(--font-size-xs);
		padding: 2px var(--spacing-sm);
		border-radius: 999px;
		color: var(--color-text-secondary);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		cursor: pointer;
	}

	.template-chip:hover {
		background-color: var(--color-accent-light);
		border-color: var(--color-accent);
		color: var(--color-accent);
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

	.issue-item {
		width: 100%;
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm) var(--spacing-md);
		border-bottom: 1px solid var(--color-border);
		text-align: left;
		background: none;
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

	.linked-header {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.add-link-btn {
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-md);
		font-weight: 700;
		color: var(--color-accent);
		background: none;
		border: 1px solid var(--color-accent);
	}

	.add-link-btn:hover {
		background-color: var(--color-accent);
		color: white;
	}

	.link-select {
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-sm);
		margin-top: var(--spacing-xs);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
	}

	.link-search-results {
		margin-top: var(--spacing-xs);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		max-height: 150px;
		overflow-y: auto;
	}

	.link-search-result {
		display: block;
		width: 100%;
		text-align: left;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
	}

	.link-search-result:hover {
		background-color: var(--color-bg-hover);
	}

	.linked-items li {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
	}

	.unlink-btn {
		color: var(--color-text-muted);
		font-size: var(--font-size-md);
		opacity: 0;
		transition: opacity var(--transition-fast);
	}

	.linked-items li:hover .unlink-btn {
		opacity: 1;
	}

	.unlink-btn:hover {
		color: var(--color-error);
	}

	.empty-link {
		color: var(--color-text-muted);
		font-style: italic;
		font-size: var(--font-size-sm);
	}
</style>
