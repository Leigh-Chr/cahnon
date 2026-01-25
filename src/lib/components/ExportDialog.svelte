<!--
  Export dialog for manuscript and data export.

  Supported formats:
  - Markdown: Full manuscript with headers
  - Plain text: Stripped formatting
  - JSON: Full backup including all metadata
  - DOCX: Word document with formatting options
  - HTML: Web-ready manuscript
  - PDF: Print-ready document
  - Outline: Chapter/scene structure only
  - Bible: Knowledge base export
  - Timeline: Events and chronology

  DOCX options include chapter headers, scene headers, and page breaks.
-->
<script lang="ts">
	import { exportApi } from '$lib/api';
	import { showSuccess, showError } from '$lib/toast';
	import { appState } from '$lib/stores';
	import { exportToDocx, downloadHtml, exportToPdf } from '$lib/export';

	interface Props {
		isOpen?: boolean;
		onclose?: () => void;
	}

	let { isOpen = false, onclose }: Props = $props();

	let exportFormat = $state<
		'markdown' | 'plaintext' | 'json' | 'docx' | 'html' | 'pdf' | 'outline' | 'bible' | 'timeline'
	>('markdown');
	let isExporting = $state(false);
	let exportResult = $state<string | null>(null);
	let error = $state<string | null>(null);

	// DOCX options
	let includeChapterHeaders = $state(true);
	let includeSceneHeaders = $state(true);
	let pageBreakBetweenChapters = $state(true);

	function close() {
		exportResult = null;
		error = null;
		onclose?.();
	}

	async function handleExport() {
		isExporting = true;
		error = null;
		try {
			switch (exportFormat) {
				case 'markdown':
					exportResult = await exportApi.markdown();
					break;
				case 'plaintext':
					exportResult = await exportApi.plainText();
					break;
				case 'json':
					exportResult = await exportApi.jsonBackup();
					break;
				case 'outline':
					exportResult = await exportApi.outline();
					break;
				case 'bible':
					exportResult = await exportApi.bible();
					break;
				case 'timeline':
					exportResult = await exportApi.timeline();
					break;
				case 'docx':
				case 'html':
				case 'pdf':
					await exportDocumentFormat();
					return; // Early return - no preview for DOCX/HTML/PDF
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Export failed';
			showError('Export failed');
		}
		isExporting = false;
	}

	async function exportDocumentFormat() {
		try {
			const proj = appState.project;
			const chaps = appState.chapters;
			const scns = appState.scenes;

			if (!proj) {
				throw new Error('No project open');
			}

			if (exportFormat === 'docx') {
				await exportToDocx(proj, chaps, scns, {
					includeChapterHeaders,
					includeSceneHeaders,
					pageBreakBetweenChapters,
				});
				showSuccess('DOCX exported successfully');
			} else if (exportFormat === 'html') {
				downloadHtml(proj, chaps, scns);
				showSuccess('HTML exported successfully');
			} else if (exportFormat === 'pdf') {
				await exportToPdf(proj, chaps, scns, {
					includeChapterHeaders,
					includeSceneHeaders,
					pageBreakBetweenChapters,
				});
				showSuccess('PDF exported successfully');
			}

			close();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Export failed';
			showError('Export failed');
		}
		isExporting = false;
	}

	function copyToClipboard() {
		if (exportResult) {
			navigator.clipboard.writeText(exportResult);
			showSuccess('Copied to clipboard');
		}
	}

	function downloadFile() {
		if (!exportResult) return;

		const extensions: Record<string, string> = {
			markdown: 'md',
			plaintext: 'txt',
			json: 'json',
			outline: 'md',
			bible: 'md',
			timeline: 'md',
		};

		const mimeTypes: Record<string, string> = {
			markdown: 'text/markdown',
			plaintext: 'text/plain',
			json: 'application/json',
			outline: 'text/markdown',
			bible: 'text/markdown',
			timeline: 'text/markdown',
		};

		const blob = new Blob([exportResult], { type: mimeTypes[exportFormat] });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `export.${extensions[exportFormat]}`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
		showSuccess('File downloaded');
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			close();
		}
	}

	function handleOverlayClick() {
		close();
	}

	function handleDialogClick(event: MouseEvent) {
		event.stopPropagation();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="dialog-overlay" onclick={handleOverlayClick} role="presentation">
		<div
			class="dialog"
			onclick={handleDialogClick}
			role="dialog"
			aria-modal="true"
			aria-labelledby="export-title"
			tabindex="-1"
		>
			<div class="dialog-header">
				<h2 id="export-title">Export Project</h2>
				<button class="close-btn" onclick={close} aria-label="Close">
					<svg
						width="20"
						height="20"
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

			<div class="dialog-content">
				{#if !exportResult}
					<div class="format-options">
						<label class="format-option" class:selected={exportFormat === 'markdown'}>
							<input type="radio" bind:group={exportFormat} value="markdown" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
									<polyline points="14 2 14 8 20 8" />
									<line x1="16" y1="13" x2="8" y2="13" />
									<line x1="16" y1="17" x2="8" y2="17" />
									<polyline points="10 9 9 9 8 9" />
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">Markdown</span>
								<span class="format-desc">Formatted with headers and structure</span>
							</div>
						</label>

						<label class="format-option" class:selected={exportFormat === 'plaintext'}>
							<input type="radio" bind:group={exportFormat} value="plaintext" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
									<polyline points="14 2 14 8 20 8" />
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">Plain Text</span>
								<span class="format-desc">Just the text, no formatting</span>
							</div>
						</label>

						<label class="format-option" class:selected={exportFormat === 'json'}>
							<input type="radio" bind:group={exportFormat} value="json" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
									<polyline points="14 2 14 8 20 8" />
									<path d="M8 13h2" />
									<path d="M8 17h2" />
									<path d="M14 13h2" />
									<path d="M14 17h2" />
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">JSON Backup</span>
								<span class="format-desc">Full project data for backup/restore</span>
							</div>
						</label>

						<label class="format-option" class:selected={exportFormat === 'docx'}>
							<input type="radio" bind:group={exportFormat} value="docx" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
									<polyline points="14 2 14 8 20 8" />
									<path d="M9 15l2 2 4-4" />
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">Word Document</span>
								<span class="format-desc">Microsoft Word .docx format</span>
							</div>
						</label>

						<label class="format-option" class:selected={exportFormat === 'html'}>
							<input type="radio" bind:group={exportFormat} value="html" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
									<polyline points="14 2 14 8 20 8" />
									<path d="M10 12l-2 2 2 2" />
									<path d="M14 12l2 2-2 2" />
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">HTML</span>
								<span class="format-desc">Web-ready HTML with styling</span>
							</div>
						</label>

						<label class="format-option" class:selected={exportFormat === 'pdf'}>
							<input type="radio" bind:group={exportFormat} value="pdf" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
									<polyline points="14 2 14 8 20 8" />
									<text x="8" y="18" font-size="7" fill="currentColor">PDF</text>
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">PDF</span>
								<span class="format-desc">Portable Document Format for printing</span>
							</div>
						</label>
					</div>

					<div class="section-divider">
						<span>Partial Exports</span>
					</div>

					<div class="format-options">
						<label class="format-option" class:selected={exportFormat === 'outline'}>
							<input type="radio" bind:group={exportFormat} value="outline" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<line x1="8" y1="6" x2="21" y2="6" />
									<line x1="8" y1="12" x2="21" y2="12" />
									<line x1="8" y1="18" x2="21" y2="18" />
									<line x1="3" y1="6" x2="3.01" y2="6" />
									<line x1="3" y1="12" x2="3.01" y2="12" />
									<line x1="3" y1="18" x2="3.01" y2="18" />
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">Outline Only</span>
								<span class="format-desc">Chapters and scene summaries</span>
							</div>
						</label>

						<label class="format-option" class:selected={exportFormat === 'bible'}>
							<input type="radio" bind:group={exportFormat} value="bible" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
									<path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
									<line x1="8" y1="7" x2="16" y2="7" />
									<line x1="8" y1="11" x2="14" y2="11" />
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">Bible Only</span>
								<span class="format-desc">Characters, locations, and world-building</span>
							</div>
						</label>

						<label class="format-option" class:selected={exportFormat === 'timeline'}>
							<input type="radio" bind:group={exportFormat} value="timeline" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<line x1="12" y1="2" x2="12" y2="22" />
									<circle cx="12" cy="6" r="2" />
									<circle cx="12" cy="12" r="2" />
									<circle cx="12" cy="18" r="2" />
									<line x1="14" y1="6" x2="20" y2="6" />
									<line x1="4" y1="12" x2="10" y2="12" />
									<line x1="14" y1="18" x2="20" y2="18" />
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">Timeline Only</span>
								<span class="format-desc">Events and chronological data</span>
							</div>
						</label>
					</div>

					{#if exportFormat === 'docx' || exportFormat === 'pdf'}
						<div class="export-options">
							<h4>Document Options</h4>
							<label class="checkbox-option">
								<input type="checkbox" bind:checked={includeChapterHeaders} />
								Include chapter headers
							</label>
							<label class="checkbox-option">
								<input type="checkbox" bind:checked={includeSceneHeaders} />
								Include scene headers
							</label>
							<label class="checkbox-option">
								<input type="checkbox" bind:checked={pageBreakBetweenChapters} />
								Page break between chapters
							</label>
						</div>
					{/if}

					{#if error}
						<div class="error-message">{error}</div>
					{/if}

					<div class="dialog-actions">
						<button class="cancel-btn" onclick={close}>Cancel</button>
						<button class="export-btn" onclick={handleExport} disabled={isExporting}>
							{#if isExporting}
								Exporting...
							{:else}
								Export
							{/if}
						</button>
					</div>
				{:else}
					<div class="export-result">
						<div class="result-header">
							<span class="success-icon">
								<svg
									width="20"
									height="20"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<polyline points="20 6 9 17 4 12" />
								</svg>
							</span>
							<span>Export complete!</span>
						</div>
						<div class="result-preview">
							<pre>{exportResult.slice(0, 500)}{exportResult.length > 500 ? '...' : ''}</pre>
						</div>
						<div class="result-info">
							{exportResult.length.toLocaleString()} characters
						</div>
					</div>

					<div class="dialog-actions">
						<button class="secondary-btn" onclick={copyToClipboard}>
							<svg
								width="16"
								height="16"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
								<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
							</svg>
							Copy
						</button>
						<button class="export-btn" onclick={downloadFile}>
							<svg
								width="16"
								height="16"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
								<polyline points="7 10 12 15 17 10" />
								<line x1="12" y1="15" x2="12" y2="3" />
							</svg>
							Download
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.dialog-overlay {
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

	.dialog {
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		width: 90%;
		max-width: 500px;
		max-height: 80vh;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.dialog-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
	}

	.dialog-header h2 {
		font-size: var(--font-size-lg);
		font-weight: 600;
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

	.dialog-content {
		padding: var(--spacing-lg);
		overflow-y: auto;
	}

	.format-options {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.format-option {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border: 2px solid transparent;
		border-radius: var(--border-radius-md);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.format-option:hover {
		background-color: var(--color-bg-hover);
	}

	.format-option.selected {
		border-color: var(--color-accent);
		background-color: var(--color-accent-light);
	}

	.format-option input {
		display: none;
	}

	.format-icon {
		color: var(--color-text-muted);
	}

	.format-option.selected .format-icon {
		color: var(--color-accent);
	}

	.format-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.format-name {
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.format-desc {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.error-message {
		margin-top: var(--spacing-md);
		padding: var(--spacing-sm) var(--spacing-md);
		background-color: var(--danger-subtle);
		border: 1px solid var(--danger-border);
		border-radius: var(--border-radius-sm);
		color: var(--color-error);
		font-size: var(--font-size-sm);
	}

	.dialog-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-sm);
		margin-top: var(--spacing-lg);
	}

	.cancel-btn {
		padding: var(--spacing-sm) var(--spacing-lg);
		color: var(--color-text-secondary);
		border-radius: var(--border-radius-md);
	}

	.cancel-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.secondary-btn {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-sm) var(--spacing-lg);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		color: var(--color-text-primary);
		border-radius: var(--border-radius-md);
		font-weight: 500;
	}

	.secondary-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.export-btn {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-sm) var(--spacing-lg);
		background-color: var(--color-accent);
		color: var(--text-on-accent);
		border-radius: var(--border-radius-md);
		font-weight: 500;
	}

	.export-btn:hover:not(:disabled) {
		background-color: var(--color-accent-hover);
	}

	.export-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.export-result {
		margin-bottom: var(--spacing-md);
	}

	.result-header {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		color: var(--color-success);
		font-weight: 500;
		margin-bottom: var(--spacing-md);
	}

	.success-icon {
		display: flex;
	}

	.result-preview {
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		padding: var(--spacing-md);
		max-height: 200px;
		overflow-y: auto;
	}

	.result-preview pre {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		white-space: pre-wrap;
		word-break: break-word;
		margin: 0;
	}

	.result-info {
		margin-top: var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		text-align: right;
	}

	.export-options {
		margin-top: var(--spacing-lg);
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-md);
	}

	.export-options h4 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		margin-bottom: var(--spacing-sm);
		color: var(--color-text-secondary);
	}

	.checkbox-option {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-xs) 0;
		font-size: var(--font-size-sm);
		cursor: pointer;
	}

	.checkbox-option input[type='checkbox'] {
		width: 16px;
		height: 16px;
		cursor: pointer;
	}

	.section-divider {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
		margin: var(--spacing-lg) 0;
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
		font-weight: 500;
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.section-divider::before,
	.section-divider::after {
		content: '';
		flex: 1;
		height: 1px;
		background-color: var(--color-border);
	}
</style>
