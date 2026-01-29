<script lang="ts">
	/**
	 * AnnotationsPanel Component
	 *
	 * Side panel for managing scene annotations (comments, questions, TODOs, etc.).
	 * Supports filtering by status and inline creation.
	 */

	import { type Annotation, annotationApi } from '$lib/api';
	import { appState } from '$lib/stores';
	import { showError } from '$lib/toast';
	import { formatDate } from '$lib/utils';
	import { getAnnotationStatusColor, getAnnotationType } from '$lib/utils/annotations';

	import { Button, Icon } from './ui';

	interface Props {
		sceneId: string;
		onSelectAnnotation?: ((annotation: Annotation) => void) | null;
	}

	let { sceneId, onSelectAnnotation = null }: Props = $props();

	let annotations = $state<Annotation[]>([]);
	let isLoading = $state(true);
	let filterStatus = $state('all');

	// Inline edit state
	let editingAnnotationId = $state<string | null>(null);
	let editedContent = $state('');

	const annotationStatuses = ['open', 'in_progress', 'resolved'];

	$effect(() => {
		if (sceneId) {
			loadAnnotations();
		}
	});

	let filteredAnnotations = $derived(
		filterStatus === 'all' ? annotations : annotations.filter((a) => a.status === filterStatus)
	);

	async function loadAnnotations() {
		isLoading = true;
		try {
			annotations = await annotationApi.getByScene(sceneId);
		} catch (e) {
			console.error('Failed to load annotations:', e);
			showError('Failed to load annotations');
			annotations = [];
		}
		isLoading = false;
	}

	async function updateAnnotationStatus(annotation: Annotation, status: string) {
		try {
			const updated = await annotationApi.update(annotation.id, { status });
			annotations = annotations.map((a) => (a.id === updated.id ? updated : a));
			appState.annotationVersion++;
		} catch (e) {
			console.error('Failed to update annotation:', e);
			showError('Failed to update annotation');
		}
	}

	async function deleteAnnotation(annotationId: string) {
		try {
			await annotationApi.delete(annotationId);
			annotations = annotations.filter((a) => a.id !== annotationId);
			appState.annotationVersion++;
		} catch (e) {
			console.error('Failed to delete annotation:', e);
			showError('Failed to delete annotation');
		}
	}

	function startEditingAnnotation(annotation: Annotation) {
		editingAnnotationId = annotation.id;
		editedContent = annotation.content;
	}

	function cancelEditingAnnotation() {
		editingAnnotationId = null;
		editedContent = '';
	}

	async function saveAnnotationContent(annotationId: string) {
		if (!editedContent.trim()) return;
		try {
			const updated = await annotationApi.update(annotationId, { content: editedContent.trim() });
			annotations = annotations.map((a) => (a.id === updated.id ? updated : a));
			editingAnnotationId = null;
			editedContent = '';
			appState.annotationVersion++;
		} catch (e) {
			console.error('Failed to update annotation:', e);
			showError('Failed to update annotation');
		}
	}

	// Watch for focusedAnnotationId from store (set by Editor on highlight click)
	$effect(() => {
		const focusedId = appState.focusedAnnotationId;
		if (!focusedId) return;

		const selector = `.annotation-item[data-annotation-id="${CSS.escape(focusedId)}"]`;

		// Retry up to a few times — the panel may still be loading annotations
		function tryScrollToAnnotation(retriesLeft: number) {
			const el = document.querySelector(selector) as HTMLElement | null;
			if (el) {
				el.scrollIntoView({ behavior: 'smooth', block: 'center' });
				el.classList.add('annotation-item-focused');
				setTimeout(() => el.classList.remove('annotation-item-focused'), 1500);
				appState.focusedAnnotationId = null;
			} else if (retriesLeft > 0) {
				setTimeout(() => tryScrollToAnnotation(retriesLeft - 1), 150);
			} else {
				appState.focusedAnnotationId = null;
			}
		}

		setTimeout(() => tryScrollToAnnotation(5), 50);
	});
</script>

<div class="annotations-panel">
	<div class="panel-header">
		<h3>Annotations</h3>
		<select bind:value={filterStatus} class="filter-select">
			<option value="all">All</option>
			<option value="open">Open</option>
			<option value="in_progress">In Progress</option>
			<option value="resolved">Resolved</option>
		</select>
	</div>

	{#if isLoading}
		<div class="loading">Loading...</div>
	{:else if filteredAnnotations.length === 0}
		<div class="empty-state">
			<p>No annotations yet</p>
			<p class="hint">Select text in the editor and add annotations.</p>
		</div>
	{:else}
		<div class="annotations-list">
			{#each filteredAnnotations as annotation (annotation.id)}
				{@const typeInfo = getAnnotationType(annotation.annotation_type)}
				<div
					class="annotation-item"
					class:resolved={annotation.status === 'resolved'}
					data-annotation-id={annotation.id}
					style="--annotation-color: {typeInfo.color}"
					onclick={() => onSelectAnnotation?.(annotation)}
					onkeydown={(e) => e.key === 'Enter' && onSelectAnnotation?.(annotation)}
					role="button"
					tabindex="0"
				>
					<div class="annotation-header">
						<span class="annotation-type">{typeInfo.icon}</span>
						<span class="annotation-date">{formatDate(annotation.created_at)}</span>
						<select
							value={annotation.status}
							onclick={(e) => e.stopPropagation()}
							onchange={(e) => updateAnnotationStatus(annotation, e.currentTarget.value)}
							class="status-select"
							style="color: {getAnnotationStatusColor(annotation.status)}"
						>
							{#each annotationStatuses as status (status)}
								<option value={status}>{status.replace('_', ' ')}</option>
							{/each}
						</select>
						<Button
							variant="icon"
							size="sm"
							onclick={(e) => {
								e.stopPropagation();
								startEditingAnnotation(annotation);
							}}
							title="Edit"
						>
							<Icon name="edit" size={12} />
						</Button>
						<Button
							variant="icon"
							size="sm"
							onclick={(e) => {
								e.stopPropagation();
								deleteAnnotation(annotation.id);
							}}
							title="Delete"
						>
							<Icon name="close" size={12} />
						</Button>
					</div>
					{#if editingAnnotationId === annotation.id}
						<div class="annotation-edit" onclick={(e) => e.stopPropagation()} role="presentation">
							<textarea bind:value={editedContent} rows="3" class="annotation-edit-textarea"
							></textarea>
							<div class="annotation-edit-actions">
								<button class="annotation-edit-btn" onclick={cancelEditingAnnotation}>
									Cancel
								</button>
								<button
									class="annotation-edit-btn save"
									onclick={() => saveAnnotationContent(annotation.id)}
								>
									Save
								</button>
							</div>
						</div>
					{:else}
						<p class="annotation-content">{annotation.content}</p>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.annotations-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-sm) var(--spacing-md);
		border-bottom: 1px solid var(--color-border-light);
	}

	.panel-header h3 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
	}

	.filter-select {
		font-size: var(--font-size-xs);
		padding: var(--spacing-xs);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
	}

	.loading,
	.empty-state {
		padding: var(--spacing-lg);
		text-align: center;
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
	}

	.empty-state .hint {
		font-size: var(--font-size-xs);
		margin-top: var(--spacing-sm);
	}

	.annotations-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-sm);
	}

	.annotation-item {
		padding: var(--spacing-sm);
		padding-left: calc(var(--spacing-sm) + 2px);
		margin-bottom: var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border-light);
		border-left: 3px solid var(--annotation-color, var(--color-border-light));
		border-radius: var(--border-radius-sm);
		transition: all var(--transition-fast);
	}

	.annotation-item:hover {
		border-color: var(--color-border);
		border-left-color: var(--annotation-color, var(--color-accent));
	}

	.annotation-item.resolved {
		opacity: 0.6;
	}

	.annotation-header {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		margin-bottom: var(--spacing-xs);
	}

	.annotation-type {
		font-size: var(--font-size-sm);
	}

	.annotation-date {
		flex: 1;
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.status-select {
		font-size: var(--font-size-xs);
		padding: 2px 4px;
		background: transparent;
		border: none;
		font-weight: 500;
		text-transform: capitalize;
	}

	.annotation-content {
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
		line-height: var(--line-height-normal);
	}

	.annotation-edit {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.annotation-edit-textarea {
		width: 100%;
		font-size: var(--font-size-sm);
		padding: var(--spacing-xs);
		border: 1px solid var(--color-accent);
		border-radius: var(--border-radius-sm);
		resize: none;
		font-family: inherit;
	}

	.annotation-edit-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-xs);
	}

	.annotation-edit-btn {
		font-size: var(--font-size-xs);
		padding: 2px var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
	}

	.annotation-edit-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.annotation-edit-btn.save {
		color: var(--color-accent);
		font-weight: 500;
	}

	/* Focus animation triggered by Editor highlight click */
	:global(.annotation-item-focused) {
		animation: annotation-flash 1.5s ease-out;
		box-shadow: 0 0 0 2px var(--annotation-color, var(--color-accent));
	}

	@keyframes annotation-flash {
		0% {
			background-color: rgba(59, 130, 246, 0.15);
			box-shadow: 0 0 0 2px var(--annotation-color, var(--color-accent));
		}
		100% {
			background-color: var(--color-bg-secondary);
			box-shadow: none;
		}
	}
</style>
