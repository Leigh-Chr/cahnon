<!--
  Narrative template management dialog.

  Features:
  - View available templates (built-in and custom)
  - Activate a template for the project
  - Create custom templates
  - Edit template steps (add, edit, delete, reorder)
  - Initialize built-in templates
  - View template step assignments
-->
<script lang="ts">
	import { type Template, templateApi, type TemplateStep } from '$lib/api';
	import { appState } from '$lib/stores';
	import { showError } from '$lib/toast';
	import { nativeConfirm } from '$lib/utils/native-dialog';

	import { Button, EmptyState, FormActions, FormGroup, Icon } from './ui';

	interface Props {
		isOpen: boolean;
		onclose: () => void;
	}

	let { isOpen, onclose }: Props = $props();

	let templates = $state<Template[]>([]);
	let steps = $state<TemplateStep[]>([]);
	let selectedTemplateId = $state<string | null>(null);
	let isLoading = $state(false);

	// Form states
	let isCreatingTemplate = $state(false);
	let isEditingStep = $state(false);
	let editingStepId = $state<string | null>(null);

	// Template form
	let newTemplateName = $state('');

	// Rename state
	let isRenaming = $state(false);
	let renameValue = $state('');

	// Step form
	let stepName = $state('');
	let stepDescription = $state('');
	let stepTypicalPosition = $state(50);
	let stepColor = $state('#6366f1');

	const stepColors = [
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

	let selectedTemplate = $derived(
		selectedTemplateId ? templates.find((t) => t.id === selectedTemplateId) || null : null
	);

	let activeTemplate = $derived(templates.find((t) => t.is_active) || null);

	$effect(() => {
		if (isOpen && appState.project) {
			loadTemplates();
		}
	});

	$effect(() => {
		if (selectedTemplateId) {
			loadSteps(selectedTemplateId);
		}
	});

	let hasTriedInit = false;

	async function loadTemplates() {
		isLoading = true;
		try {
			templates = await templateApi.getAll();
			// If no templates and we haven't already tried, initialize built-ins
			if (templates.length === 0 && !hasTriedInit) {
				await initBuiltinTemplates();
			}
		} catch (e) {
			console.error('Failed to load templates:', e);
			showError('Failed to load templates');
		} finally {
			isLoading = false;
		}
	}

	async function loadSteps(templateId: string) {
		try {
			steps = await templateApi.getSteps(templateId);
		} catch (e) {
			console.error('Failed to load steps:', e);
			showError('Failed to load template steps');
			steps = [];
		}
	}

	async function initBuiltinTemplates() {
		hasTriedInit = true;
		try {
			await templateApi.initBuiltin();
			templates = await templateApi.getAll();
		} catch (e) {
			console.error('Failed to initialize built-in templates:', e);
			showError('Failed to initialize templates');
		}
	}

	async function activateTemplate(templateId: string) {
		try {
			await templateApi.setActive(templateId);
			templates = templates.map((t) => ({
				...t,
				is_active: t.id === templateId,
			}));
		} catch (e) {
			console.error('Failed to activate template:', e);
			showError('Failed to activate template');
		}
	}

	async function createTemplate() {
		if (!newTemplateName.trim()) return;

		try {
			const newTemplate = await templateApi.create(newTemplateName.trim());
			templates = [...templates, newTemplate];
			selectedTemplateId = newTemplate.id;
			newTemplateName = '';
			isCreatingTemplate = false;
		} catch (e) {
			console.error('Failed to create template:', e);
			showError('Failed to create template');
		}
	}

	async function deleteTemplate(templateId: string) {
		const template = templates.find((t) => t.id === templateId);
		if (!template) return;

		if (template.is_builtin) {
			showError('Cannot delete built-in templates');
			return;
		}

		if (
			!(await nativeConfirm(
				`Delete template "${template.name}"? This action cannot be undone.`,
				'Delete Template'
			))
		)
			return;

		try {
			await templateApi.delete(templateId);
			templates = templates.filter((t) => t.id !== templateId);
			if (selectedTemplateId === templateId) {
				selectedTemplateId = null;
				steps = [];
			}
		} catch (e) {
			console.error('Failed to delete template:', e);
			showError('Failed to delete template');
		}
	}

	function startEditStep(step?: TemplateStep) {
		if (step) {
			editingStepId = step.id;
			stepName = step.name;
			stepDescription = step.description || '';
			stepTypicalPosition = step.typical_position;
			stepColor = step.color || '#6366f1';
		} else {
			editingStepId = null;
			stepName = '';
			stepDescription = '';
			stepTypicalPosition = 50;
			stepColor = '#6366f1';
		}
		isEditingStep = true;
	}

	function cancelEditStep() {
		isEditingStep = false;
		editingStepId = null;
		stepName = '';
		stepDescription = '';
		stepTypicalPosition = 50;
		stepColor = '#6366f1';
	}

	async function saveStep() {
		if (!stepName.trim() || !selectedTemplateId) return;

		try {
			if (editingStepId) {
				// Update existing step
				const updated = await templateApi.updateStep(editingStepId, {
					name: stepName.trim(),
					description: stepDescription.trim() || undefined,
					typical_position: stepTypicalPosition,
					color: stepColor,
				});
				steps = steps.map((s) => (s.id === updated.id ? updated : s));
			} else {
				// Create new step
				const newStep = await templateApi.createStep({
					template_id: selectedTemplateId,
					name: stepName.trim(),
					description: stepDescription.trim() || undefined,
					typical_position: stepTypicalPosition,
					color: stepColor,
				});
				steps = [...steps, newStep];
			}
			cancelEditStep();
		} catch (e) {
			console.error('Failed to save step:', e);
			showError('Failed to save step');
		}
	}

	async function deleteStep(stepId: string) {
		if (!(await nativeConfirm('Delete this step?', 'Delete Step'))) return;

		try {
			await templateApi.deleteStep(stepId);
			steps = steps.filter((s) => s.id !== stepId);
		} catch (e) {
			console.error('Failed to delete step:', e);
			showError('Failed to delete step');
		}
	}

	function startRenaming() {
		if (!selectedTemplate || selectedTemplate.is_builtin) return;
		renameValue = selectedTemplate.name;
		isRenaming = true;
	}

	async function finishRenaming() {
		if (!selectedTemplate || !renameValue.trim()) {
			isRenaming = false;
			return;
		}
		try {
			const updated = await templateApi.update(selectedTemplate.id, renameValue.trim());
			templates = templates.map((t) => (t.id === updated.id ? updated : t));
			isRenaming = false;
		} catch (e) {
			console.error('Failed to rename template:', e);
			showError('Failed to rename template');
		}
	}

	function cancelRenaming() {
		isRenaming = false;
	}

	function handleOverlayClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			onclose();
		}
	}
</script>

{#if isOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="modal-overlay" onclick={handleOverlayClick} role="presentation">
		<div
			class="modal-container"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="templates-title"
			tabindex="-1"
		>
			<div class="modal-header">
				<h2 id="templates-title">Narrative Templates</h2>
				<button class="close-btn" onclick={onclose} aria-label="Close">
					<Icon name="close" size={20} />
				</button>
			</div>

			<div class="modal-body">
				<div class="templates-sidebar">
					<div class="sidebar-header">
						<Button size="sm" variant="primary" onclick={() => (isCreatingTemplate = true)}>
							<Icon name="plus" size={14} />
							New Template
						</Button>
					</div>

					{#if activeTemplate}
						<div class="active-template-badge">
							<span class="badge-label">Active:</span>
							<span class="badge-name">{activeTemplate.name}</span>
						</div>
					{/if}

					<div class="templates-list">
						{#if isLoading}
							<div class="loading">Loading templates...</div>
						{:else if templates.length === 0}
							<EmptyState title="No templates" />
						{:else}
							{#each templates as template (template.id)}
								<button
									class="template-item"
									class:selected={selectedTemplateId === template.id}
									class:active={template.is_active}
									onclick={() => {
										selectedTemplateId = template.id;
										isCreatingTemplate = false;
										isEditingStep = false;
									}}
								>
									<div class="template-info">
										<span class="template-name">
											{template.name}
											{#if template.is_builtin}
												<span class="builtin-badge">Built-in</span>
											{/if}
										</span>
										{#if template.is_active}
											<span class="active-badge">Active</span>
										{/if}
									</div>
								</button>
							{/each}
						{/if}
					</div>
				</div>

				<div class="templates-detail">
					{#if isCreatingTemplate}
						<form
							onsubmit={(e) => {
								e.preventDefault();
								createTemplate();
							}}
						>
							<h3>Create New Template</h3>

							<FormGroup label="Template Name">
								<input
									type="text"
									bind:value={newTemplateName}
									placeholder="My Custom Template"
									required
								/>
							</FormGroup>

							<FormActions>
								<Button onclick={() => (isCreatingTemplate = false)}>Cancel</Button>
								<Button variant="primary" type="submit">Create Template</Button>
							</FormActions>
						</form>
					{:else if isEditingStep}
						<form
							onsubmit={(e) => {
								e.preventDefault();
								saveStep();
							}}
						>
							<h3>{editingStepId ? 'Edit Step' : 'Add New Step'}</h3>

							<FormGroup label="Step Name">
								<input
									type="text"
									bind:value={stepName}
									placeholder="e.g., Inciting Incident"
									required
								/>
							</FormGroup>

							<FormGroup label="Description">
								<textarea
									bind:value={stepDescription}
									rows="3"
									placeholder="What happens at this beat?"
								></textarea>
							</FormGroup>

							<FormGroup label="Typical Position ({stepTypicalPosition}%)">
								<input type="range" min="0" max="100" bind:value={stepTypicalPosition} />
								<div class="position-labels">
									<span>Start</span>
									<span>Middle</span>
									<span>End</span>
								</div>
							</FormGroup>

							<FormGroup label="Color">
								<div class="color-picker">
									{#each stepColors as color (color)}
										<button
											type="button"
											class="color-option"
											class:selected={stepColor === color}
											style="background-color: {color}"
											onclick={() => (stepColor = color)}
											aria-label="Select color"
										></button>
									{/each}
								</div>
							</FormGroup>

							<FormActions>
								<Button onclick={cancelEditStep}>Cancel</Button>
								<Button variant="primary" type="submit">
									{editingStepId ? 'Save Changes' : 'Add Step'}
								</Button>
							</FormActions>
						</form>
					{:else if selectedTemplate}
						<div class="template-detail">
							<div class="detail-header">
								{#if isRenaming}
									<input
										type="text"
										class="rename-input"
										bind:value={renameValue}
										onkeydown={(e) => {
											if (e.key === 'Enter') finishRenaming();
											if (e.key === 'Escape') cancelRenaming();
										}}
										onblur={finishRenaming}
									/>
								{:else}
									<h3
										ondblclick={() => {
											if (selectedTemplate && !selectedTemplate.is_builtin) startRenaming();
										}}
									>
										{selectedTemplate.name}
									</h3>
								{/if}
								<div class="detail-actions">
									{#if !selectedTemplate.is_active}
										<Button
											size="sm"
											variant="primary"
											onclick={() => {
												if (selectedTemplate) activateTemplate(selectedTemplate.id);
											}}
										>
											Activate
										</Button>
									{/if}
									{#if !selectedTemplate.is_builtin}
										<Button size="sm" onclick={startRenaming}>Rename</Button>
										<Button
											size="sm"
											onclick={() => {
												if (selectedTemplate) deleteTemplate(selectedTemplate.id);
											}}
										>
											<Icon name="trash" size={14} />
											Delete
										</Button>
									{/if}
								</div>
							</div>

							{#if selectedTemplate.is_builtin}
								<p class="builtin-notice">
									This is a built-in template. You can view its steps but not modify them.
								</p>
							{/if}

							<div class="steps-section">
								<div class="steps-header">
									<h4>Steps ({steps.length})</h4>
									{#if !selectedTemplate.is_builtin}
										<Button size="sm" onclick={() => startEditStep()}>
											<Icon name="plus" size={14} />
											Add Step
										</Button>
									{/if}
								</div>

								<div class="steps-list">
									{#each [...steps].sort((a, b) => a.typical_position - b.typical_position) as step (step.id)}
										<div class="step-item" style="--step-color: {step.color || '#6366f1'}">
											<div class="step-position">{step.typical_position}%</div>
											<div class="step-color-bar"></div>
											<div class="step-content">
												<span class="step-name">{step.name}</span>
												{#if step.description}
													<span class="step-description">{step.description}</span>
												{/if}
											</div>
											{#if !selectedTemplate.is_builtin}
												<div class="step-actions">
													<Button variant="icon" size="sm" onclick={() => startEditStep(step)}>
														<Icon name="edit" size={14} />
													</Button>
													<Button variant="icon" size="sm" onclick={() => deleteStep(step.id)}>
														<Icon name="trash" size={14} />
													</Button>
												</div>
											{/if}
										</div>
									{:else}
										<p class="empty-message">No steps defined for this template.</p>
									{/each}
								</div>
							</div>
						</div>
					{:else}
						<EmptyState title="Select a template or create a new one" />
					{/if}
				</div>
			</div>
		</div>
	</div>
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

	.templates-sidebar {
		width: 280px;
		border-right: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
	}

	.sidebar-header {
		padding: var(--spacing-md);
		border-bottom: 1px solid var(--color-border);
	}

	.active-template-badge {
		padding: var(--spacing-sm) var(--spacing-md);
		background-color: var(--color-accent-light);
		border-bottom: 1px solid var(--color-border);
		font-size: var(--font-size-sm);
	}

	.badge-label {
		color: var(--color-text-secondary);
	}

	.badge-name {
		font-weight: 500;
		color: var(--color-accent);
	}

	.templates-list {
		flex: 1;
		overflow-y: auto;
	}

	.loading {
		padding: var(--spacing-lg);
		text-align: center;
		color: var(--color-text-secondary);
	}

	.template-item {
		width: 100%;
		display: flex;
		align-items: center;
		padding: var(--spacing-sm) var(--spacing-md);
		border-bottom: 1px solid var(--color-border-light);
		text-align: left;
		background: none;
		transition: background-color var(--transition-fast);
	}

	.template-item:hover {
		background-color: var(--color-bg-hover);
	}

	.template-item.selected {
		background-color: var(--color-accent-light);
	}

	.template-info {
		flex: 1;
	}

	.template-name {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-weight: 500;
	}

	.builtin-badge {
		font-size: var(--font-size-xs);
		padding: 1px 6px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
		font-weight: normal;
	}

	.active-badge {
		display: inline-block;
		font-size: var(--font-size-xs);
		padding: 1px 6px;
		background-color: var(--color-success-light);
		color: var(--color-success);
		border-radius: var(--border-radius-sm);
	}

	.templates-detail {
		flex: 1;
		padding: var(--spacing-lg);
		overflow-y: auto;
	}

	.templates-detail h3 {
		margin: 0 0 var(--spacing-lg) 0;
		font-size: var(--font-size-lg);
	}

	.templates-detail form {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
		max-width: 500px;
	}

	.templates-detail input,
	.templates-detail textarea {
		width: 100%;
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
		font-size: var(--font-size-base);
	}

	.templates-detail textarea {
		resize: vertical;
	}

	.templates-detail input[type='range'] {
		padding: 0;
		margin: var(--spacing-sm) 0;
	}

	.position-labels {
		display: flex;
		justify-content: space-between;
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.color-picker {
		display: flex;
		gap: var(--spacing-xs);
		flex-wrap: wrap;
	}

	.color-option {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		border: 2px solid transparent;
		transition: transform var(--transition-fast);
	}

	.color-option:hover {
		transform: scale(1.1);
	}

	.color-option.selected {
		border-color: var(--color-text-primary);
	}

	.template-detail {
		max-width: 600px;
	}

	.detail-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--spacing-md);
	}

	.detail-header h3 {
		margin: 0;
	}

	.rename-input {
		font-size: var(--font-size-lg);
		font-weight: 600;
		border: none;
		border-bottom: 2px solid var(--color-accent);
		background: none;
		padding: 0;
		outline: none;
		flex: 1;
		min-width: 0;
	}

	.detail-actions {
		display: flex;
		gap: var(--spacing-xs);
	}

	.builtin-notice {
		padding: var(--spacing-sm);
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-lg);
	}

	.steps-section {
		margin-top: var(--spacing-lg);
	}

	.steps-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--spacing-md);
	}

	.steps-header h4 {
		margin: 0;
		font-size: var(--font-size-base);
		font-weight: 600;
	}

	.steps-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.step-item {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-sm);
		border-left: 4px solid var(--step-color);
	}

	.step-position {
		font-size: var(--font-size-xs);
		font-weight: 600;
		color: var(--color-text-secondary);
		min-width: 35px;
		text-align: right;
	}

	.step-color-bar {
		width: 4px;
		height: 100%;
		min-height: 30px;
		background-color: var(--step-color);
		border-radius: 2px;
	}

	.step-content {
		flex: 1;
	}

	.step-name {
		display: block;
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.step-description {
		display: block;
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		margin-top: 2px;
	}

	.step-actions {
		display: flex;
		gap: 2px;
		opacity: 0;
		transition: opacity var(--transition-fast);
	}

	.step-item:hover .step-actions {
		opacity: 1;
	}

	.empty-message {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		font-style: italic;
	}
</style>
