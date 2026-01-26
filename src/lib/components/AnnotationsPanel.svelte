<script lang="ts">
	/**
	 * AnnotationsPanel Component
	 *
	 * Side panel for managing scene annotations (comments, questions, TODOs, etc.).
	 * Supports filtering by status and inline creation.
	 */

	import { type Annotation, annotationApi } from '$lib/api';
	import { showError } from '$lib/toast';

	import { Button, FormActions, Icon } from './ui';

	interface Props {
		sceneId: string;
		onSelectAnnotation?: ((annotation: Annotation) => void) | null;
	}

	let { sceneId, onSelectAnnotation = null }: Props = $props();

	let annotations = $state<Annotation[]>([]);
	let isLoading = $state(true);
	let filterStatus = $state('all');
	let showNewForm = $state(false);

	let newContent = $state('');
	let newType = $state('comment');
	let pendingOffsets = $state<{ start: number; end: number } | null>(null);

	const annotationTypes = [
		{ value: 'comment', label: 'Comment', icon: '💬' },
		{ value: 'question', label: 'Question', icon: '❓' },
		{ value: 'todo', label: 'TODO', icon: '✅' },
		{ value: 'research', label: 'Research', icon: '🔍' },
		{ value: 'revision', label: 'Revision', icon: '✏️' },
	];

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

	async function createAnnotation(startOffset: number, endOffset: number) {
		if (!newContent.trim()) return;
		try {
			const annotation = await annotationApi.create({
				scene_id: sceneId,
				start_offset: startOffset,
				end_offset: endOffset,
				annotation_type: newType,
				content: newContent.trim(),
			});
			annotations = [...annotations, annotation];
			resetNewForm();
		} catch (e) {
			console.error('Failed to create annotation:', e);
			showError('Failed to create annotation');
		}
	}

	// Expose this for the editor to call via handle prop
	export async function addAnnotationForSelection(startOffset: number, endOffset: number) {
		showNewForm = true;
		// Store offsets for when form is submitted
		pendingOffsets = { start: startOffset, end: endOffset };
	}

	async function handleSubmitNew() {
		if (pendingOffsets) {
			await createAnnotation(pendingOffsets.start, pendingOffsets.end);
			pendingOffsets = null;
		}
	}

	function resetNewForm() {
		showNewForm = false;
		newContent = '';
		newType = 'comment';
		pendingOffsets = null;
	}

	async function updateAnnotationStatus(annotation: Annotation, status: string) {
		try {
			const updated = await annotationApi.update(annotation.id, { status });
			annotations = annotations.map((a) => (a.id === updated.id ? updated : a));
		} catch (e) {
			console.error('Failed to update annotation:', e);
			showError('Failed to update annotation');
		}
	}

	async function deleteAnnotation(annotationId: string) {
		try {
			await annotationApi.delete(annotationId);
			annotations = annotations.filter((a) => a.id !== annotationId);
		} catch (e) {
			console.error('Failed to delete annotation:', e);
			showError('Failed to delete annotation');
		}
	}

	function getTypeInfo(type: string) {
		return (
			annotationTypes.find((t) => t.value === type) || { value: type, label: type, icon: '📝' }
		);
	}

	function getStatusColor(status: string): string {
		const colors: Record<string, string> = {
			open: 'var(--color-warning)',
			in_progress: 'var(--color-info)',
			resolved: 'var(--color-success)',
		};
		return colors[status] || 'var(--color-text-muted)';
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString();
	}
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
	{:else}
		{#if showNewForm}
			<div class="new-annotation-form">
				<select bind:value={newType} class="type-select">
					{#each annotationTypes as type (type.value)}
						<option value={type.value}>{type.icon} {type.label}</option>
					{/each}
				</select>
				<textarea bind:value={newContent} placeholder="Add your note..." rows="3"></textarea>
				<FormActions>
					<Button variant="ghost" size="sm" onclick={resetNewForm}>Cancel</Button>
					<Button
						variant="primary"
						size="sm"
						onclick={handleSubmitNew}
						disabled={!newContent.trim()}
					>
						Add
					</Button>
				</FormActions>
			</div>
		{/if}

		{#if filteredAnnotations.length === 0}
			<div class="empty-state">
				<p>No annotations yet</p>
				<p class="hint">Select text in the editor and add annotations.</p>
			</div>
		{:else}
			<div class="annotations-list">
				{#each filteredAnnotations as annotation (annotation.id)}
					{@const typeInfo = getTypeInfo(annotation.annotation_type)}
					<div
						class="annotation-item"
						class:resolved={annotation.status === 'resolved'}
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
								style="color: {getStatusColor(annotation.status)}"
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
									deleteAnnotation(annotation.id);
								}}
								title="Delete"
							>
								<Icon name="close" size={12} />
							</Button>
						</div>
						<p class="annotation-content">{annotation.content}</p>
					</div>
				{/each}
			</div>
		{/if}
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

	.new-annotation-form {
		padding: var(--spacing-md);
		border-bottom: 1px solid var(--color-border-light);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.type-select {
		font-size: var(--font-size-sm);
		padding: var(--spacing-xs) var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
	}

	.new-annotation-form textarea {
		font-size: var(--font-size-sm);
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		resize: none;
	}

	.annotations-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-sm);
	}

	.annotation-item {
		padding: var(--spacing-sm);
		margin-bottom: var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border-light);
		border-radius: var(--border-radius-sm);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.annotation-item:hover {
		border-color: var(--color-accent);
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
		cursor: pointer;
	}

	.annotation-content {
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
		line-height: var(--line-height-normal);
	}
</style>
