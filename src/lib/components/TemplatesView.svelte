<script lang="ts">
	import { onMount } from 'svelte';
	import { templateApi, type Template, type TemplateStep } from '$lib/api';
	import { showSuccess, showError } from '$lib/toast';

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
	let newStepColor = $state('#525252');
	let editingStepId = $state<string | null>(null);
	let editStepName = $state('');
	let editStepDescription = $state('');
	let editStepPosition = $state(50);
	let editStepColor = $state('#525252');

	// Use onMount for one-time initialization
	onMount(() => {
		loadTemplates();
	});

	async function loadTemplates() {
		isLoading = true;
		try {
			// Initialize builtin templates if needed
			await templateApi.initBuiltin();
			templates = await templateApi.getAll();
			// Auto-select active template or first one
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
			newStepColor = '#525252';
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
		editStepColor = step.color || '#525252';
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
		return step.color || '#525252';
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
		<div class="loading">Loading templates...</div>
	{:else}
		<div class="templates-layout">
			<div class="templates-list">
				<div class="list-header">
					<span>Templates</span>
					<button
						class="add-btn"
						onclick={() => (isCreatingTemplate = true)}
						title="Create template"
					>
						<svg
							width="14"
							height="14"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<line x1="12" y1="5" x2="12" y2="19" />
							<line x1="5" y1="12" x2="19" y2="12" />
						</svg>
					</button>
				</div>

				{#if isCreatingTemplate}
					<div class="new-template-form">
						<input
							type="text"
							bind:value={newTemplateName}
							placeholder="Template name..."
							onkeydown={(e) => e.key === 'Enter' && createTemplate()}
						/>
						<div class="form-actions">
							<button
								class="cancel-btn"
								onclick={() => {
									isCreatingTemplate = false;
									newTemplateName = '';
								}}>Cancel</button
							>
							<button class="save-btn" onclick={createTemplate} disabled={!newTemplateName.trim()}
								>Create</button
							>
						</div>
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
								<span class="active-badge">Active</span>
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
								<button class="cancel-btn" onclick={() => (isEditingTemplate = false)}
									>Cancel</button
								>
								<button class="save-btn" onclick={updateTemplate}>Save</button>
							</div>
						{:else}
							<h3>{selectedTemplate.name}</h3>
							<div class="header-actions">
								{#if !selectedTemplate.is_builtin}
									<button
										class="edit-btn"
										aria-label="Edit template"
										onclick={() => {
											isEditingTemplate = true;
											editTemplateName = selectedTemplate!.name;
										}}
									>
										<svg
											width="14"
											height="14"
											viewBox="0 0 24 24"
											fill="none"
											stroke="currentColor"
											stroke-width="2"
										>
											<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
											<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
										</svg>
									</button>
									<button class="delete-btn" aria-label="Delete template" onclick={deleteTemplate}>
										<svg
											width="14"
											height="14"
											viewBox="0 0 24 24"
											fill="none"
											stroke="currentColor"
											stroke-width="2"
										>
											<polyline points="3 6 5 6 21 6" />
											<path
												d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
											/>
										</svg>
									</button>
								{/if}
								{#if !selectedTemplate.is_active}
									<button class="activate-btn" onclick={() => setActiveTemplate(selectedTemplate!)}>
										Set as Active
									</button>
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
										<div class="form-actions">
											<button class="cancel-btn" onclick={() => (editingStepId = null)}
												>Cancel</button
											>
											<button class="save-btn" onclick={updateStep}>Save</button>
										</div>
									</div>
								{:else}
									<div class="step-content">
										<div class="step-header">
											<span class="step-name">{step.name}</span>
											<span class="step-position">{formatPosition(step.typical_position)}</span>
											{#if !selectedTemplate.is_builtin}
												<div class="step-actions">
													<button
														class="step-edit-btn"
														onclick={() => startEditingStep(step)}
														title="Edit step"
													>
														<svg
															width="12"
															height="12"
															viewBox="0 0 24 24"
															fill="none"
															stroke="currentColor"
															stroke-width="2"
														>
															<path
																d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
															/>
															<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
														</svg>
													</button>
													<button
														class="step-delete-btn"
														onclick={() => deleteStep(step.id)}
														title="Delete step"
													>
														<svg
															width="12"
															height="12"
															viewBox="0 0 24 24"
															fill="none"
															stroke="currentColor"
															stroke-width="2"
														>
															<line x1="18" y1="6" x2="6" y2="18" />
															<line x1="6" y1="6" x2="18" y2="18" />
														</svg>
													</button>
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
									<div class="form-actions">
										<button
											class="cancel-btn"
											onclick={() => {
												isAddingStep = false;
												newStepName = '';
												newStepDescription = '';
											}}>Cancel</button
										>
										<button class="save-btn" onclick={createStep} disabled={!newStepName.trim()}
											>Add Step</button
										>
									</div>
								</div>
							{:else}
								<button class="add-step-btn" onclick={() => (isAddingStep = true)}>
									<svg
										width="14"
										height="14"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"
									>
										<line x1="12" y1="5" x2="12" y2="19" />
										<line x1="5" y1="12" x2="19" y2="12" />
									</svg>
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

	.loading {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 1;
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

	.active-badge {
		font-size: var(--font-size-xs);
		padding: 2px 6px;
		background-color: var(--color-success);
		color: white;
		border-radius: var(--border-radius-sm);
		font-weight: 500;
	}

	.builtin-badge {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
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

	.activate-btn {
		padding: var(--spacing-xs) var(--spacing-md);
		background-color: var(--color-accent);
		color: white;
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		font-weight: 500;
	}

	.activate-btn:hover {
		background-color: var(--color-accent-hover);
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

	.step-description {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		line-height: var(--line-height-normal);
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

	.list-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.add-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.add-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-accent);
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
	}

	.add-step-form textarea,
	.step-edit-form textarea {
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		resize: vertical;
		width: 100%;
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

	.form-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-sm);
	}

	.cancel-btn {
		padding: var(--spacing-xs) var(--spacing-md);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		border-radius: var(--border-radius-sm);
	}

	.cancel-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.save-btn {
		padding: var(--spacing-xs) var(--spacing-md);
		font-size: var(--font-size-sm);
		background-color: var(--color-accent);
		color: white;
		border-radius: var(--border-radius-sm);
	}

	.save-btn:hover:not(:disabled) {
		background-color: var(--color-accent-hover);
	}

	.save-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.custom-badge {
		font-size: var(--font-size-xs);
		color: var(--color-accent);
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.edit-btn,
	.delete-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.edit-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-accent);
	}

	.delete-btn:hover {
		background-color: rgba(239, 68, 68, 0.1);
		color: var(--color-error);
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

	.step-edit-btn,
	.step-delete-btn {
		padding: 2px;
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.step-edit-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-accent);
	}

	.step-delete-btn:hover {
		background-color: rgba(239, 68, 68, 0.1);
		color: var(--color-error);
	}

	.step-edit-form {
		flex: 1;
		padding-bottom: var(--spacing-md);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
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

	.edit-name-input {
		font-size: var(--font-size-lg);
		font-weight: 600;
		flex: 1;
	}
</style>
