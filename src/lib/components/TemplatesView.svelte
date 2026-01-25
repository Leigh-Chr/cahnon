<script lang="ts">
	/**
	 * TemplatesView Component
	 *
	 * Manages narrative templates (story structures) like Save the Cat, Hero's Journey, etc.
	 * Supports built-in and custom templates with editable steps.
	 */

	import { onMount } from 'svelte';
	import { templateApi, type Template, type TemplateStep } from '$lib/api';
	import { showSuccess, showError } from '$lib/toast';
	import { DEFAULT_CUSTOM_COLOR } from '$lib/utils';
	import { Icon, Button, Badge, LoadingState, FormActions } from './ui';

	let templates = $state<Template[]>([]);
	let selectedTemplate = $state<Template | null>(null);
	let steps = $state<TemplateStep[]>([]);
	let isLoading = $state(true);

	// Editing state
	let isCreatingTemplate = $state(false);
	let newTemplateName = $state('');
	let isEditingTemplate = $state(false);
	let editTemplateName = $state('');
	let isAddingStep = $state(false);
	let newStepName = $state('');
	let newStepDescription = $state('');
	let newStepPosition = $state(50);
	let newStepColor = $state(DEFAULT_CUSTOM_COLOR);
	let editingStepId = $state<string | null>(null);
	let editStepName = $state('');
	let editStepDescription = $state('');
	let editStepPosition = $state(50);
	let editStepColor = $state(DEFAULT_CUSTOM_COLOR);

	onMount(() => {
		loadTemplates();
	});

	async function loadTemplates() {
		isLoading = true;
		try {
			await templateApi.initBuiltin();
			templates = await templateApi.getAll();
			const active = templates.find((t) => t.is_active) || templates[0];
			if (active) {
				await selectTemplate(active);
			}
		} catch (e) {
			console.error('Failed to load templates:', e);
		}
		isLoading = false;
	}

	async function selectTemplate(template: Template) {
		selectedTemplate = template;
		isEditingTemplate = false;
		editingStepId = null;
		isAddingStep = false;
		try {
			steps = await templateApi.getSteps(template.id);
		} catch (e) {
			console.error('Failed to load template steps:', e);
			steps = [];
		}
	}

	async function setActiveTemplate(template: Template) {
		try {
			await templateApi.setActive(template.id);
			templates = templates.map((t) => ({
				...t,
				is_active: t.id === template.id,
			}));
			showSuccess('Template set as active');
		} catch (e) {
			console.error('Failed to set active template:', e);
			showError('Failed to set active template');
		}
	}

	async function createTemplate() {
		if (!newTemplateName.trim()) return;
		try {
			const template = await templateApi.create(newTemplateName.trim());
			templates = [...templates, template];
			newTemplateName = '';
			isCreatingTemplate = false;
			await selectTemplate(template);
			showSuccess('Template created');
		} catch (e) {
			console.error('Failed to create template:', e);
			showError('Failed to create template');
		}
	}

	async function updateTemplate() {
		if (!selectedTemplate || !editTemplateName.trim()) return;
		try {
			const updated = await templateApi.update(selectedTemplate.id, editTemplateName.trim());
			templates = templates.map((t) => (t.id === updated.id ? updated : t));
			selectedTemplate = updated;
			isEditingTemplate = false;
			showSuccess('Template updated');
		} catch (e) {
			console.error('Failed to update template:', e);
			showError('Failed to update template');
		}
	}

	async function deleteTemplate() {
		if (!selectedTemplate || selectedTemplate.is_builtin) return;
		if (!confirm(`Delete template "${selectedTemplate.name}"?`)) return;
		try {
			await templateApi.delete(selectedTemplate.id);
			templates = templates.filter((t) => t.id !== selectedTemplate!.id);
			selectedTemplate = null;
			steps = [];
			showSuccess('Template deleted');
		} catch (e) {
			console.error('Failed to delete template:', e);
			showError('Failed to delete template');
		}
	}

	async function createStep() {
		if (!selectedTemplate || !newStepName.trim()) return;
		try {
			const step = await templateApi.createStep({
				template_id: selectedTemplate.id,
				name: newStepName.trim(),
				description: newStepDescription.trim() || undefined,
				typical_position: newStepPosition,
				color: newStepColor,
			});
			steps = [...steps, step];
			newStepName = '';
			newStepDescription = '';
			newStepPosition = 50;
			newStepColor = DEFAULT_CUSTOM_COLOR;
			isAddingStep = false;
			showSuccess('Step added');
		} catch (e) {
			console.error('Failed to create step:', e);
			showError('Failed to create step');
		}
	}

	function startEditingStep(step: TemplateStep) {
		editingStepId = step.id;
		editStepName = step.name;
		editStepDescription = step.description || '';
		editStepPosition = step.typical_position;
		editStepColor = step.color || DEFAULT_CUSTOM_COLOR;
	}

	async function updateStep() {
		if (!editingStepId || !editStepName.trim()) return;
		try {
			const updated = await templateApi.updateStep(editingStepId, {
				name: editStepName.trim(),
				description: editStepDescription.trim() || undefined,
				typical_position: editStepPosition,
				color: editStepColor,
			});
			steps = steps.map((s) => (s.id === updated.id ? updated : s));
			editingStepId = null;
			showSuccess('Step updated');
		} catch (e) {
			console.error('Failed to update step:', e);
			showError('Failed to update step');
		}
	}

	async function deleteStep(stepId: string) {
		if (!confirm('Delete this step?')) return;
		try {
			await templateApi.deleteStep(stepId);
			steps = steps.filter((s) => s.id !== stepId);
			showSuccess('Step deleted');
		} catch (e) {
			console.error('Failed to delete step:', e);
			showError('Failed to delete step');
		}
	}

	function getStepColor(step: TemplateStep): string {
		return step.color || 'var(--color-neutral)';
	}

	function formatPosition(position: number): string {
		const positions = [
			'Beginning',
			'Early',
			'First Quarter',
			'Middle',
			'Third Quarter',
			'Late',
			'End',
		];
		const idx = Math.min(Math.floor(position / 15), positions.length - 1);
		return `${Math.round(position)}% - ${positions[idx]}`;
	}
</script>

<div class="templates-view">
	<div class="templates-header">
		<h2>Narrative Templates</h2>
		<p class="subtitle">Use story structures to plan your narrative</p>
	</div>

	{#if isLoading}
		<LoadingState message="Loading templates..." />
	{:else}
		<div class="templates-layout">
			<div class="templates-list">
				<div class="list-header">
					<span>Templates</span>
					<Button
						variant="icon"
						size="sm"
						onclick={() => (isCreatingTemplate = true)}
						title="Create template"
					>
						<Icon name="plus" size={14} />
					</Button>
				</div>

				{#if isCreatingTemplate}
					<div class="new-template-form">
						<input
							type="text"
							bind:value={newTemplateName}
							placeholder="Template name..."
							onkeydown={(e) => e.key === 'Enter' && createTemplate()}
						/>
						<FormActions>
							<Button
								variant="ghost"
								size="sm"
								onclick={() => {
									isCreatingTemplate = false;
									newTemplateName = '';
								}}
							>
								Cancel
							</Button>
							<Button
								variant="primary"
								size="sm"
								onclick={createTemplate}
								disabled={!newTemplateName.trim()}
							>
								Create
							</Button>
						</FormActions>
					</div>
				{/if}

				{#each templates as template (template.id)}
					<button
						class="template-item"
						class:selected={selectedTemplate?.id === template.id}
						class:active={template.is_active}
						onclick={() => selectTemplate(template)}
					>
						<div class="template-name">
							{template.name}
							{#if template.is_active}
								<Badge variant="success" size="sm">Active</Badge>
							{/if}
						</div>
						{#if template.is_builtin}
							<span class="builtin-badge">Built-in</span>
						{:else}
							<span class="custom-badge">Custom</span>
						{/if}
					</button>
				{/each}
			</div>

			<div class="template-detail">
				{#if selectedTemplate}
					<div class="detail-header">
						{#if isEditingTemplate && !selectedTemplate.is_builtin}
							<input
								type="text"
								class="edit-name-input"
								bind:value={editTemplateName}
								onkeydown={(e) => e.key === 'Enter' && updateTemplate()}
							/>
							<div class="header-actions">
								<Button variant="ghost" size="sm" onclick={() => (isEditingTemplate = false)}>
									Cancel
								</Button>
								<Button variant="primary" size="sm" onclick={updateTemplate}>Save</Button>
							</div>
						{:else}
							<h3>{selectedTemplate.name}</h3>
							<div class="header-actions">
								{#if !selectedTemplate.is_builtin}
									<Button
										variant="icon"
										title="Edit template"
										onclick={() => {
											isEditingTemplate = true;
											editTemplateName = selectedTemplate!.name;
										}}
									>
										<Icon name="edit" size={14} />
									</Button>
									<Button
										variant="icon"
										class="danger"
										title="Delete template"
										onclick={deleteTemplate}
									>
										<Icon name="delete" size={14} />
									</Button>
								{/if}
								{#if !selectedTemplate.is_active}
									<Button
										variant="primary"
										size="sm"
										onclick={() => setActiveTemplate(selectedTemplate!)}
									>
										Set as Active
									</Button>
								{/if}
							</div>
						{/if}
					</div>

					<div class="steps-list">
						{#each steps as step (step.id)}
							<div class="step-item" style="--step-color: {getStepColor(step)}">
								<div class="step-marker">
									<div class="step-dot"></div>
									<div class="step-line"></div>
								</div>
								{#if editingStepId === step.id && !selectedTemplate.is_builtin}
									<div class="step-edit-form">
										<input type="text" bind:value={editStepName} placeholder="Step name" />
										<textarea
											bind:value={editStepDescription}
											placeholder="Description (optional)"
											rows="2"
										></textarea>
										<div class="step-form-row">
											<label>
												Position:
												<input type="range" min="0" max="100" bind:value={editStepPosition} />
												<span>{Math.round(editStepPosition)}%</span>
											</label>
											<label>
												Color:
												<input type="color" bind:value={editStepColor} />
											</label>
										</div>
										<FormActions>
											<Button variant="ghost" size="sm" onclick={() => (editingStepId = null)}>
												Cancel
											</Button>
											<Button variant="primary" size="sm" onclick={updateStep}>Save</Button>
										</FormActions>
									</div>
								{:else}
									<div class="step-content">
										<div class="step-header">
											<span class="step-name">{step.name}</span>
											<span class="step-position">{formatPosition(step.typical_position)}</span>
											{#if !selectedTemplate.is_builtin}
												<div class="step-actions">
													<Button
														variant="icon"
														size="sm"
														onclick={() => startEditingStep(step)}
														title="Edit step"
													>
														<Icon name="edit" size={12} />
													</Button>
													<Button
														variant="icon"
														size="sm"
														class="danger"
														onclick={() => deleteStep(step.id)}
														title="Delete step"
													>
														<Icon name="close" size={12} />
													</Button>
												</div>
											{/if}
										</div>
										{#if step.description}
											<p class="step-description">{step.description}</p>
										{/if}
									</div>
								{/if}
							</div>
						{/each}

						{#if !selectedTemplate.is_builtin}
							{#if isAddingStep}
								<div class="add-step-form">
									<input type="text" bind:value={newStepName} placeholder="Step name" />
									<textarea
										bind:value={newStepDescription}
										placeholder="Description (optional)"
										rows="2"
									></textarea>
									<div class="step-form-row">
										<label>
											Position:
											<input type="range" min="0" max="100" bind:value={newStepPosition} />
											<span>{newStepPosition}%</span>
										</label>
										<label>
											Color:
											<input type="color" bind:value={newStepColor} />
										</label>
									</div>
									<FormActions>
										<Button
											variant="ghost"
											size="sm"
											onclick={() => {
												isAddingStep = false;
												newStepName = '';
												newStepDescription = '';
											}}
										>
											Cancel
										</Button>
										<Button
											variant="primary"
											size="sm"
											onclick={createStep}
											disabled={!newStepName.trim()}
										>
											Add Step
										</Button>
									</FormActions>
								</div>
							{:else}
								<button class="add-step-btn" onclick={() => (isAddingStep = true)}>
									<Icon name="plus" size={14} />
									Add Step
								</button>
							{/if}
						{/if}
					</div>

					<div class="template-info">
						<p class="info-text">
							Assign scenes to template steps in the scene editor to track your story's structure.
						</p>
					</div>
				{:else}
					<div class="no-selection">
						<p>Select a template to view its structure</p>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.templates-view {
		height: 100%;
		display: flex;
		flex-direction: column;
		background-color: var(--color-bg-primary);
	}

	.templates-header {
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
	}

	.templates-header h2 {
		font-size: var(--font-size-lg);
		font-weight: 600;
		margin-bottom: var(--spacing-xs);
	}

	.subtitle {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.templates-layout {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.templates-list {
		width: 260px;
		border-right: 1px solid var(--color-border);
		overflow-y: auto;
	}

	.list-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md);
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
		background-color: var(--color-bg-secondary);
		position: sticky;
		top: 0;
	}

	.template-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: var(--spacing-md);
		text-align: left;
		border-bottom: 1px solid var(--color-border-light);
		transition: background-color var(--transition-fast);
	}

	.template-item:hover {
		background-color: var(--color-bg-hover);
	}

	.template-item.selected {
		background-color: var(--color-accent-light);
	}

	.template-name {
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.builtin-badge {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.custom-badge {
		font-size: var(--font-size-xs);
		color: var(--color-accent);
	}

	.template-detail {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-lg);
	}

	.detail-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--spacing-lg);
	}

	.detail-header h3 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	/* Danger style for icon buttons */
	.header-actions :global(.btn-icon.danger:hover) {
		color: var(--color-error);
	}

	.steps-list {
		display: flex;
		flex-direction: column;
	}

	.step-item {
		display: flex;
		gap: var(--spacing-md);
		position: relative;
	}

	.step-marker {
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 20px;
		flex-shrink: 0;
	}

	.step-dot {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background-color: var(--step-color);
		flex-shrink: 0;
	}

	.step-line {
		width: 2px;
		flex: 1;
		background-color: var(--color-border);
		margin-top: var(--spacing-xs);
	}

	.step-item:last-child .step-line {
		display: none;
	}

	.step-content {
		flex: 1;
		padding-bottom: var(--spacing-lg);
	}

	.step-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--spacing-xs);
	}

	.step-name {
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.step-position {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.step-actions {
		display: flex;
		gap: var(--spacing-xs);
		opacity: 0;
		transition: opacity var(--transition-fast);
	}

	.step-item:hover .step-actions {
		opacity: 1;
	}

	.step-actions :global(.btn-icon.danger:hover) {
		color: var(--color-error);
	}

	.step-description {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		line-height: var(--line-height-normal);
	}

	.step-edit-form {
		flex: 1;
		padding-bottom: var(--spacing-md);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.template-info {
		margin-top: var(--spacing-lg);
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-md);
	}

	.info-text {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
	}

	.no-selection {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--color-text-muted);
	}

	.new-template-form,
	.add-step-form {
		padding: var(--spacing-md);
		border-bottom: 1px solid var(--color-border-light);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.new-template-form input,
	.add-step-form input[type='text'],
	.step-edit-form input[type='text'],
	.edit-name-input {
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		width: 100%;
		background-color: var(--color-bg-primary);
		color: var(--color-text-primary);
	}

	.add-step-form textarea,
	.step-edit-form textarea {
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		resize: vertical;
		width: 100%;
		background-color: var(--color-bg-primary);
		color: var(--color-text-primary);
	}

	.step-form-row {
		display: flex;
		align-items: center;
		gap: var(--spacing-lg);
	}

	.step-form-row label {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
	}

	.step-form-row input[type='range'] {
		width: 100px;
	}

	.step-form-row input[type='color'] {
		width: 32px;
		height: 24px;
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		cursor: pointer;
	}

	.edit-name-input {
		font-size: var(--font-size-lg);
		font-weight: 600;
		flex: 1;
	}

	.add-step-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-md);
		width: 100%;
		color: var(--color-text-muted);
		border: 2px dashed var(--color-border);
		border-radius: var(--border-radius-md);
		margin-top: var(--spacing-md);
		font-size: var(--font-size-sm);
	}

	.add-step-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
		background-color: var(--color-accent-light);
	}
</style>
