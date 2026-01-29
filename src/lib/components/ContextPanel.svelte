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
	import type { Annotation, BibleEntry, Issue } from '$lib/api';
	import type { SceneHealth } from '$lib/api';
	import {
		type Arc,
		arcApi,
		associationApi,
		eventApi,
		issueApi,
		templateApi,
		type TemplateStep,
		type TimelineEvent,
	} from '$lib/api';
	import { sceneApi } from '$lib/api';
	import { appState } from '$lib/stores';
	import { showError } from '$lib/toast';
	import { bibleEntryTypes, countWords, formatWordCount } from '$lib/utils';

	import AnnotationsPanel from './AnnotationsPanel.svelte';
	import CharacterThread from './CharacterThread.svelte';
	import NarrativeContext from './NarrativeContext.svelte';
	import RevisionChecklist from './RevisionChecklist.svelte';
	import { Button, Icon } from './ui';

	interface Props {
		onSelectAnnotation?: ((annotation: Annotation) => void) | null;
	}

	let { onSelectAnnotation = null }: Props = $props();

	// Derived values for proper reactivity tracking in templates
	let selectedScene = $derived(appState.selectedScene);
	let selectedSceneId = $derived(appState.selectedSceneId);

	// Scene health
	let sceneHealth = $derived<SceneHealth | undefined>(
		selectedSceneId ? appState.sceneHealthMap.get(selectedSceneId) : undefined
	);

	// Collapsible sections (persisted per project)
	function getCollapsedKey(): string {
		return `contextPanelCollapsed:${appState.projectPath || 'default'}`;
	}
	let collapsedSections = $state<Set<string>>(
		new Set(
			(() => {
				try {
					return JSON.parse(localStorage.getItem(getCollapsedKey()) || '[]');
				} catch {
					return [];
				}
			})()
		)
	);

	$effect(() => {
		try {
			localStorage.setItem(getCollapsedKey(), JSON.stringify([...collapsedSections]));
		} catch {
			/* ignore */
		}
	});

	function toggleSection(name: string) {
		if (collapsedSections.has(name)) {
			collapsedSections = new Set([...collapsedSections].filter((s) => s !== name));
		} else {
			collapsedSections = new Set([...collapsedSections, name]);
		}
	}

	function isSectionCollapsed(name: string): boolean {
		return collapsedSections.has(name);
	}

	// World State collapsible toggle (defaults closed, auto-opens in revision mode)
	let worldStateExpanded = $state(false);

	$effect(() => {
		if (appState.workMode === 'revision') {
			worldStateExpanded = true;
		}
	});

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
			try {
				await appState.updateScene(selectedScene.id, { notes: editedNotes || null });
				isEditingNotes = false;
			} catch (e) {
				console.error('Failed to save notes:', e);
				showError('Failed to save notes');
			}
		}
	}

	async function saveTodos() {
		if (selectedScene) {
			try {
				await appState.updateScene(selectedScene.id, { todos: editedTodos || null });
				isEditingTodos = false;
			} catch (e) {
				console.error('Failed to save TODOs:', e);
				showError('Failed to save TODOs');
			}
		}
	}

	function cancelEditingNotes() {
		isEditingNotes = false;
	}

	function cancelEditingTodos() {
		isEditingTodos = false;
	}

	let associations = $state<BibleEntry[]>([]);
	let isAddingAssociation = $state(false);
	let searchQuery = $state('');
	let associationSearchIndex = $state(0);

	// Reset search index when query changes
	$effect(() => {
		void searchQuery; // Track searchQuery changes
		associationSearchIndex = 0;
	});

	// State for arcs, template step, and linked events
	let sceneArcs = $state<Arc[]>([]);
	let allArcs = $state<Arc[]>([]);
	let isAddingArc = $state(false);
	let selectedArcToAdd = $state('');
	let templateStep = $state<TemplateStep | null>(null);
	let allTemplateSteps = $state<TemplateStep[]>([]);
	let isSelectingStep = $state(false);
	let selectedStepToAssign = $state('');
	let linkedEvents = $state<TimelineEvent[]>([]);
	let allEvents = $state<TimelineEvent[]>([]);
	let isAddingEvent = $state(false);
	let selectedEventToAdd = $state('');

	// Scene issues
	let sceneIssues = $state<Issue[]>([]);

	// Word target inline editing state
	let isEditingWordTarget = $state(false);
	let editedWordTarget = $state('');

	function startEditingWordTarget() {
		editedWordTarget = selectedScene?.word_target ? String(selectedScene.word_target) : '';
		isEditingWordTarget = true;
	}

	async function saveWordTarget() {
		if (selectedScene) {
			const trimmed = editedWordTarget.trim();
			const target = trimmed === '' ? null : parseInt(trimmed);
			if (target === null || (!isNaN(target) && target > 0)) {
				try {
					await appState.updateScene(selectedScene.id, { word_target: target });
				} catch (e) {
					console.error('Failed to set word target:', e);
					showError('Failed to set word target');
				}
			}
		}
		isEditingWordTarget = false;
	}

	function cancelEditingWordTarget() {
		isEditingWordTarget = false;
	}

	// Timeline editing state
	let isEditingTimeline = $state(false);
	let editedOnTimeline = $state(false);
	let editedTimePoint = $state('');
	let editedTimeStart = $state('');
	let editedTimeEnd = $state('');

	// Derived: arcs not already linked to the scene
	let availableArcs = $derived(allArcs.filter((arc) => !sceneArcs.some((sa) => sa.id === arc.id)));
	let availableEvents = $derived(
		allEvents.filter((ev) => !linkedEvents.some((le) => le.id === ev.id))
	);

	let loadGeneration = 0;
	// AA2: Loading states for skeleton loaders
	let isLoadingAssociations = $state(false);
	let isLoadingArcs = $state(false);
	let isLoadingEvents = $state(false);

	// AW6: Progressive loading - associations first (immediate), then arcs/events/issues (delayed 100ms)
	$effect(() => {
		if (!selectedSceneId) return;

		const gen = ++loadGeneration;
		// AA2: Set loading states
		isLoadingAssociations = true;
		isLoadingArcs = true;
		isLoadingEvents = true;

		// Phase 1: Load associations immediately (most commonly used)
		loadAssociations(gen).finally(() => {
			if (gen === loadGeneration) isLoadingAssociations = false;
		});

		// Phase 2: Delay secondary data to avoid blocking UI
		const delayedLoadTimeout = setTimeout(() => {
			if (gen !== loadGeneration) return; // Stale request

			loadSceneArcs(gen).finally(() => {
				if (gen === loadGeneration) isLoadingArcs = false;
			});
			loadAllArcs();
			loadTemplateStep(gen);
			loadLinkedEvents(gen).finally(() => {
				if (gen === loadGeneration) isLoadingEvents = false;
			});
			loadAllEvents();
			loadSceneIssues(gen);
		}, 100);

		// Cleanup timeout on scene change
		return () => clearTimeout(delayedLoadTimeout);
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

	async function loadAssociations(gen?: number) {
		if (selectedSceneId) {
			try {
				const result = await associationApi.getByScene(selectedSceneId);
				if (gen === undefined || gen === loadGeneration) {
					associations = result;
				}
			} catch (e) {
				console.error('Failed to load associations:', e);
			}
		}
	}

	async function loadSceneArcs(gen?: number) {
		if (selectedSceneId) {
			try {
				const result = await arcApi.getSceneArcs(selectedSceneId);
				if (gen === undefined || gen === loadGeneration) {
					sceneArcs = result;
				}
			} catch (e) {
				console.error('Failed to load arcs:', e);
				sceneArcs = [];
			}
		}
	}

	async function loadAllArcs() {
		try {
			allArcs = await arcApi.getAll();
		} catch (e) {
			console.error('Failed to load all arcs:', e);
			allArcs = [];
		}
	}

	async function addArcToScene() {
		if (!selectedSceneId || !selectedArcToAdd) return;
		try {
			await arcApi.linkScene(selectedSceneId, selectedArcToAdd);
			await loadSceneArcs();
			selectedArcToAdd = '';
			isAddingArc = false;
		} catch (e) {
			console.error('Failed to link scene to arc:', e);
			showError('Failed to link arc');
		}
	}

	async function removeArcFromScene(arcId: string) {
		if (!selectedSceneId) return;
		try {
			await arcApi.unlinkScene(selectedSceneId, arcId);
			await loadSceneArcs();
		} catch (e) {
			console.error('Failed to unlink scene from arc:', e);
			showError('Failed to unlink arc');
		}
	}

	async function loadTemplateStep(gen?: number) {
		if (selectedSceneId) {
			try {
				const step = await templateApi.getSceneStep(selectedSceneId);
				// Also load all available steps from active template
				const templates = await templateApi.getAll();
				const activeTemplate = templates.find((t) => t.is_active);
				const steps = activeTemplate ? await templateApi.getSteps(activeTemplate.id) : [];
				if (gen === undefined || gen === loadGeneration) {
					templateStep = step;
					allTemplateSteps = steps;
				}
			} catch (e) {
				console.error('Failed to load template step:', e);
				templateStep = null;
				allTemplateSteps = [];
			}
		}
	}

	async function assignSceneToStep() {
		if (!selectedSceneId || !selectedStepToAssign) return;
		try {
			await templateApi.assignSceneToStep(selectedSceneId, selectedStepToAssign);
			await loadTemplateStep();
			selectedStepToAssign = '';
			isSelectingStep = false;
		} catch (e) {
			console.error('Failed to assign scene to step:', e);
			showError('Failed to assign template step');
		}
	}

	async function loadLinkedEvents(gen?: number) {
		if (selectedSceneId) {
			try {
				const result = await eventApi.getSceneEvents(selectedSceneId);
				if (gen === undefined || gen === loadGeneration) {
					linkedEvents = result;
				}
			} catch (e) {
				console.error('Failed to load linked events:', e);
				linkedEvents = [];
			}
		}
	}

	async function loadAllEvents() {
		try {
			allEvents = await eventApi.getAll();
		} catch (e) {
			console.error('Failed to load all events:', e);
			allEvents = [];
		}
	}

	async function addEventToScene() {
		if (!selectedSceneId || !selectedEventToAdd) return;
		try {
			await eventApi.linkScene(selectedSceneId, selectedEventToAdd);
			await loadLinkedEvents();
			selectedEventToAdd = '';
			isAddingEvent = false;
		} catch (e) {
			console.error('Failed to link event:', e);
			showError('Failed to link event');
		}
	}

	async function removeEventFromScene(eventId: string) {
		if (!selectedSceneId) return;
		try {
			await eventApi.unlinkScene(selectedSceneId, eventId);
			await loadLinkedEvents();
		} catch (e) {
			console.error('Failed to unlink event:', e);
			showError('Failed to unlink event');
		}
	}

	async function loadSceneIssues(gen?: number) {
		if (selectedSceneId) {
			try {
				const result = await issueApi.getSceneIssues(selectedSceneId);
				if (gen === undefined || gen === loadGeneration) {
					sceneIssues = result;
				}
			} catch (e) {
				console.error('Failed to load scene issues:', e);
				sceneIssues = [];
			}
		}
	}

	function startEditingTimeline() {
		editedOnTimeline = selectedScene?.on_timeline ?? false;
		editedTimePoint = selectedScene?.time_point || '';
		editedTimeStart = selectedScene?.time_start || '';
		editedTimeEnd = selectedScene?.time_end || '';
		isEditingTimeline = true;
	}

	async function saveTimeline() {
		if (selectedScene) {
			try {
				await appState.updateScene(selectedScene.id, {
					on_timeline: editedOnTimeline,
					time_point: editedTimePoint.trim() || null,
					time_start: editedTimeStart.trim() || null,
					time_end: editedTimeEnd.trim() || null,
				});
				isEditingTimeline = false;
			} catch (e) {
				console.error('Failed to save timeline:', e);
				showError('Failed to save timeline info');
			}
		}
	}

	function cancelEditingTimeline() {
		isEditingTimeline = false;
	}

	async function savePov(value: string) {
		if (selectedScene) {
			try {
				await appState.updateScene(selectedScene.id, { pov: value.trim() || null });
			} catch (e) {
				console.error('Failed to save POV:', e);
				showError('Failed to save POV');
			}
		}
	}

	async function saveTags(value: string) {
		if (selectedScene) {
			try {
				await appState.updateScene(selectedScene.id, { tags: value.trim() || null });
			} catch (e) {
				console.error('Failed to save tags:', e);
				showError('Failed to save tags');
			}
		}
	}

	async function addAssociation(entry: BibleEntry) {
		if (selectedSceneId) {
			try {
				await associationApi.create(selectedSceneId, entry.id);
				await loadAssociations();
				searchQuery = '';
				isAddingAssociation = false;
			} catch (e) {
				console.error('Failed to add association:', e);
				showError('Failed to link entry');
			}
		}
	}

	async function removeAssociation(entryId: string) {
		if (selectedSceneId) {
			try {
				await associationApi.delete(selectedSceneId, entryId);
				await loadAssociations();
			} catch (e) {
				console.error('Failed to remove association:', e);
				showError('Failed to unlink entry');
			}
		}
	}

	function getTypeIcon(type: string): string {
		return bibleEntryTypes.find((t) => t.value === type)?.icon || '?';
	}

	// AG3: Badge counts for collapsed sections
	let notesBadge = $derived(selectedScene?.notes && isSectionCollapsed('notes') ? 1 : 0);
	let todosBadge = $derived(() => {
		if (!selectedScene?.todos || !isSectionCollapsed('todos')) return 0;
		return selectedScene.todos.split('\n').filter((t) => t.trim()).length;
	});
	let associationsBadge = $derived(isSectionCollapsed('associations') ? associations.length : 0);
	let arcsBadge = $derived(isSectionCollapsed('arcs') ? sceneArcs.length : 0);
	let eventsBadge = $derived(isSectionCollapsed('events') ? linkedEvents.length : 0);
</script>

<div class="context-panel">
	<div class="panel-tabs">
		<button
			class="panel-tab"
			class:active={appState.contextPanelTab === 'writing'}
			onclick={() => (appState.contextPanelTab = 'writing')}>Writing</button
		>
		<button
			class="panel-tab"
			class:active={appState.contextPanelTab === 'links'}
			onclick={() => (appState.contextPanelTab = 'links')}>Links</button
		>
		<button
			class="panel-tab"
			class:active={appState.contextPanelTab === 'analysis'}
			onclick={() => (appState.contextPanelTab = 'analysis')}>Analysis</button
		>
	</div>

	<!-- BC3: Quick actions bar for common operations -->
	{#if selectedScene}
		<div class="quick-actions">
			<button
				class="quick-action-btn"
				onclick={() => {
					appState.contextPanelTab = 'links';
					isAddingAssociation = true;
				}}
				title="Link character or location"
			>
				<Icon name="user" size={14} />
			</button>
			<button
				class="quick-action-btn"
				onclick={() => {
					appState.contextPanelTab = 'links';
					isAddingArc = true;
				}}
				title="Link to arc"
			>
				<Icon name="bookmark" size={14} />
			</button>
			<button
				class="quick-action-btn"
				onclick={() => {
					appState.contextPanelTab = 'writing';
					if (isSectionCollapsed('notes')) {
						toggleSection('notes');
					}
					isEditingNotes = true;
				}}
				title="Add note"
			>
				<Icon name="edit" size={14} />
			</button>
		</div>
	{/if}

	<!-- BD5: Tab content with transition animation -->
	{#if selectedScene}
		{@const sceneWords = countWords(selectedScene.text)}

		<!-- Use key block to trigger animation on tab change -->
		{#key appState.contextPanelTab}
			<div class="tab-content">
				<!-- Analysis tab: Health/Problems -->
				{#if appState.contextPanelTab === 'analysis' && sceneHealth && sceneHealth.score < 1.0}
					<section class="panel-section">
						<h3>Problems</h3>
						<div class="health-checks">
							{#each sceneHealth.checks.filter((c) => !c.passed) as check (check.name)}
								<div class="health-check check-failed">
									<span class="check-icon">!</span>
									<span class="check-label">{check.label}</span>
								</div>
							{/each}
						</div>
					</section>
				{/if}

				<!-- Writing tab: Word Count, POV, Tags, Notes, TODOs -->
				{#if appState.contextPanelTab === 'writing'}
					<section class="panel-section">
						<button class="section-toggle" onclick={() => toggleSection('wordCount')}>
							<span class="section-toggle-icon" class:collapsed={isSectionCollapsed('wordCount')}>
								<svg
									width="12"
									height="12"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg
								>
							</span>
							<h3>Word Count</h3>
						</button>
						{#if !isSectionCollapsed('wordCount')}
							<div class="word-stats">
								<div class="stat">
									<span class="stat-value">{formatWordCount(sceneWords)}</span>
									<span class="stat-label"
										>Scene (~{Math.max(1, Math.ceil(sceneWords / 250))} min)</span
									>
								</div>
								{#if appState.wordCounts && appState.selectedChapterId}
									{@const chapterStats = appState.wordCounts.by_chapter.find(
										(c) => c.chapter_id === appState.selectedChapterId
									)}
									{#if chapterStats}
										<div class="stat">
											<span class="stat-value">{formatWordCount(chapterStats.word_count)}</span>
											<span class="stat-label">Chapter</span>
										</div>
									{/if}
								{/if}
								{#if appState.wordCounts}
									<div class="stat">
										<span class="stat-value">{formatWordCount(appState.wordCounts.total)}</span>
										<span class="stat-label">Total</span>
									</div>
								{/if}
							</div>
							<div class="word-target-section">
								{#if isEditingWordTarget}
									<div class="word-target-edit">
										<input
											type="number"
											class="inline-input"
											placeholder="Word target..."
											bind:value={editedWordTarget}
											onkeydown={(e) => {
												if (e.key === 'Enter') saveWordTarget();
												if (e.key === 'Escape') cancelEditingWordTarget();
											}}
										/>
										<div class="edit-actions">
											<Button variant="ghost" size="sm" onclick={cancelEditingWordTarget}
												>Cancel</Button
											>
											<Button variant="primary" size="sm" onclick={saveWordTarget}>Save</Button>
										</div>
									</div>
								{:else if selectedScene.word_target}
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
											onclick={startEditingWordTarget}
											title="Edit word target"
										>
											<Icon name="edit" size={12} />
										</Button>
									</div>
								{:else}
									<button class="set-target-btn" onclick={startEditingWordTarget}>
										Set word target
									</button>
								{/if}
							</div>
						{/if}
					</section>
				{/if}

				{#if appState.contextPanelTab === 'writing'}
					<!-- POV Section -->
					<section class="panel-section">
						<button class="section-toggle" onclick={() => toggleSection('pov')}>
							<span class="section-toggle-icon" class:collapsed={isSectionCollapsed('pov')}>
								<svg
									width="12"
									height="12"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg
								>
							</span>
							<h3>POV</h3>
						</button>
						{#if !isSectionCollapsed('pov')}
							<input
								type="text"
								class="inline-input"
								placeholder="Set POV character..."
								value={selectedScene.pov || ''}
								onblur={(e) => savePov(e.currentTarget.value)}
							/>
						{/if}
					</section>

					<!-- Tags Section -->
					<section class="panel-section">
						<button class="section-toggle" onclick={() => toggleSection('tags')}>
							<span class="section-toggle-icon" class:collapsed={isSectionCollapsed('tags')}>
								<svg
									width="12"
									height="12"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg
								>
							</span>
							<h3>Tags</h3>
						</button>
						{#if !isSectionCollapsed('tags')}
							<input
								type="text"
								class="inline-input"
								placeholder="Add tags (comma separated)..."
								value={selectedScene.tags || ''}
								onblur={(e) => saveTags(e.currentTarget.value)}
							/>
						{/if}
					</section>
				{/if}

				<!-- Links tab: Associations Section -->
				{#if appState.contextPanelTab === 'links'}
					<section class="panel-section">
						<div class="section-header">
							<button class="section-toggle-inline" onclick={() => toggleSection('associations')}>
								<span
									class="section-toggle-icon"
									class:collapsed={isSectionCollapsed('associations')}
								>
									<svg
										width="12"
										height="12"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg
									>
								</span>
								<h3>Characters & Locations</h3>
								{#if associationsBadge > 0}
									<span class="section-badge">{associationsBadge}</span>
								{/if}
							</button>
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

						{#if !isSectionCollapsed('associations')}
							{#if isAddingAssociation}
								<div class="association-search">
									<input
										type="text"
										placeholder="Search bible entries..."
										bind:value={searchQuery}
										onkeydown={(e) => {
											const entries = filteredEntries.slice(0, 10);
											if (e.key === 'ArrowDown') {
												e.preventDefault();
												associationSearchIndex = Math.min(
													associationSearchIndex + 1,
													entries.length - 1
												);
											} else if (e.key === 'ArrowUp') {
												e.preventDefault();
												associationSearchIndex = Math.max(associationSearchIndex - 1, 0);
											} else if (e.key === 'Enter' && entries.length > 0) {
												e.preventDefault();
												addAssociation(entries[associationSearchIndex]);
											} else if (e.key === 'Escape') {
												e.stopPropagation();
												searchQuery = '';
												isAddingAssociation = false;
											}
										}}
									/>
									{#if filteredEntries.length > 0}
										<div class="search-results">
											{#each filteredEntries.slice(0, 10) as entry, i (entry.id)}
												<button
													class="search-result"
													class:selected={i === associationSearchIndex}
													onclick={() => addAssociation(entry)}
												>
													<span class="entry-icon">{getTypeIcon(entry.entry_type)}</span>
													<span class="entry-name">{entry.name}</span>
													<span class="entry-type">{entry.entry_type}</span>
												</button>
											{/each}
										</div>
									{/if}
								</div>
							{/if}

							<!-- AA2: Skeleton loader for associations -->
							<div class="associations-list">
								{#if isLoadingAssociations}
									<div class="skeleton-item"></div>
									<div class="skeleton-item"></div>
								{:else}
									{#each associations as entry (entry.id)}
										<div
											class="association-item"
											style="--entry-color: {entry.color || 'var(--color-accent)'}"
										>
											<span class="entry-icon">{getTypeIcon(entry.entry_type)}</span>
											<button
												class="entry-link"
												onclick={() => appState.navigateToBibleEntry(entry.id)}
												title="Go to {entry.name}">{entry.name}</button
											>
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
								{/if}
							</div>
						{/if}
					</section>
				{/if}

				<!-- Links tab: Arcs Section -->
				{#if appState.contextPanelTab === 'links'}
					<section class="panel-section">
						<div class="section-header">
							<button class="section-toggle-inline" onclick={() => toggleSection('arcs')}>
								<span class="section-toggle-icon" class:collapsed={isSectionCollapsed('arcs')}>
									<svg
										width="12"
										height="12"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg
									>
								</span>
								<h3>Arcs</h3>
								{#if arcsBadge > 0}
									<span class="section-badge">{arcsBadge}</span>
								{/if}
							</button>
							<Button variant="icon" size="sm" onclick={() => (isAddingArc = !isAddingArc)}>
								{#if isAddingArc}
									<Icon name="close" size={16} />
								{:else}
									<Icon name="plus" size={16} />
								{/if}
							</Button>
						</div>

						{#if !isSectionCollapsed('arcs')}
							{#if isAddingArc}
								<div class="arc-search">
									<select bind:value={selectedArcToAdd} onchange={addArcToScene}>
										<option value="">Select an arc...</option>
										{#each availableArcs as arc (arc.id)}
											<option value={arc.id}>{arc.name}</option>
										{/each}
									</select>
								</div>
							{/if}

							<!-- AA2: Skeleton loader for arcs -->
							<div class="arcs-list">
								{#if isLoadingArcs}
									<div class="skeleton-item"></div>
									<div class="skeleton-item"></div>
								{:else}
									{#each sceneArcs as arc (arc.id)}
										<div class="arc-item" style="--arc-color: {arc.color || 'var(--color-accent)'}">
											<span class="arc-color-dot"></span>
											<button
												class="entry-link arc-name"
												onclick={() => appState.navigateToArc(arc.id)}
												title="Go to {arc.name}">{arc.name}</button
											>
											<span class="arc-status">{arc.status}</span>
											<button
												class="remove-btn"
												onclick={() => removeArcFromScene(arc.id)}
												title="Remove from arc"
											>
												&times;
											</button>
										</div>
									{:else}
										<p class="empty-message">No arcs linked to this scene.</p>
									{/each}
								{/if}
							</div>
						{/if}
					</section>
				{/if}

				<!-- Links tab: Template Step Section -->
				{#if appState.contextPanelTab === 'links' && (allTemplateSteps.length > 0 || templateStep)}
					<section class="panel-section">
						<div class="section-header">
							<button class="section-toggle-inline" onclick={() => toggleSection('template')}>
								<span class="section-toggle-icon" class:collapsed={isSectionCollapsed('template')}>
									<svg
										width="12"
										height="12"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg
									>
								</span>
								<h3>Template Step</h3>
							</button>
							<Button variant="icon" size="sm" onclick={() => (isSelectingStep = !isSelectingStep)}>
								{#if isSelectingStep}
									<Icon name="close" size={16} />
								{:else}
									<Icon name="edit" size={16} />
								{/if}
							</Button>
						</div>

						{#if !isSectionCollapsed('template')}
							{#if isSelectingStep}
								<div class="step-search">
									<select bind:value={selectedStepToAssign} onchange={assignSceneToStep}>
										<option value="">Select a step...</option>
										{#each allTemplateSteps as step (step.id)}
											<option value={step.id}>{step.name} ({step.typical_position}%)</option>
										{/each}
									</select>
								</div>
							{/if}

							{#if templateStep && !isSelectingStep}
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
							{:else if !isSelectingStep}
								<p class="empty-message">No template step assigned.</p>
							{/if}
						{/if}
					</section>
				{/if}

				<!-- Links tab: Linked Timeline Events Section -->
				{#if appState.contextPanelTab === 'links'}
					<section class="panel-section">
						<div class="section-header">
							<button class="section-toggle-inline" onclick={() => toggleSection('events')}>
								<span class="section-toggle-icon" class:collapsed={isSectionCollapsed('events')}>
									<svg
										width="12"
										height="12"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg
									>
								</span>
								<h3>Linked Events</h3>
								{#if eventsBadge > 0}
									<span class="section-badge">{eventsBadge}</span>
								{/if}
							</button>
							<Button variant="icon" size="sm" onclick={() => (isAddingEvent = !isAddingEvent)}>
								{#if isAddingEvent}
									<Icon name="close" size={16} />
								{:else}
									<Icon name="plus" size={16} />
								{/if}
							</Button>
						</div>

						{#if !isSectionCollapsed('events')}
							{#if isAddingEvent}
								<div class="arc-search">
									<select bind:value={selectedEventToAdd} onchange={addEventToScene}>
										<option value="">Select an event...</option>
										{#each availableEvents as event (event.id)}
											<option value={event.id}>{event.title}</option>
										{/each}
									</select>
								</div>
							{/if}

							<!-- AA2: Skeleton loader for events -->
							<div class="events-list">
								{#if isLoadingEvents}
									<div class="skeleton-item"></div>
									<div class="skeleton-item"></div>
								{:else}
									{#each linkedEvents as event (event.id)}
										<div class="event-item">
											<div class="event-header">
												<button
													class="entry-link event-title"
													onclick={() => appState.navigateToEvent(event.id)}
													title="Go to {event.title}">{event.title}</button
												>
												<span class="event-type">{event.event_type}</span>
												<button
													class="remove-btn"
													onclick={() => removeEventFromScene(event.id)}
													title="Unlink event"
												>
													&times;
												</button>
											</div>
											{#if event.time_point || event.time_start}
												<div class="event-time">
													{event.time_point ||
														`${event.time_start}${event.time_end ? ` - ${event.time_end}` : ''}`}
												</div>
											{/if}
										</div>
									{:else}
										<p class="empty-message">No events linked to this scene.</p>
									{/each}
								{/if}
							</div>
						{/if}
					</section>
				{/if}

				<!-- Analysis tab: Linked Issues Section -->
				{#if appState.contextPanelTab === 'analysis' && sceneIssues.length > 0}
					<section class="panel-section">
						<h3>Continuity ({sceneIssues.length})</h3>
						<div class="issues-list">
							{#each sceneIssues as issue (issue.id)}
								<div class="issue-item" data-severity={issue.severity} data-status={issue.status}>
									<span class="issue-severity-dot"></span>
									<div class="issue-info">
										<span class="issue-title">{issue.title}</span>
										<span class="issue-meta">
											{issue.severity} &middot; {issue.status}
										</span>
									</div>
								</div>
							{/each}
						</div>
					</section>
				{/if}

				<!-- Links tab: Scene Timeline Info -->
				{#if appState.contextPanelTab === 'links'}
					<section class="panel-section">
						<div class="section-header">
							<button class="section-toggle-inline" onclick={() => toggleSection('timeline')}>
								<span class="section-toggle-icon" class:collapsed={isSectionCollapsed('timeline')}>
									<svg
										width="12"
										height="12"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg
									>
								</span>
								<h3>Timeline</h3>
							</button>
							{#if !isEditingTimeline}
								<Button
									variant="icon"
									size="sm"
									onclick={startEditingTimeline}
									title="Edit timeline"
								>
									<Icon name="edit" size={14} />
								</Button>
							{/if}
						</div>
						{#if !isSectionCollapsed('timeline')}
							{#if isEditingTimeline}
								<div class="timeline-edit-form">
									<label class="checkbox-label">
										<input type="checkbox" bind:checked={editedOnTimeline} />
										Show on timeline
									</label>
									<input
										type="text"
										class="inline-input"
										placeholder="Time point (e.g., Day 3)"
										bind:value={editedTimePoint}
									/>
									<input
										type="text"
										class="inline-input"
										placeholder="Time start"
										bind:value={editedTimeStart}
									/>
									<input
										type="text"
										class="inline-input"
										placeholder="Time end"
										bind:value={editedTimeEnd}
									/>
									<div class="edit-actions">
										<Button variant="ghost" size="sm" onclick={cancelEditingTimeline}>Cancel</Button
										>
										<Button variant="primary" size="sm" onclick={saveTimeline}>Save</Button>
									</div>
								</div>
							{:else}
								<div class="timeline-info">
									{#if selectedScene.on_timeline}
										<div class="timeline-badge">On Timeline</div>
									{/if}
									{#if selectedScene.time_point}
										<div class="timeline-time">
											<span class="time-label">Time:</span>
											<span class="time-value">{selectedScene.time_point}</span>
										</div>
									{:else if selectedScene.time_start}
										<div class="timeline-time">
											<span class="time-label">From:</span>
											<span class="time-value">{selectedScene.time_start}</span>
											{#if selectedScene.time_end}
												<span class="time-label">To:</span>
												<span class="time-value">{selectedScene.time_end}</span>
											{/if}
										</div>
									{:else if !selectedScene.on_timeline}
										<p class="empty-message">Not on timeline.</p>
									{/if}
								</div>
							{/if}
						{/if}
					</section>
				{/if}

				<!-- Writing tab: Notes Section -->
				{#if appState.contextPanelTab === 'writing'}
					<section class="panel-section">
						<div class="section-header">
							<button class="section-toggle-inline" onclick={() => toggleSection('notes')}>
								<span class="section-toggle-icon" class:collapsed={isSectionCollapsed('notes')}>
									<svg
										width="12"
										height="12"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg
									>
								</span>
								<h3>Notes</h3>
								{#if notesBadge > 0}
									<span class="section-badge">{notesBadge}</span>
								{/if}
							</button>
							{#if !isEditingNotes}
								<Button variant="icon" size="sm" onclick={startEditingNotes} title="Edit notes">
									<Icon name="edit" size={14} />
								</Button>
							{/if}
						</div>
						{#if !isSectionCollapsed('notes')}
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
									<button class="editable-field" onclick={startEditingNotes}>
										<p>{selectedScene.notes}</p>
										<span class="edit-hint"><Icon name="edit" size={12} /></span>
									</button>
								{:else}
									<button class="editable-field empty" onclick={startEditingNotes}>
										<p class="placeholder-text">Click to add notes...</p>
										<span class="edit-hint"><Icon name="edit" size={12} /></span>
									</button>
								{/if}
							</div>
						{/if}
					</section>
				{/if}

				<!-- Writing tab: TODOs Section -->
				{#if appState.contextPanelTab === 'writing'}
					<section class="panel-section">
						<div class="section-header">
							<button class="section-toggle-inline" onclick={() => toggleSection('todos')}>
								<span class="section-toggle-icon" class:collapsed={isSectionCollapsed('todos')}>
									<svg
										width="12"
										height="12"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"><polyline points="6 9 12 15 18 9" /></svg
									>
								</span>
								<h3>TODOs</h3>
								{#if todosBadge() > 0}
									<span class="section-badge">{todosBadge()}</span>
								{/if}
							</button>
							{#if !isEditingTodos}
								<Button variant="icon" size="sm" onclick={startEditingTodos} title="Edit TODOs">
									<Icon name="edit" size={14} />
								</Button>
							{/if}
						</div>
						{#if !isSectionCollapsed('todos')}
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
									<button class="editable-field" onclick={startEditingTodos}>
										<ul class="todos-list">
											{#each selectedScene.todos
												.split('\n')
												.filter((t) => t.trim()) as todo, index (index)}
												<li>{todo}</li>
											{/each}
										</ul>
										<span class="edit-hint"><Icon name="edit" size={12} /></span>
									</button>
								{:else}
									<button class="editable-field empty" onclick={startEditingTodos}>
										<p class="placeholder-text">Click to add TODOs...</p>
										<span class="edit-hint"><Icon name="edit" size={12} /></span>
									</button>
								{/if}
							</div>
						{/if}
					</section>
				{/if}

				<!-- Analysis tab: World State / Character Thread -->
				{#if appState.contextPanelTab === 'analysis'}
					<section class="panel-section">
						{#if appState.characterThreadId}
							<CharacterThread />
						{:else}
							<button
								class="section-toggle"
								onclick={() => (worldStateExpanded = !worldStateExpanded)}
							>
								<span class="toggle-icon">{worldStateExpanded ? '▾' : '▸'}</span>
								<h3>World State</h3>
							</button>
							{#if worldStateExpanded}
								<NarrativeContext />
							{/if}
						{/if}
					</section>
				{/if}

				<!-- Analysis tab: Annotations Section -->
				{#if appState.contextPanelTab === 'analysis'}
					<section class="panel-section annotations-section">
						<AnnotationsPanel sceneId={selectedSceneId || ''} {onSelectAnnotation} />
					</section>
				{/if}

				<!-- Analysis tab: Revision Checklist Section (revision mode only) -->
				{#if appState.contextPanelTab === 'analysis' && appState.workMode === 'revision'}
					<section class="panel-section">
						<RevisionChecklist
							checklist={(() => {
								if (!selectedScene.revision_checklist) return {};
								try {
									return JSON.parse(selectedScene.revision_checklist);
								} catch {
									return {};
								}
							})()}
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
			</div>
		{/key}
	{:else}
		<!-- AL2: Empty state with keyboard shortcut tips -->
		<div class="no-selection">
			<div class="no-selection-icon">
				<Icon name="file" size={32} />
			</div>
			<h4>No scene selected</h4>
			<p>Select a scene from the outline to edit its metadata.</p>
			<div class="tips">
				<p class="tip"><kbd>↑</kbd><kbd>↓</kbd> Navigate scenes</p>
				<p class="tip"><kbd>⌘K</kbd> Quick Open</p>
				<p class="tip"><kbd>⌘N</kbd> New scene</p>
			</div>
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

	.panel-tabs {
		display: flex;
		gap: 2px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-md);
		padding: 2px;
	}

	/* BD5: Tab content transition */
	.tab-content {
		animation: fadeIn 150ms ease-out;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: translateY(4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	/* BC3: Quick actions bar */
	.quick-actions {
		display: flex;
		justify-content: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-xs) 0;
		border-bottom: 1px solid var(--color-border-light);
	}

	.quick-action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: var(--border-radius-sm);
		color: var(--color-text-muted);
		transition: all var(--transition-fast);
	}

	.quick-action-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-accent);
	}

	.panel-tab {
		flex: 1;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-xs);
		font-weight: 500;
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
		text-align: center;
		transition: all var(--transition-fast);
	}

	.panel-tab:hover {
		color: var(--color-text-primary);
	}

	.panel-tab.active {
		background-color: var(--color-bg-primary);
		color: var(--color-text-primary);
		box-shadow: var(--shadow-sm);
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

	.word-target-edit {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
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

	.search-result:hover,
	.search-result.selected {
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

	.entry-link {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		text-align: left;
		color: inherit;
		cursor: pointer;
		text-decoration: none;
		transition: color var(--transition-fast);
	}

	.entry-link:hover {
		color: var(--color-accent);
		text-decoration: underline;
		text-underline-offset: 2px;
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

	.editable-field {
		position: relative;
		display: block;
		width: 100%;
		text-align: left;
		padding: var(--spacing-xs) var(--spacing-sm);
		border: 1px dashed transparent;
		border-radius: var(--border-radius-sm);
		cursor: text;
		transition: all var(--transition-fast);
	}

	.editable-field:hover {
		border-color: var(--color-border);
		background-color: var(--color-bg-secondary);
	}

	.editable-field .edit-hint {
		position: absolute;
		top: var(--spacing-xs);
		right: var(--spacing-xs);
		opacity: 0;
		color: var(--color-text-muted);
		transition: opacity var(--transition-fast);
	}

	.editable-field:hover .edit-hint {
		opacity: 1;
	}

	.editable-field.empty {
		border-color: var(--color-border-light, var(--color-border));
		border-style: dashed;
	}

	.placeholder-text {
		color: var(--color-text-muted);
		font-style: italic;
		font-size: var(--font-size-sm);
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

	.todos-list {
		list-style: disc;
		padding-left: var(--spacing-md);
		margin: 0;
	}

	.todos-list li {
		margin-bottom: var(--spacing-xs);
	}

	/* Arcs styles */
	.arc-search {
		margin-bottom: var(--spacing-sm);
	}

	.arc-search select {
		width: 100%;
		padding: var(--spacing-sm);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
	}

	.step-search {
		margin-bottom: var(--spacing-sm);
	}

	.step-search select {
		width: 100%;
		padding: var(--spacing-sm);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
	}

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

	.arc-item .remove-btn {
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

	.arc-item:hover .remove-btn {
		opacity: 1;
	}

	.arc-item .remove-btn:hover {
		background-color: var(--color-error);
		color: var(--text-on-accent);
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

	/* AL2: Enhanced empty state with tips */
	.no-selection {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-xl);
		text-align: center;
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
	}

	.no-selection-icon {
		opacity: 0.5;
		margin-bottom: var(--spacing-sm);
	}

	.no-selection h4 {
		font-size: var(--font-size-base);
		font-weight: 600;
		color: var(--color-text-secondary);
		margin: 0;
	}

	.no-selection p {
		margin: 0;
		max-width: 200px;
	}

	.tips {
		margin-top: var(--spacing-md);
		padding-top: var(--spacing-md);
		border-top: 1px solid var(--color-border-light);
	}

	.tip {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		margin: var(--spacing-xs) 0;
	}

	.tip kbd {
		display: inline-block;
		padding: 2px 6px;
		background-color: var(--color-bg-tertiary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-family: var(--font-family-mono);
		font-size: 10px;
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

	/* Inline input for POV, Tags */
	.inline-input {
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border: 1px dashed transparent;
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
		transition: border-color var(--transition-fast);
	}

	.inline-input:hover {
		border-color: var(--color-border);
	}

	.inline-input:focus {
		outline: none;
		border-color: var(--color-accent);
		border-style: solid;
	}

	/* Timeline edit form */
	.timeline-edit-form {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.checkbox-label input[type='checkbox'] {
		width: 16px;
		height: 16px;
	}

	/* Event item remove button visibility */
	.event-item .remove-btn {
		opacity: 0;
	}

	.event-item:hover .remove-btn {
		opacity: 1;
	}

	/* Scene Issues */
	.issues-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.issue-item {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-sm);
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
	}

	.issue-severity-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
		margin-top: 5px;
		background-color: var(--color-warning);
	}

	.issue-item[data-severity='error'] .issue-severity-dot {
		background-color: var(--color-error);
	}

	.issue-item[data-severity='info'] .issue-severity-dot {
		background-color: var(--color-info);
	}

	.issue-item[data-status='resolved'] .issue-severity-dot {
		opacity: 0.4;
	}

	.issue-info {
		flex: 1;
		min-width: 0;
	}

	.issue-title {
		display: block;
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.issue-item[data-status='resolved'] .issue-title {
		text-decoration: line-through;
		opacity: 0.6;
	}

	.issue-meta {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.health-checks {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.health-check {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: 2px var(--spacing-xs);
		font-size: var(--font-size-sm);
		border-radius: var(--border-radius-sm);
	}

	.check-icon {
		font-weight: 700;
		width: 16px;
		text-align: center;
		flex-shrink: 0;
	}

	.check-failed .check-icon {
		color: var(--color-warning);
	}

	.check-label {
		color: var(--color-text-secondary);
	}

	.check-failed .check-label {
		color: var(--color-text-primary);
	}

	/* Section toggle (collapsible headers) */
	.section-toggle {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		width: 100%;
		padding: 0;
		margin-bottom: var(--spacing-sm);
		background: none;
		border: none;
		text-align: left;
		cursor: default;
	}

	.section-toggle h3 {
		margin: 0;
	}

	.section-toggle-inline {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: 0;
		background: none;
		border: none;
		cursor: default;
	}

	.section-toggle-inline h3 {
		margin: 0;
	}

	.section-toggle-icon {
		display: flex;
		color: var(--color-text-muted);
		transition: transform var(--transition-fast);
	}

	.section-toggle-icon.collapsed {
		transform: rotate(-90deg);
	}

	/* AG3: Badge for collapsed sections with content */
	.section-badge {
		background-color: var(--color-accent);
		color: var(--text-on-accent);
		font-size: var(--font-size-xs);
		font-weight: 600;
		padding: 0 6px;
		border-radius: 10px;
		min-width: 18px;
		text-align: center;
		margin-left: auto;
	}

	.toggle-icon {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		width: 14px;
		flex-shrink: 0;
	}

	/* AA2: Skeleton loader styles */
	.skeleton-item {
		height: 36px;
		background: linear-gradient(
			90deg,
			var(--color-bg-tertiary) 25%,
			var(--color-bg-secondary) 50%,
			var(--color-bg-tertiary) 75%
		);
		background-size: 200% 100%;
		border-radius: var(--border-radius-sm);
		animation: skeleton-shimmer 1.5s ease-in-out infinite;
	}

	@keyframes skeleton-shimmer {
		0% {
			background-position: 200% 0;
		}
		100% {
			background-position: -200% 0;
		}
	}
</style>
