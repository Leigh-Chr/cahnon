<!--
  Side panel displaying scene metadata and associations.

  Features:
  - Scene notes and TODOs (editable in-place)
  - Word count and statistics
  - Bible entry associations (add/remove canonical links)
  - Arc membership
  - Timeline event links
  - Narrative template step assignment
  - Annotations panel (in revision mode)
  - Revision checklist (in revision mode)
  - Status chart showing scene status distribution
-->
<script lang="ts">
	import { appState } from '$lib/stores';
	import {
		associationApi,
		arcApi,
		templateApi,
		eventApi,
		type Arc,
		type TemplateStep,
		type TimelineEvent,
	} from '$lib/api';
	import type { BibleEntry, Annotation } from '$lib/api';
	import { bibleEntryTypes, countWords, formatWordCount } from '$lib/utils';
	import AnnotationsPanel from './AnnotationsPanel.svelte';
	import StatusChart from './StatusChart.svelte';
	import RevisionChecklist from './RevisionChecklist.svelte';
	import { sceneApi } from '$lib/api';
	import { showError } from '$lib/toast';
	import { Icon, Button } from './ui';

	interface Props {
		onSelectAnnotation?: ((annotation: Annotation) => void) | null;
	}

	let { onSelectAnnotation = null }: Props = $props();

	// Derived values for proper reactivity tracking in templates
	let selectedScene = $derived(appState.selectedScene);
	let selectedSceneId = $derived(appState.selectedSceneId);

	let annotationsPanel = $state<AnnotationsPanel | null>(null);

	// Editable fields state
	let isEditingNotes = $state(false);
	let isEditingTodos = $state(false);
	let editedNotes = $state('');
	let editedTodos = $state('');

	function startEditingNotes() {
		editedNotes = selectedScene?.notes || '';
		isEditingNotes = true;
	}

	function startEditingTodos() {
		editedTodos = selectedScene?.todos || '';
		isEditingTodos = true;
	}

	async function saveNotes() {
		if (selectedScene) {
			await appState.updateScene(selectedScene.id, { notes: editedNotes || null });
			isEditingNotes = false;
		}
	}

	async function saveTodos() {
		if (selectedScene) {
			await appState.updateScene(selectedScene.id, { todos: editedTodos || null });
			isEditingTodos = false;
		}
	}

	function cancelEditingNotes() {
		isEditingNotes = false;
	}

	function cancelEditingTodos() {
		isEditingTodos = false;
	}

	// Export function for editor to call when text is selected
	export function addAnnotationForSelection(startOffset: number, endOffset: number) {
		annotationsPanel?.addAnnotationForSelection(startOffset, endOffset);
	}

	let associations = $state<BibleEntry[]>([]);
	let isAddingAssociation = $state(false);
	let searchQuery = $state('');

	// State for arcs, template step, and linked events
	let sceneArcs = $state<Arc[]>([]);
	let templateStep = $state<TemplateStep | null>(null);
	let linkedEvents = $state<TimelineEvent[]>([]);

	$effect(() => {
		if (selectedSceneId) {
			loadAssociations();
			loadSceneArcs();
			loadTemplateStep();
			loadLinkedEvents();
		}
	});

	let filteredEntries = $derived(
		searchQuery
			? appState.bibleEntries.filter(
					(e) =>
						e.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
						(e.aliases && e.aliases.toLowerCase().includes(searchQuery.toLowerCase()))
				)
			: []
	);

	async function loadAssociations() {
		if (selectedSceneId) {
			associations = await associationApi.getByScene(selectedSceneId);
		}
	}

	async function loadSceneArcs() {
		if (selectedSceneId) {
			try {
				sceneArcs = await arcApi.getSceneArcs(selectedSceneId);
			} catch (e) {
				console.error('Failed to load arcs:', e);
				sceneArcs = [];
			}
		}
	}

	async function loadTemplateStep() {
		if (selectedSceneId) {
			try {
				templateStep = await templateApi.getSceneStep(selectedSceneId);
			} catch (e) {
				console.error('Failed to load template step:', e);
				templateStep = null;
			}
		}
	}

	async function loadLinkedEvents() {
		if (selectedSceneId) {
			try {
				linkedEvents = await eventApi.getSceneEvents(selectedSceneId);
			} catch (e) {
				console.error('Failed to load linked events:', e);
				linkedEvents = [];
			}
		}
	}

	async function addAssociation(entry: BibleEntry) {
		if (selectedSceneId) {
			await associationApi.create(selectedSceneId, entry.id);
			await loadAssociations();
			searchQuery = '';
			isAddingAssociation = false;
		}
	}

	async function removeAssociation(entryId: string) {
		if (selectedSceneId) {
			await associationApi.delete(selectedSceneId, entryId);
			await loadAssociations();
		}
	}

	function getTypeIcon(type: string): string {
		return bibleEntryTypes.find((t) => t.value === type)?.icon || '?';
	}
</script>

<div class="context-panel">
	{#if selectedScene}
		<!-- Word Count Section -->
		<section class="panel-section">
			<h3>Word Count</h3>
			<div class="word-stats">
				<div class="stat">
					<span class="stat-value">{formatWordCount(countWords(selectedScene.text))}</span>
					<span class="stat-label">Scene</span>
				</div>
				{#if appState.wordCounts}
					<div class="stat">
						<span class="stat-value">{formatWordCount(appState.wordCounts.total)}</span>
						<span class="stat-label">Total</span>
					</div>
				{/if}
			</div>
			<div class="word-target-section">
				{#if selectedScene.word_target}
					{@const progress = Math.min(
						100,
						(countWords(selectedScene.text) / selectedScene.word_target) * 100
					)}
					<div class="word-progress">
						<div class="progress-bar">
							<div class="progress-fill" style="width: {progress}%"></div>
						</div>
						<span class="progress-text">
							{countWords(selectedScene.text)} / {selectedScene.word_target}
						</span>
						<Button
							variant="icon"
							size="sm"
							onclick={() => {
								const newTarget = prompt(
									'Set word target for this scene:',
									String(selectedScene?.word_target || '')
								);
								if (newTarget !== null && selectedScene) {
									const target = newTarget.trim() === '' ? null : parseInt(newTarget);
									if (target === null || !isNaN(target)) {
										appState.updateScene(selectedScene.id, { word_target: target });
									}
								}
							}}
							title="Edit word target"
						>
							<Icon name="edit" size={12} />
						</Button>
					</div>
				{:else}
					<button
						class="set-target-btn"
						onclick={() => {
							const newTarget = prompt('Set word target for this scene:');
							if (newTarget !== null && selectedScene) {
								const target = parseInt(newTarget);
								if (!isNaN(target) && target > 0) {
									appState.updateScene(selectedScene.id, { word_target: target });
								}
							}
						}}
					>
						Set word target
					</button>
				{/if}
			</div>
		</section>

		<!-- Status Chart Section -->
		<section class="panel-section">
			<StatusChart />
		</section>

		<!-- Associations Section -->
		<section class="panel-section">
			<div class="section-header">
				<h3>Characters & Locations</h3>
				<Button
					variant="icon"
					size="sm"
					onclick={() => (isAddingAssociation = !isAddingAssociation)}
				>
					{#if isAddingAssociation}
						<Icon name="close" size={16} />
					{:else}
						<Icon name="plus" size={16} />
					{/if}
				</Button>
			</div>

			{#if isAddingAssociation}
				<div class="association-search">
					<input type="text" placeholder="Search bible entries..." bind:value={searchQuery} />
					{#if filteredEntries.length > 0}
						<div class="search-results">
							{#each filteredEntries.slice(0, 10) as entry (entry.id)}
								<button class="search-result" onclick={() => addAssociation(entry)}>
									<span class="entry-icon">{getTypeIcon(entry.entry_type)}</span>
									<span class="entry-name">{entry.name}</span>
									<span class="entry-type">{entry.entry_type}</span>
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{/if}

			<div class="associations-list">
				{#each associations as entry (entry.id)}
					<div
						class="association-item"
						style="--entry-color: {entry.color || 'var(--color-accent)'}"
					>
						<span class="entry-icon">{getTypeIcon(entry.entry_type)}</span>
						<span class="entry-name">{entry.name}</span>
						<button
							class="remove-btn"
							onclick={() => removeAssociation(entry.id)}
							title="Remove association"
						>
							×
						</button>
					</div>
				{:else}
					<p class="empty-message">No characters or locations linked to this scene.</p>
				{/each}
			</div>
		</section>

		<!-- Linked Arcs Section -->
		{#if sceneArcs.length > 0}
			<section class="panel-section">
				<h3>Arcs</h3>
				<div class="arcs-list">
					{#each sceneArcs as arc (arc.id)}
						<div class="arc-item" style="--arc-color: {arc.color || 'var(--color-accent)'}">
							<span class="arc-color-dot"></span>
							<span class="arc-name">{arc.name}</span>
							<span class="arc-status">{arc.status}</span>
						</div>
					{/each}
				</div>
			</section>
		{/if}

		<!-- Template Step Section -->
		{#if templateStep}
			<section class="panel-section">
				<h3>Template Step</h3>
				<div
					class="template-step"
					style="--step-color: {templateStep.color || 'var(--color-accent)'}"
				>
					<span class="step-color-dot"></span>
					<div class="step-info">
						<span class="step-name">{templateStep.name}</span>
						{#if templateStep.description}
							<span class="step-description">{templateStep.description}</span>
						{/if}
					</div>
				</div>
			</section>
		{/if}

		<!-- Linked Timeline Events Section -->
		{#if linkedEvents.length > 0}
			<section class="panel-section">
				<h3>Linked Events</h3>
				<div class="events-list">
					{#each linkedEvents as event (event.id)}
						<div class="event-item">
							<div class="event-header">
								<span class="event-title">{event.title}</span>
								<span class="event-type">{event.event_type}</span>
							</div>
							{#if event.time_point || event.time_start}
								<div class="event-time">
									{event.time_point ||
										`${event.time_start}${event.time_end ? ` - ${event.time_end}` : ''}`}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			</section>
		{/if}

		<!-- Scene Timeline Info -->
		{#if selectedScene.on_timeline && (selectedScene.time_point || selectedScene.time_start)}
			<section class="panel-section">
				<h3>Timeline</h3>
				<div class="timeline-info">
					<div class="timeline-badge">On Timeline</div>
					<div class="timeline-time">
						{#if selectedScene.time_point}
							<span class="time-label">Time:</span>
							<span class="time-value">{selectedScene.time_point}</span>
						{:else if selectedScene.time_start}
							<span class="time-label">From:</span>
							<span class="time-value">{selectedScene.time_start}</span>
							{#if selectedScene.time_end}
								<span class="time-label">To:</span>
								<span class="time-value">{selectedScene.time_end}</span>
							{/if}
						{/if}
					</div>
				</div>
			</section>
		{/if}

		<!-- Notes Section -->
		<section class="panel-section">
			<div class="section-header">
				<h3>Notes</h3>
				{#if !isEditingNotes}
					<Button variant="icon" size="sm" onclick={startEditingNotes} title="Edit notes">
						<Icon name="edit" size={14} />
					</Button>
				{/if}
			</div>
			<div class="notes-content">
				{#if isEditingNotes}
					<textarea
						bind:value={editedNotes}
						placeholder="Add private notes..."
						rows="4"
						class="edit-textarea"
					></textarea>
					<div class="edit-actions">
						<Button variant="ghost" size="sm" onclick={cancelEditingNotes}>Cancel</Button>
						<Button variant="primary" size="sm" onclick={saveNotes}>Save</Button>
					</div>
				{:else if selectedScene.notes}
					<p>{selectedScene.notes}</p>
				{:else}
					<p class="empty-message">
						No notes for this scene. <button class="add-link" onclick={startEditingNotes}
							>Add notes</button
						>
					</p>
				{/if}
			</div>
		</section>

		<!-- TODOs Section -->
		<section class="panel-section">
			<div class="section-header">
				<h3>TODOs</h3>
				{#if !isEditingTodos}
					<Button variant="icon" size="sm" onclick={startEditingTodos} title="Edit TODOs">
						<Icon name="edit" size={14} />
					</Button>
				{/if}
			</div>
			<div class="todos-content">
				{#if isEditingTodos}
					<textarea
						bind:value={editedTodos}
						placeholder="Add TODOs (one per line)..."
						rows="4"
						class="edit-textarea"
					></textarea>
					<div class="edit-actions">
						<Button variant="ghost" size="sm" onclick={cancelEditingTodos}>Cancel</Button>
						<Button variant="primary" size="sm" onclick={saveTodos}>Save</Button>
					</div>
				{:else if selectedScene.todos}
					<ul class="todos-list">
						{#each selectedScene.todos.split('\n').filter((t) => t.trim()) as todo, index (index)}
							<li>{todo}</li>
						{/each}
					</ul>
				{:else}
					<p class="empty-message">
						No TODOs for this scene. <button class="add-link" onclick={startEditingTodos}
							>Add TODOs</button
						>
					</p>
				{/if}
			</div>
		</section>

		<!-- Annotations Section (Revision Mode) -->
		{#if appState.workMode === 'revision'}
			<section class="panel-section annotations-section">
				<AnnotationsPanel
					bind:this={annotationsPanel}
					sceneId={selectedSceneId || ''}
					{onSelectAnnotation}
				/>
			</section>

			<!-- Revision Checklist Section -->
			<section class="panel-section">
				<RevisionChecklist
					checklist={selectedScene.revision_checklist
						? JSON.parse(selectedScene.revision_checklist)
						: {}}
					onchange={async (checklist) => {
						if (selectedSceneId) {
							try {
								await sceneApi.update(selectedSceneId, {
									revision_checklist: JSON.stringify(checklist),
								});
							} catch (_err) {
								showError('Failed to save checklist');
							}
						}
					}}
				/>
			</section>
		{/if}
	{:else}
		<div class="no-selection">
			<p>Select a scene to see context information.</p>
		</div>
	{/if}
</div>

<style>
	.context-panel {
		padding: var(--spacing-md);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
	}

	.panel-section {
		padding-bottom: var(--spacing-md);
		border-bottom: 1px solid var(--color-border-light);
	}

	.panel-section:last-child {
		border-bottom: none;
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--spacing-sm);
	}

	h3 {
		font-size: var(--font-size-xs);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-muted);
		margin-bottom: var(--spacing-sm);
	}

	.section-header h3 {
		margin-bottom: 0;
	}

	.word-stats {
		display: flex;
		gap: var(--spacing-lg);
	}

	.stat {
		display: flex;
		flex-direction: column;
	}

	.stat-value {
		font-size: var(--font-size-xl);
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.stat-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.word-progress {
		margin-top: var(--spacing-sm);
	}

	.progress-bar {
		height: 4px;
		background-color: var(--color-bg-tertiary);
		border-radius: 2px;
		overflow: hidden;
		margin-bottom: var(--spacing-xs);
	}

	.progress-fill {
		height: 100%;
		background-color: var(--color-accent);
		border-radius: 2px;
		transition: width var(--transition-normal);
	}

	.progress-text {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.word-target-section {
		margin-top: var(--spacing-sm);
	}

	.word-progress {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: var(--spacing-xs);
	}

	.word-progress .progress-bar {
		flex: 1;
		min-width: 60px;
		margin-bottom: 0;
	}

	.word-progress :global(.btn-icon) {
		opacity: 0.5;
	}

	.word-progress :global(.btn-icon:hover) {
		opacity: 1;
	}

	.set-target-btn {
		font-size: var(--font-size-xs);
		color: var(--color-accent);
		padding: var(--spacing-xs) var(--spacing-sm);
		border: 1px dashed var(--color-border);
		border-radius: var(--border-radius-sm);
		transition: all var(--transition-fast);
	}

	.set-target-btn:hover {
		background-color: var(--color-bg-hover);
		border-color: var(--color-accent);
	}

	.association-search {
		margin-bottom: var(--spacing-sm);
	}

	.association-search input {
		width: 100%;
		padding: var(--spacing-sm);
		font-size: var(--font-size-sm);
	}

	.search-results {
		margin-top: var(--spacing-xs);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		max-height: 200px;
		overflow-y: auto;
	}

	.search-result {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		width: 100%;
		padding: var(--spacing-sm);
		text-align: left;
		font-size: var(--font-size-sm);
		transition: background-color var(--transition-fast);
	}

	.search-result:hover {
		background-color: var(--color-bg-hover);
	}

	.entry-icon {
		flex-shrink: 0;
	}

	.entry-name {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.entry-type {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.associations-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.association-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-sm);
		border-left: 3px solid var(--entry-color);
	}

	.remove-btn {
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-md);
		color: var(--color-text-muted);
		opacity: 0;
		transition: all var(--transition-fast);
	}

	.association-item:hover .remove-btn {
		opacity: 1;
	}

	.remove-btn:hover {
		background-color: var(--color-error);
		color: var(--text-on-accent);
	}

	.empty-message {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		font-style: italic;
	}

	.notes-content,
	.todos-content {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		line-height: var(--line-height-normal);
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.section-header h3 {
		margin: 0;
	}

	.edit-textarea {
		width: 100%;
		padding: var(--spacing-sm);
		font-size: var(--font-size-sm);
		font-family: inherit;
		line-height: var(--line-height-normal);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		resize: vertical;
		background-color: var(--color-bg-primary);
	}

	.edit-textarea:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.edit-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-sm);
		margin-top: var(--spacing-sm);
	}

	.add-link {
		color: var(--color-accent);
		font-style: normal;
		cursor: pointer;
	}

	.add-link:hover {
		text-decoration: underline;
	}

	.todos-list {
		list-style: disc;
		padding-left: var(--spacing-md);
		margin: 0;
	}

	.todos-list li {
		margin-bottom: var(--spacing-xs);
	}

	/* Arcs styles */
	.arcs-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.arc-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-xs) var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
	}

	.arc-color-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background-color: var(--arc-color);
		flex-shrink: 0;
	}

	.arc-name {
		flex: 1;
		color: var(--color-text-primary);
	}

	.arc-status {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-transform: capitalize;
	}

	/* Template step styles */
	.template-step {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-sm);
		border-left: 3px solid var(--step-color);
	}

	.step-color-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background-color: var(--step-color);
		flex-shrink: 0;
		margin-top: 4px;
	}

	.step-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.step-name {
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.step-description {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	/* Linked events styles */
	.events-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.event-item {
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-sm);
		border-left: 3px solid var(--color-accent);
	}

	.event-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--spacing-sm);
	}

	.event-title {
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.event-type {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-transform: capitalize;
	}

	.event-time {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		margin-top: 2px;
	}

	/* Timeline info styles */
	.timeline-info {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.timeline-badge {
		display: inline-flex;
		align-items: center;
		padding: 2px 8px;
		background-color: var(--color-accent-light);
		color: var(--color-accent);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-xs);
		font-weight: 500;
		width: fit-content;
	}

	.timeline-time {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-xs);
		font-size: var(--font-size-sm);
	}

	.time-label {
		color: var(--color-text-muted);
	}

	.time-value {
		color: var(--color-text-primary);
		font-weight: 500;
	}

	.no-selection {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 200px;
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
	}

	.annotations-section {
		flex: 1;
		min-height: 200px;
		display: flex;
		flex-direction: column;
	}

	.annotations-section :global(.annotations-panel) {
		flex: 1;
	}
</style>
