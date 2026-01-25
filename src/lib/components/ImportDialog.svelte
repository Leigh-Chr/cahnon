<script lang="ts">
	import { appState } from '$lib/stores';
	import { importApi } from '$lib/api';
	import { showSuccess, showError } from '$lib/toast';
	import { importFromDocx } from '$lib/export';
	import { Icon, Button } from './ui';

	interface Props {
		isOpen?: boolean;
		onclose?: () => void;
	}

	let { isOpen = $bindable(false), onclose }: Props = $props();

	type ImportMode = 'structured' | 'single-scene' | 'docx' | 'json-backup';
	type FileType = 'markdown' | 'text' | 'docx' | 'json';

	let importMode = $state<ImportMode>('structured');
	let fileType = $state<FileType>('markdown');
	let content = $state('');
	let sceneTitle = $state('Imported Scene');
	let selectedChapterId = $state('');
	let isImporting = $state(false);
	let result = $state<{ chapters: number; scenes: number } | null>(null);
	let error = $state<string | null>(null);
	let docxFile = $state<File | null>(null);

	$effect(() => {
		if (appState.chapters.length > 0 && !selectedChapterId) {
			selectedChapterId = appState.chapters[0].id;
		}
	});

	function handleFileSelect(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		// Auto-detect file type
		if (file.name.endsWith('.docx')) {
			fileType = 'docx';
			importMode = 'docx';
			docxFile = file;
			content = `DOCX file selected: ${file.name}`;
		} else if (file.name.endsWith('.json')) {
			fileType = 'json';
			importMode = 'json-backup';
			docxFile = null;
			const reader = new FileReader();
			reader.onload = (e) => {
				content = (e.target?.result as string) || '';
			};
			reader.readAsText(file);
		} else {
			docxFile = null;
			const reader = new FileReader();
			reader.onload = (e) => {
				content = (e.target?.result as string) || '';
			};
			reader.readAsText(file);

			if (file.name.endsWith('.md') || file.name.endsWith('.markdown')) {
				fileType = 'markdown';
			} else {
				fileType = 'text';
			}
		}

		// Use filename as scene title
		sceneTitle = file.name.replace(/\.(md|markdown|txt|docx|json)$/i, '');
	}

	async function handleImport() {
		if (importMode === 'docx') {
			if (!docxFile) {
				error = 'Please select a DOCX file';
				return;
			}
		} else if (importMode === 'json-backup') {
			if (!content.trim()) {
				error = 'Please select a JSON backup file';
				return;
			}
			// Confirm before importing (it's destructive)
			if (
				!confirm(
					'Are you sure you want to import this JSON backup?\n\n' +
						'This will REPLACE all current project data (chapters, scenes, bible entries, arcs, events).\n\n' +
						'An automatic backup will be created before importing.'
				)
			) {
				return;
			}
		} else if (!content.trim()) {
			error = 'Please provide content to import';
			return;
		}

		isImporting = true;
		error = null;
		result = null;

		try {
			if (importMode === 'json-backup') {
				// Import JSON backup
				await importApi.jsonBackup(content);
				// Reload all project data
				await appState.loadManuscript();
				await appState.loadBible();
				await appState.loadStats();
				result = { chapters: -1, scenes: -1 }; // Special marker for "full restore"
				showSuccess('JSON backup imported successfully');
			} else if (importMode === 'docx') {
				// Import DOCX file
				const docxResult = await importFromDocx(docxFile!);
				let chaptersCreated = 0;
				let scenesCreated = 0;

				for (const chapter of docxResult.chapters) {
					// Create chapter
					const newChapter = await appState.createChapter(chapter.title);
					chaptersCreated++;

					// Create scenes in that chapter
					for (const scene of chapter.scenes) {
						const createdScene = await appState.createScene(newChapter.id, scene.title);
						// Update scene with content
						if (createdScene) {
							await appState.updateScene(createdScene.id, { text: scene.text });
						}
						scenesCreated++;
					}
				}

				result = { chapters: chaptersCreated, scenes: scenesCreated };
				await appState.loadChapters();
				showSuccess(
					`Imported ${chaptersCreated} chapter(s) and ${scenesCreated} scene(s) from DOCX`
				);
			} else if (importMode === 'structured') {
				const importResult = await importApi.markdownStructured(content);
				result = {
					chapters: importResult.chapters_created,
					scenes: importResult.scenes_created,
				};
				// Reload manuscript to show new content
				await appState.loadChapters();
				showSuccess(
					`Imported ${importResult.chapters_created} chapter(s) and ${importResult.scenes_created} scene(s)`
				);
			} else {
				if (!selectedChapterId) {
					error = 'Please select a chapter';
					return;
				}

				if (fileType === 'markdown') {
					await importApi.markdownAsScene(selectedChapterId, sceneTitle, content);
				} else {
					await importApi.textAsScene(selectedChapterId, sceneTitle, content);
				}
				result = { chapters: 0, scenes: 1 };
				// Reload manuscript to show new content
				await appState.loadChapters();
				showSuccess('Scene imported successfully');
			}
		} catch (e) {
			error = String(e);
			showError('Import failed: ' + String(e));
		} finally {
			isImporting = false;
		}
	}

	function handleClose() {
		isOpen = false;
		content = '';
		result = null;
		error = null;
		onclose?.();
	}

	function reset() {
		content = '';
		result = null;
		error = null;
		sceneTitle = 'Imported Scene';
		docxFile = null;
		importMode = 'structured';
	}

	function handleOverlayClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			handleClose();
		}
	}
</script>

{#if isOpen}
	<div class="dialog-overlay" onclick={handleOverlayClick} role="presentation">
		<div class="dialog-container">
			<div class="dialog-header">
				<h2>Import Content</h2>
				<Button variant="icon" onclick={handleClose} title="Close">
					<Icon name="close" size={20} />
				</Button>
			</div>

			{#if result}
				<div class="dialog-content">
					<div class="success-message">
						<Icon name="check" size={48} />
						<h3>Import Successful</h3>
						<p>
							{#if result.chapters === -1 && result.scenes === -1}
								Project data fully restored from backup.
							{:else if result.chapters > 0}
								Created {result.chapters} chapter{result.chapters !== 1 ? 's' : ''} and
								{result.scenes} scene{result.scenes !== 1 ? 's' : ''}.
							{:else}
								{result.scenes} scene{result.scenes !== 1 ? 's' : ''}.
							{/if}
						</p>
						<Button variant="primary" onclick={reset}>Import More</Button>
					</div>
				</div>
			{:else}
				<div class="dialog-content">
					<div class="import-modes">
						<button
							class="mode-btn"
							class:active={importMode === 'structured'}
							onclick={() => {
								importMode = 'structured';
								docxFile = null;
							}}
						>
							<svg
								width="20"
								height="20"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
								<polyline points="14 2 14 8 20 8" />
								<line x1="16" y1="13" x2="8" y2="13" />
								<line x1="16" y1="17" x2="8" y2="17" />
							</svg>
							<span>Structured Import</span>
							<small>Auto-detect chapters (# ) and scenes (## )</small>
						</button>
						<button
							class="mode-btn"
							class:active={importMode === 'single-scene'}
							onclick={() => {
								importMode = 'single-scene';
								docxFile = null;
							}}
						>
							<svg
								width="20"
								height="20"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<path d="M12 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
								<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
							</svg>
							<span>Single Scene</span>
							<small>Import as one scene in existing chapter</small>
						</button>
						<button
							class="mode-btn"
							class:active={importMode === 'docx'}
							onclick={() => (importMode = 'docx')}
						>
							<svg
								width="20"
								height="20"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
								<polyline points="14 2 14 8 20 8" />
								<path d="M9 15l2 2 4-4" />
							</svg>
							<span>DOCX Import</span>
							<small>Import Word document with structure</small>
						</button>
						<button
							class="mode-btn"
							class:active={importMode === 'json-backup'}
							onclick={() => {
								importMode = 'json-backup';
								docxFile = null;
							}}
						>
							<svg
								width="20"
								height="20"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
								<polyline points="7 10 12 15 17 10" />
								<line x1="12" y1="15" x2="12" y2="3" />
							</svg>
							<span>JSON Backup</span>
							<small>Restore full project from backup</small>
						</button>
					</div>

					<div class="file-input-section">
						<label class="file-input-label">
							<input
								type="file"
								accept={importMode === 'docx'
									? '.docx'
									: importMode === 'json-backup'
										? '.json'
										: '.md,.markdown,.txt,.docx,.json'}
								onchange={handleFileSelect}
							/>
							<Icon name="upload" size={20} />
							<span>Choose file</span>
							<small>{importMode === 'docx'
								? '.docx'
								: importMode === 'json-backup'
									? '.json'
									: '.md, .markdown, .txt, .docx, .json'}</small>
						</label>
					</div>

					{#if importMode === 'single-scene'}
						<div class="form-group">
							<label for="chapter">Target Chapter</label>
							<select id="chapter" bind:value={selectedChapterId}>
								{#each appState.chapters as chapter (chapter.id)}
									<option value={chapter.id}>{chapter.title}</option>
								{/each}
							</select>
						</div>

						<div class="form-group">
							<label for="scene-title">Scene Title</label>
							<input id="scene-title" type="text" bind:value={sceneTitle} />
						</div>

						<div class="form-group">
							<span id="file-type-label">File Type</span>
							<div class="radio-group" role="radiogroup" aria-labelledby="file-type-label">
								<label>
									<input type="radio" bind:group={fileType} value="markdown" />
									Markdown
								</label>
								<label>
									<input type="radio" bind:group={fileType} value="text" />
									Plain Text
								</label>
							</div>
						</div>
					{/if}

					<div class="form-group">
						<label for="content">Content Preview</label>
						<textarea
							id="content"
							bind:value={content}
							placeholder="Paste content here or select a file above..."
							rows="10"
						></textarea>
					</div>

					{#if error}
						<div class="error-message">{error}</div>
					{/if}
				</div>

				<div class="dialog-footer">
					<Button variant="ghost" onclick={handleClose}>Cancel</Button>
					<Button
						variant="primary"
						onclick={handleImport}
						disabled={isImporting || !content.trim()}
					>
						{#if isImporting}
							Importing...
						{:else}
							Import
						{/if}
					</Button>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.dialog-overlay {
		position: fixed;
		inset: 0;
		background-color: var(--overlay-backdrop);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		padding: var(--spacing-xl);
	}

	.dialog-container {
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		width: 100%;
		max-width: 600px;
		max-height: 90vh;
		display: flex;
		flex-direction: column;
	}

	.dialog-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border-light);
	}

	.dialog-header h2 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.dialog-content {
		padding: var(--spacing-lg);
		overflow-y: auto;
		flex: 1;
	}

	.import-modes {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--spacing-md);
		margin-bottom: var(--spacing-lg);
	}

	.mode-btn {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-md);
		border: 2px solid var(--color-border);
		border-radius: var(--border-radius-md);
		background-color: var(--color-bg-secondary);
		transition: all var(--transition-fast);
	}

	.mode-btn:hover {
		border-color: var(--color-accent);
	}

	.mode-btn.active {
		border-color: var(--color-accent);
		background-color: var(--color-accent-light);
	}

	.mode-btn span {
		font-weight: 500;
	}

	.mode-btn small {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-align: center;
	}

	.file-input-section {
		margin-bottom: var(--spacing-lg);
	}

	.file-input-label {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-lg);
		border: 2px dashed var(--color-border);
		border-radius: var(--border-radius-md);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.file-input-label:hover {
		border-color: var(--color-accent);
		background-color: var(--color-bg-hover);
	}

	.file-input-label input {
		display: none;
	}

	.file-input-label small {
		color: var(--color-text-muted);
		font-size: var(--font-size-xs);
	}

	.form-group {
		margin-bottom: var(--spacing-md);
	}

	.form-group label {
		display: block;
		font-size: var(--font-size-sm);
		font-weight: 500;
		margin-bottom: var(--spacing-xs);
	}

	.form-group input[type='text'],
	.form-group select,
	.form-group textarea {
		width: 100%;
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
	}

	.form-group textarea {
		font-family: monospace;
		resize: vertical;
	}

	.radio-group {
		display: flex;
		gap: var(--spacing-md);
	}

	.radio-group label {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-weight: normal;
	}

	.error-message {
		padding: var(--spacing-sm);
		background-color: var(--danger-subtle);
		color: var(--color-error);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
	}

	.success-message {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--spacing-md);
		padding: var(--spacing-xl);
		text-align: center;
	}

	.success-message :global(.icon) {
		color: var(--color-success);
	}

	.success-message h3 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.success-message p {
		color: var(--color-text-secondary);
	}

	.dialog-footer {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-sm);
		padding: var(--spacing-md) var(--spacing-lg);
		border-top: 1px solid var(--color-border-light);
	}
</style>
