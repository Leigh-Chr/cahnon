<!--
  Event management dialog.

  Features:
  - View all timeline events in the project
  - Create new events with title, description, time fields, type, importance
  - Edit existing events
  - Delete events
  - Link/unlink scenes to events
  - Link/unlink bible entries to events
-->
<script lang="ts">
	import { type BibleEntry, eventApi, type Scene, type TimelineEvent } from '$lib/api';
	import { appState } from '$lib/stores';
	import { showError } from '$lib/toast';
	import { bibleEntryTypes, formatDate } from '$lib/utils';
	import { trapFocus } from '$lib/utils/focus-trap';
	import { nativeConfirm } from '$lib/utils/native-dialog';

	import { Button, EmptyState, FormActions, FormGroup, Icon, LoadingState } from './ui';
	import ContextMenu from './ui/ContextMenu.svelte';
	import ContextMenuItem from './ui/ContextMenuItem.svelte';
	import ContextMenuSeparator from './ui/ContextMenuSeparator.svelte';

	let eventContextMenu = $state<{ x: number; y: number; eventId: string } | null>(null);

	function handleEventContextMenu(event: MouseEvent, eventId: string) {
		event.preventDefault();
		eventContextMenu = { x: event.clientX, y: event.clientY, eventId };
	}

	function closeEventContextMenu() {
		eventContextMenu = null;
	}

	interface Props {
		isOpen: boolean;
		onclose: () => void;
	}

	let { isOpen, onclose }: Props = $props();

	let events = $state<TimelineEvent[]>([]);
	let selectedEventId = $state<string | null>(null);
	let isCreating = $state(false);
	let isEditing = $state(false);
	let isLoading = $state(false);

	// Search and filter state
	let searchQuery = $state('');
	let filterType = $state('');
	let filterImportance = $state('');

	// Form state
	let formTitle = $state('');
	let formDescription = $state('');
	let formTimePoint = $state('');
	let formTimeStart = $state('');
	let formTimeEnd = $state('');
	let formEventType = $state('plot');
	let formImportance = $state('moderate');

	// Linked items
	let linkedScenes = $state<Scene[]>([]);
	let linkedBibleEntries = $state<BibleEntry[]>([]);
	let isAddingScene = $state(false);
	let isAddingBibleEntry = $state(false);
	let selectedSceneToAdd = $state('');
	let bibleSearchQuery = $state('');

	const eventTypes = [
		{ value: 'plot', label: 'Plot' },
		{ value: 'backstory', label: 'Backstory' },
		{ value: 'worldbuilding', label: 'Worldbuilding' },
		{ value: 'historical', label: 'Historical' },
	];

	const importanceLevels = [
		{ value: 'minor', label: 'Minor' },
		{ value: 'moderate', label: 'Moderate' },
		{ value: 'major', label: 'Major' },
		{ value: 'critical', label: 'Critical' },
	];

	let selectedEvent = $derived(
		selectedEventId ? events.find((e) => e.id === selectedEventId) || null : null
	);

	let allScenes = $derived.by(() => {
		const result: Array<{ id: string; title: string; chapterId: string }> = [];
		for (const chapter of appState.chapters) {
			const chapterScenes = appState.scenes.get(chapter.id) || [];
			for (const scene of chapterScenes) {
				result.push({ id: scene.id, title: scene.title, chapterId: chapter.id });
			}
		}
		return result;
	});

	let linkedSceneIds = $derived(linkedScenes.map((s) => s.id));
	let linkedBibleIds = $derived(linkedBibleEntries.map((e) => e.id));

	let availableScenes = $derived(allScenes.filter((s) => !linkedSceneIds.includes(s.id)));

	let filteredEvents = $derived(
		events.filter(
			(e) =>
				(!searchQuery || e.title.toLowerCase().includes(searchQuery.toLowerCase())) &&
				(!filterType || e.event_type === filterType) &&
				(!filterImportance || e.importance === filterImportance)
		)
	);

	let filteredBibleEntries = $derived(
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

	$effect(() => {
		if (isOpen && appState.project) {
			loadEvents();
		}
	});

	$effect(() => {
		if (selectedEventId && !isCreating && !isEditing) {
			loadLinkedItems(selectedEventId);
		}
	});

	async function loadEvents() {
		isLoading = true;
		try {
			events = await eventApi.getAll();
		} catch (e) {
			console.error('Failed to load events:', e);
			showError('Failed to load events');
		} finally {
			isLoading = false;
		}
	}

	async function loadLinkedItems(eventId: string) {
		try {
			linkedScenes = await eventApi.getEventScenes(eventId);
			linkedBibleEntries = await eventApi.getEventBibleEntries(eventId);
		} catch (e) {
			console.error('Failed to load linked items:', e);
		}
	}

	function startCreate() {
		isCreating = true;
		isEditing = false;
		selectedEventId = null;
		resetForm();
	}

	function startEdit(event: TimelineEvent) {
		isEditing = true;
		isCreating = false;
		selectedEventId = event.id;
		formTitle = event.title;
		formDescription = event.description || '';
		formTimePoint = event.time_point || '';
		formTimeStart = event.time_start || '';
		formTimeEnd = event.time_end || '';
		formEventType = event.event_type;
		formImportance = event.importance;
	}

	function resetForm() {
		formTitle = '';
		formDescription = '';
		formTimePoint = '';
		formTimeStart = '';
		formTimeEnd = '';
		formEventType = 'plot';
		formImportance = 'moderate';
	}

	function cancelForm() {
		isCreating = false;
		isEditing = false;
		resetForm();
	}

	async function saveEvent() {
		if (!formTitle.trim()) return;

		try {
			if (isCreating) {
				const newEvent = await eventApi.create({
					title: formTitle.trim(),
					description: formDescription.trim() || undefined,
					time_point: formTimePoint.trim() || undefined,
					time_start: formTimeStart.trim() || undefined,
					time_end: formTimeEnd.trim() || undefined,
					event_type: formEventType,
					importance: formImportance,
				});
				events = [...events, newEvent];
				selectedEventId = newEvent.id;
			} else if (isEditing && selectedEventId) {
				const updated = await eventApi.update(selectedEventId, {
					title: formTitle.trim(),
					description: formDescription.trim() || null,
					time_point: formTimePoint.trim() || null,
					time_start: formTimeStart.trim() || null,
					time_end: formTimeEnd.trim() || null,
					event_type: formEventType,
					importance: formImportance,
				});
				events = events.map((e) => (e.id === updated.id ? updated : e));
			}
			isCreating = false;
			isEditing = false;
		} catch (e) {
			console.error('Failed to save event:', e);
			showError('Failed to save event');
		}
	}

	async function deleteEvent(eventId: string) {
		if (!(await nativeConfirm('Delete this event? This action cannot be undone.', 'Delete Event')))
			return;

		try {
			await eventApi.delete(eventId);
			events = events.filter((e) => e.id !== eventId);
			if (selectedEventId === eventId) {
				selectedEventId = null;
			}
		} catch (e) {
			console.error('Failed to delete event:', e);
			showError('Failed to delete event');
		}
	}

	async function linkScene() {
		if (!selectedEventId || !selectedSceneToAdd) return;
		try {
			await eventApi.linkScene(selectedSceneToAdd, selectedEventId);
			await loadLinkedItems(selectedEventId);
			selectedSceneToAdd = '';
			isAddingScene = false;
		} catch (e) {
			console.error('Failed to link scene:', e);
			showError('Failed to link scene');
		}
	}

	async function unlinkScene(sceneId: string) {
		if (!selectedEventId) return;
		try {
			await eventApi.unlinkScene(sceneId, selectedEventId);
			await loadLinkedItems(selectedEventId);
		} catch (e) {
			console.error('Failed to unlink scene:', e);
			showError('Failed to unlink scene');
		}
	}

	async function linkBibleEntry(entryId: string) {
		if (!selectedEventId) return;
		try {
			await eventApi.linkBibleEntry(entryId, selectedEventId);
			await loadLinkedItems(selectedEventId);
			bibleSearchQuery = '';
			isAddingBibleEntry = false;
		} catch (e) {
			console.error('Failed to link bible entry:', e);
			showError('Failed to link bible entry');
		}
	}

	async function unlinkBibleEntry(entryId: string) {
		if (!selectedEventId) return;
		try {
			await eventApi.unlinkBibleEntry(entryId, selectedEventId);
			await loadLinkedItems(selectedEventId);
		} catch (e) {
			console.error('Failed to unlink bible entry:', e);
			showError('Failed to unlink bible entry');
		}
	}

	function getTypeIcon(type: string): string {
		return bibleEntryTypes.find((t) => t.value === type)?.icon || '?';
	}

	function getTypeLabel(type: string): string {
		return eventTypes.find((t) => t.value === type)?.label || type;
	}

	function getImportanceLabel(importance: string): string {
		return importanceLevels.find((l) => l.value === importance)?.label || importance;
	}

	function handleOverlayClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			onclose();
		}
	}
</script>

{#if isOpen}
	<div
		class="modal-overlay"
		onclick={handleOverlayClick}
		onkeydown={(e) => {
			if (e.key === 'Escape') onclose();
		}}
		role="presentation"
		tabindex="-1"
	>
		<!-- AE1: Focus trap -->
		<div
			class="modal-container modal-enter"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="events-title"
			tabindex="-1"
			use:trapFocus={{ onEscape: onclose }}
		>
			<div class="modal-header">
				<h2 id="events-title">Timeline Events</h2>
				<button class="close-btn" onclick={onclose} aria-label="Close">
					<Icon name="close" size={20} />
				</button>
			</div>

			<div class="modal-body">
				<div class="events-sidebar">
					<div class="sidebar-header">
						<Button size="sm" variant="primary" onclick={startCreate}>
							<Icon name="plus" size={14} />
							New Event
						</Button>
					</div>

					<div class="events-filters">
						<input
							type="text"
							class="events-search"
							placeholder="Search events..."
							bind:value={searchQuery}
						/>
						<div class="filter-row">
							<select bind:value={filterType} class="filter-select">
								<option value="">All types</option>
								{#each eventTypes as type (type.value)}
									<option value={type.value}>{type.label}</option>
								{/each}
							</select>
							<select bind:value={filterImportance} class="filter-select">
								<option value="">All importance</option>
								{#each importanceLevels as level (level.value)}
									<option value={level.value}>{level.label}</option>
								{/each}
							</select>
						</div>
					</div>

					<div class="events-list">
						{#if isLoading}
							<LoadingState message="Loading events..." size="sm" />
						{:else if filteredEvents.length === 0}
							<EmptyState
								compact
								icon="calendar"
								title={events.length === 0 ? 'No events yet' : 'No matching events'}
								description={events.length === 0
									? 'Timeline events help track what happens when in your story.'
									: 'Try adjusting your search or filters.'}
							/>
						{:else}
							{#each filteredEvents as event (event.id)}
								<button
									class="event-item"
									class:selected={selectedEventId === event.id && !isCreating}
									onclick={() => {
										selectedEventId = event.id;
										isCreating = false;
										isEditing = false;
									}}
									oncontextmenu={(e) => handleEventContextMenu(e, event.id)}
								>
									<div class="event-info">
										<span class="event-name">{event.title}</span>
										<span class="event-meta">
											{getTypeLabel(event.event_type)} &bull; {getImportanceLabel(event.importance)}
										</span>
									</div>
								</button>
							{/each}
						{/if}
					</div>
				</div>

				<div class="events-detail">
					{#if isCreating || isEditing}
						<form
							onsubmit={(e) => {
								e.preventDefault();
								saveEvent();
							}}
						>
							<h3>{isCreating ? 'Create New Event' : 'Edit Event'}</h3>

							<FormGroup label="Title">
								<input
									type="text"
									bind:value={formTitle}
									placeholder="Event title"
									maxlength={100}
									required
								/>
							</FormGroup>

							<FormGroup label="Description">
								<textarea
									bind:value={formDescription}
									rows="3"
									placeholder="What happens in this event?"
								></textarea>
							</FormGroup>

							<FormGroup label="Time Point">
								<input
									type="text"
									bind:value={formTimePoint}
									placeholder="e.g., Day 3, Chapter 5, Year 1020"
								/>
								<span class="time-hint"
									>Use consistent format across events (e.g., "Day 1", "Year 1020")</span
								>
							</FormGroup>

							<FormGroup label="Time Start">
								<input type="text" bind:value={formTimeStart} placeholder="e.g., Day 1, Year 500" />
							</FormGroup>

							<FormGroup label="Time End">
								<input type="text" bind:value={formTimeEnd} placeholder="e.g., Day 5, Year 510" />
							</FormGroup>

							<FormGroup label="Event Type">
								<select bind:value={formEventType}>
									{#each eventTypes as type (type.value)}
										<option value={type.value}>{type.label}</option>
									{/each}
								</select>
							</FormGroup>

							<FormGroup label="Importance">
								<select bind:value={formImportance}>
									{#each importanceLevels as level (level.value)}
										<option value={level.value}>{level.label}</option>
									{/each}
								</select>
							</FormGroup>

							<FormActions>
								<Button onclick={cancelForm}>Cancel</Button>
								<Button variant="primary" type="submit">
									{isCreating ? 'Create Event' : 'Save Changes'}
								</Button>
							</FormActions>
						</form>
					{:else if selectedEvent}
						<div class="event-detail">
							<div class="detail-header">
								<div class="detail-title">
									<h3>{selectedEvent.title}</h3>
									<div class="detail-badges">
										<span class="type-badge">{getTypeLabel(selectedEvent.event_type)}</span>
										<span class="importance-badge" data-importance={selectedEvent.importance}>
											{getImportanceLabel(selectedEvent.importance)}
										</span>
									</div>
								</div>
								<div class="detail-actions">
									<Button
										size="sm"
										onclick={() => {
											if (selectedEvent) startEdit(selectedEvent);
										}}
									>
										<Icon name="edit" size={14} />
										Edit
									</Button>
									<Button
										size="sm"
										onclick={() => {
											if (selectedEvent) deleteEvent(selectedEvent.id);
										}}
									>
										<Icon name="trash" size={14} />
										Delete
									</Button>
								</div>
							</div>

							{#if selectedEvent.description}
								<div class="detail-section">
									<h4>Description</h4>
									<p>{selectedEvent.description}</p>
								</div>
							{/if}

							{#if selectedEvent.time_point || selectedEvent.time_start}
								<div class="detail-section">
									<h4>Timeline</h4>
									<div class="time-info">
										{#if selectedEvent.time_point}
											<span><strong>Time:</strong> {selectedEvent.time_point}</span>
										{/if}
										{#if selectedEvent.time_start}
											<span><strong>From:</strong> {selectedEvent.time_start}</span>
										{/if}
										{#if selectedEvent.time_end}
											<span><strong>To:</strong> {selectedEvent.time_end}</span>
										{/if}
									</div>
								</div>
							{/if}

							<!-- Linked Scenes -->
							<div class="detail-section">
								<div class="section-header">
									<h4>Linked Scenes</h4>
									<Button variant="icon" size="sm" onclick={() => (isAddingScene = !isAddingScene)}>
										{#if isAddingScene}
											<Icon name="close" size={14} />
										{:else}
											<Icon name="plus" size={14} />
										{/if}
									</Button>
								</div>

								{#if isAddingScene}
									<div class="link-search">
										<select bind:value={selectedSceneToAdd} onchange={linkScene}>
											<option value="">Select a scene...</option>
											{#each availableScenes as scene (scene.id)}
												<option value={scene.id}>{scene.title}</option>
											{/each}
										</select>
									</div>
								{/if}

								<div class="linked-list">
									{#each linkedScenes as scene (scene.id)}
										<div class="linked-item">
											<span class="linked-name">{scene.title}</span>
											<button
												class="remove-btn"
												onclick={() => unlinkScene(scene.id)}
												title="Unlink scene"
											>
												&times;
											</button>
										</div>
									{:else}
										<p class="empty-message">No scenes linked.</p>
									{/each}
								</div>
							</div>

							<!-- Linked Bible Entries -->
							<div class="detail-section">
								<div class="section-header">
									<h4>Linked Bible Entries</h4>
									<Button
										variant="icon"
										size="sm"
										onclick={() => (isAddingBibleEntry = !isAddingBibleEntry)}
									>
										{#if isAddingBibleEntry}
											<Icon name="close" size={14} />
										{:else}
											<Icon name="plus" size={14} />
										{/if}
									</Button>
								</div>

								{#if isAddingBibleEntry}
									<div class="link-search">
										<input
											type="text"
											placeholder="Search bible entries..."
											bind:value={bibleSearchQuery}
										/>
										{#if filteredBibleEntries.length > 0}
											<div class="search-results">
												{#each filteredBibleEntries as entry (entry.id)}
													<button class="search-result" onclick={() => linkBibleEntry(entry.id)}>
														<span class="entry-icon">{getTypeIcon(entry.entry_type)}</span>
														<span class="entry-name">{entry.name}</span>
													</button>
												{/each}
											</div>
										{/if}
									</div>
								{/if}

								<div class="linked-list">
									{#each linkedBibleEntries as entry (entry.id)}
										<div class="linked-item">
											<span class="linked-name">{entry.name}</span>
											<button
												class="remove-btn"
												onclick={() => unlinkBibleEntry(entry.id)}
												title="Unlink entry"
											>
												&times;
											</button>
										</div>
									{:else}
										<p class="empty-message">No bible entries linked.</p>
									{/each}
								</div>
							</div>

							<div class="detail-section">
								<h4>Metadata</h4>
								<div class="meta-info">
									<span>Created: {formatDate(selectedEvent.created_at)}</span>
									<span>Updated: {formatDate(selectedEvent.updated_at)}</span>
								</div>
							</div>
						</div>
					{:else}
						<EmptyState
							icon="calendar"
							title="Select an event or create a new one"
							description="Choose an event from the list to view its details, or create a new one to track timeline events."
						/>
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}

{#if eventContextMenu}
	<ContextMenu x={eventContextMenu.x} y={eventContextMenu.y} onclose={closeEventContextMenu}>
		<ContextMenuItem
			label="Edit"
			onclick={() => {
				selectedEventId = eventContextMenu!.eventId;
				isEditing = true;
				isCreating = false;
				closeEventContextMenu();
			}}
		/>
		<ContextMenuSeparator />
		<ContextMenuItem
			label="Delete"
			danger
			onclick={() => {
				deleteEvent(eventContextMenu!.eventId);
				closeEventContextMenu();
			}}
		/>
	</ContextMenu>
{/if}

<style>
	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: var(--overlay-backdrop);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal-container {
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		width: 90%;
		max-width: 900px;
		height: 80vh;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
	}

	.modal-header h2 {
		margin: 0;
		font-size: var(--font-size-lg);
	}

	.close-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.close-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.modal-body {
		flex: 1;
		display: flex;
		overflow: hidden;
	}

	.events-sidebar {
		width: 280px;
		border-right: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
	}

	.sidebar-header {
		padding: var(--spacing-md);
		border-bottom: 1px solid var(--color-border);
	}

	.events-filters {
		padding: 0 var(--spacing-sm) var(--spacing-sm);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.events-search {
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background: var(--color-bg-primary);
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.events-search:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.filter-row {
		display: flex;
		gap: var(--spacing-xs);
	}

	.filter-select {
		flex: 1;
		padding: var(--spacing-xs) var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background: var(--color-bg-primary);
		font-size: var(--font-size-xs);
		color: var(--color-text-primary);
	}

	.events-list {
		flex: 1;
		overflow-y: auto;
	}

	.event-item {
		width: 100%;
		display: flex;
		align-items: center;
		padding: var(--spacing-sm) var(--spacing-md);
		border-bottom: 1px solid var(--color-border-light);
		text-align: left;
		background: none;
		transition: background-color var(--transition-fast);
	}

	.event-item:hover {
		background-color: var(--color-bg-hover);
	}

	.event-item.selected {
		background-color: var(--color-accent-light);
	}

	.event-info {
		flex: 1;
		min-width: 0;
	}

	.event-name {
		display: block;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.event-meta {
		display: block;
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
	}

	.events-detail {
		flex: 1;
		padding: var(--spacing-lg);
		overflow-y: auto;
	}

	.events-detail h3 {
		margin: 0 0 var(--spacing-lg) 0;
		font-size: var(--font-size-lg);
	}

	.events-detail form {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
		max-width: 500px;
	}

	.events-detail input,
	.events-detail select,
	.events-detail textarea {
		width: 100%;
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
		font-size: var(--font-size-base);
	}

	.events-detail textarea {
		resize: vertical;
	}

	.event-detail {
		max-width: 600px;
	}

	.detail-header {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-md);
		margin-bottom: var(--spacing-lg);
	}

	.detail-title {
		flex: 1;
	}

	.detail-title h3 {
		margin: 0 0 var(--spacing-xs) 0;
	}

	.detail-badges {
		display: flex;
		gap: var(--spacing-xs);
	}

	.type-badge,
	.importance-badge {
		display: inline-block;
		padding: 2px 8px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-xs);
		font-weight: 500;
		text-transform: capitalize;
	}

	.importance-badge[data-importance='critical'] {
		background-color: var(--color-error);
		color: white;
	}

	.importance-badge[data-importance='major'] {
		background-color: var(--color-warning-light);
		color: var(--color-warning);
	}

	.detail-actions {
		display: flex;
		gap: var(--spacing-xs);
	}

	.detail-section {
		margin-bottom: var(--spacing-lg);
	}

	.detail-section h4 {
		margin: 0 0 var(--spacing-xs) 0;
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.detail-section p {
		margin: 0;
		color: var(--color-text-primary);
		line-height: 1.6;
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--spacing-xs);
	}

	.section-header h4 {
		margin: 0;
	}

	.time-info {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
		font-size: var(--font-size-sm);
	}

	.link-search {
		margin-bottom: var(--spacing-sm);
	}

	.link-search select,
	.link-search input {
		width: 100%;
		padding: var(--spacing-sm);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
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

	.linked-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.linked-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-xs) var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
	}

	.linked-name {
		flex: 1;
		color: var(--color-text-primary);
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

	.linked-item:hover .remove-btn {
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

	.meta-info {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
	}

	.time-hint {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		font-style: italic;
		margin-top: var(--spacing-xs);
		display: block;
	}
</style>
