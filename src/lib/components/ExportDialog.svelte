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
	import { SvelteSet } from 'svelte/reactivity';

	import { exportApi } from '$lib/api';
	import { downloadHtml, exportToDocx, exportToPdf } from '$lib/export';
	import { appState } from '$lib/stores';
	import { showError, showSuccess } from '$lib/toast';
	import { trapFocus } from '$lib/utils/focus-trap';

	import { Button, Icon, ProgressBar } from './ui';

	interface Props {
		isOpen?: boolean;
		onclose?: () => void;
	}

	let { isOpen = false, onclose }: Props = $props();

	let exportFormat = $state<
		| 'markdown'
		| 'plaintext'
		| 'json'
		| 'docx'
		| 'html'
		| 'pdf'
		| 'outline'
		| 'bible'
		| 'timeline'
		| 'csv_bible'
		| 'csv_timeline'
		| 'csv_review'
		| 'csv_stats'
	>('markdown');
	let isExporting = $state(false);
	let exportResult = $state<string | null>(null);
	let error = $state<string | null>(null);

	// DOCX options
	let includeChapterHeaders = $state(true);
	let includeSceneHeaders = $state(true);
	let pageBreakBetweenChapters = $state(true);

	// Scope & separator options (markdown/plaintext)
	let exportScope = $state<'all' | 'selected'>('all');
	let selectedChapterIds = new SvelteSet<string>();
	let sceneSeparator = $state('');
	let includeSceneTitles = $state(true);

	const defaultSeparators: Record<string, string> = {
		markdown: '###',
		plaintext: '* * *',
	};

	// CE1: Export presets
	const PRESETS_KEY = 'cahnon-export-presets';

	interface ExportPreset {
		name: string;
		format: typeof exportFormat;
		options: {
			includeChapterHeaders: boolean;
			includeSceneHeaders: boolean;
			pageBreakBetweenChapters: boolean;
			includeSceneTitles: boolean;
			sceneSeparator: string;
		};
	}

	function loadPresets(): ExportPreset[] {
		try {
			return JSON.parse(localStorage.getItem(PRESETS_KEY) || '[]');
		} catch {
			return [];
		}
	}

	let presets = $state<ExportPreset[]>(loadPresets());
	let newPresetName = $state('');

	function savePreset() {
		if (!newPresetName.trim()) return;

		const preset: ExportPreset = {
			name: newPresetName.trim(),
			format: exportFormat,
			options: {
				includeChapterHeaders,
				includeSceneHeaders,
				pageBreakBetweenChapters,
				includeSceneTitles,
				sceneSeparator,
			},
		};

		// Replace if exists, otherwise add
		const idx = presets.findIndex((p) => p.name === preset.name);
		if (idx >= 0) {
			presets[idx] = preset;
		} else {
			presets = [...presets, preset];
		}

		try {
			localStorage.setItem(PRESETS_KEY, JSON.stringify(presets));
		} catch {
			// localStorage unavailable
		}

		newPresetName = '';
		showSuccess(`Preset "${preset.name}" saved`);
	}

	function applyPreset(preset: ExportPreset) {
		exportFormat = preset.format;
		includeChapterHeaders = preset.options.includeChapterHeaders;
		includeSceneHeaders = preset.options.includeSceneHeaders;
		pageBreakBetweenChapters = preset.options.pageBreakBetweenChapters;
		includeSceneTitles = preset.options.includeSceneTitles;
		sceneSeparator = preset.options.sceneSeparator;
	}

	function deletePreset(name: string) {
		presets = presets.filter((p) => p.name !== name);
		try {
			localStorage.setItem(PRESETS_KEY, JSON.stringify(presets));
		} catch {
			// localStorage unavailable
		}
	}

	// CE2: Live preview (simplified)
	let previewContent = $derived.by(() => {
		if (!isOpen) return '';

		// Only generate preview for text formats
		if (!['markdown', 'plaintext'].includes(exportFormat)) return '';

		// Generate a simple preview of the first scene
		const firstChapter = appState.chapters[0];
		if (!firstChapter) return 'No content to preview.';

		const scenes = appState.scenes.get(firstChapter.id) || [];
		const firstScene = scenes[0];
		if (!firstScene) return 'No scenes to preview.';

		const separator = sceneSeparator || defaultSeparators[exportFormat] || '';

		let preview = '';
		if (includeChapterHeaders) {
			preview += `# ${firstChapter.title}\n\n`;
		}
		if (includeSceneTitles) {
			preview += `## ${firstScene.title}\n\n`;
		}
		preview += (firstScene.text || '').slice(0, 200);
		if ((firstScene.text?.length || 0) > 200) preview += '...';
		preview += `\n\n${separator}\n\n`;
		preview += '(Preview of first scene)';

		return preview;
	});

	function toggleChapter(id: string) {
		if (selectedChapterIds.has(id)) {
			selectedChapterIds.delete(id);
		} else {
			selectedChapterIds.add(id);
		}
	}

	function selectAllChapters() {
		for (const c of appState.chapters) {
			selectedChapterIds.add(c.id);
		}
	}

	function deselectAllChapters() {
		selectedChapterIds.clear();
	}

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
				case 'markdown': {
					const chapterIds =
						exportScope === 'selected' && selectedChapterIds.size > 0
							? [...selectedChapterIds]
							: undefined;
					const sep = sceneSeparator.trim() || undefined;
					exportResult = await exportApi.markdown({
						chapterIds,
						sceneSeparator: sep,
						includeTitles: includeSceneTitles,
					});
					break;
				}
				case 'plaintext': {
					const chapterIds =
						exportScope === 'selected' && selectedChapterIds.size > 0
							? [...selectedChapterIds]
							: undefined;
					const sep = sceneSeparator.trim() || undefined;
					exportResult = await exportApi.plainText({ chapterIds, sceneSeparator: sep });
					break;
				}
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
				case 'csv_bible':
					exportResult = await exportApi.exportBibleCsv();
					break;
				case 'csv_timeline':
					exportResult = await exportApi.exportTimelineCsv();
					break;
				case 'csv_review':
					exportResult = await exportApi.exportReviewGridCsv();
					break;
				case 'csv_stats':
					exportResult = await exportApi.exportStatsCsv();
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
			navigator.clipboard.writeText(exportResult).then(
				() => showSuccess('Copied to clipboard'),
				() => showError('Failed to copy to clipboard')
			);
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
			csv_bible: 'csv',
			csv_timeline: 'csv',
			csv_review: 'csv',
			csv_stats: 'csv',
		};

		const mimeTypes: Record<string, string> = {
			markdown: 'text/markdown',
			plaintext: 'text/plain',
			json: 'application/json',
			outline: 'text/markdown',
			bible: 'text/markdown',
			timeline: 'text/markdown',
			csv_bible: 'text/csv',
			csv_timeline: 'text/csv',
			csv_review: 'text/csv',
			csv_stats: 'text/csv',
		};

		const blob = new Blob([exportResult], { type: mimeTypes[exportFormat] });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		// Use project title for filename, sanitized for filesystem
		const projectName = appState.project?.title || 'export';
		const safeFilename = projectName.replace(/[<>:"/\\|?*]/g, '_').trim();
		const suffixes: Record<string, string> = {
			csv_bible: '-bible',
			csv_timeline: '-timeline',
			csv_review: '-review-grid',
			csv_stats: '-stats',
		};
		const suffix = suffixes[exportFormat] || '';
		a.download = `${safeFilename}${suffix}.${extensions[exportFormat]}`;
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
	<div
		class="dialog-overlay"
		onclick={handleOverlayClick}
		onkeydown={(e) => {
			if (e.key === 'Escape') close();
		}}
		role="presentation"
		tabindex="-1"
	>
		<!-- AE1: Focus trap -->
		<div
			class="dialog modal-enter"
			onclick={handleDialogClick}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="export-title"
			tabindex="-1"
			use:trapFocus={{ onEscape: close }}
		>
			<div class="dialog-header">
				<h2 id="export-title">Export Project</h2>
				<Button variant="icon" onclick={close} title="Close">
					<Icon name="close" size={20} />
				</Button>
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
								<span class="format-desc">For sharing text. Most universal format.</span>
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
								<span class="format-desc">Raw text without any markup or styling.</span>
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
								<span class="format-desc">For editors and publishers. Preserves formatting.</span>
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
								<span class="format-desc">For web publishing. Ready to use online.</span>
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
								<span class="format-desc">For printing or final distribution.</span>
							</div>
						</label>
					</div>

					<div class="section-divider">
						<span>Partial Export</span>
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

					<div class="section-divider">
						<span>Data &amp; Analysis</span>
					</div>

					<div class="format-options">
						<label class="format-option" class:selected={exportFormat === 'csv_bible'}>
							<input type="radio" bind:group={exportFormat} value="csv_bible" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<rect x="3" y="3" width="18" height="18" rx="2" />
									<line x1="3" y1="9" x2="21" y2="9" />
									<line x1="3" y1="15" x2="21" y2="15" />
									<line x1="9" y1="3" x2="9" y2="21" />
									<line x1="15" y1="3" x2="15" y2="21" />
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">Bible CSV</span>
								<span class="format-desc">For spreadsheet analysis of characters and world.</span>
							</div>
						</label>

						<label class="format-option" class:selected={exportFormat === 'csv_timeline'}>
							<input type="radio" bind:group={exportFormat} value="csv_timeline" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<rect x="3" y="3" width="18" height="18" rx="2" />
									<line x1="3" y1="9" x2="21" y2="9" />
									<line x1="3" y1="15" x2="21" y2="15" />
									<line x1="9" y1="3" x2="9" y2="21" />
									<line x1="15" y1="3" x2="15" y2="21" />
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">Timeline CSV</span>
								<span class="format-desc">For spreadsheet analysis of chronology.</span>
							</div>
						</label>

						<label class="format-option" class:selected={exportFormat === 'csv_review'}>
							<input type="radio" bind:group={exportFormat} value="csv_review" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<rect x="3" y="3" width="18" height="18" rx="2" />
									<line x1="3" y1="9" x2="21" y2="9" />
									<line x1="3" y1="15" x2="21" y2="15" />
									<line x1="9" y1="3" x2="9" y2="21" />
									<line x1="15" y1="3" x2="15" y2="21" />
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">Review Grid CSV</span>
								<span class="format-desc">Scene status, POV, tension data</span>
							</div>
						</label>

						<label class="format-option" class:selected={exportFormat === 'csv_stats'}>
							<input type="radio" bind:group={exportFormat} value="csv_stats" />
							<div class="format-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<rect x="3" y="3" width="18" height="18" rx="2" />
									<line x1="3" y1="9" x2="21" y2="9" />
									<line x1="3" y1="15" x2="21" y2="15" />
									<line x1="9" y1="3" x2="9" y2="21" />
									<line x1="15" y1="3" x2="15" y2="21" />
								</svg>
							</div>
							<div class="format-info">
								<span class="format-name">Statistics CSV</span>
								<span class="format-desc">Word counts by chapter and status</span>
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

					{#if exportFormat === 'markdown' || exportFormat === 'plaintext'}
						<div class="export-options">
							<h4>Export Options</h4>

							<div class="scope-option">
								<label class="radio-option">
									<input type="radio" bind:group={exportScope} value="all" />
									All chapters
								</label>
								<label class="radio-option">
									<input type="radio" bind:group={exportScope} value="selected" />
									Selected chapters
								</label>
							</div>

							{#if exportScope === 'selected'}
								<div class="chapter-select">
									<div class="chapter-select-actions">
										<button type="button" class="link-btn" onclick={selectAllChapters}
											>Select all</button
										>
										<button type="button" class="link-btn" onclick={deselectAllChapters}
											>Clear</button
										>
									</div>
									{#each appState.chapters as chapter (chapter.id)}
										<label class="checkbox-option">
											<input
												type="checkbox"
												checked={selectedChapterIds.has(chapter.id)}
												onchange={() => toggleChapter(chapter.id)}
											/>
											{chapter.title}
										</label>
									{/each}
								</div>
							{/if}

							<div class="text-option">
								<label for="scene-separator">Scene separator</label>
								<input
									id="scene-separator"
									type="text"
									bind:value={sceneSeparator}
									placeholder={defaultSeparators[exportFormat] ?? '###'}
								/>
								<!-- AZ5: Preview separator in context -->
								<div class="separator-preview">
									<span class="preview-text">...end of scene.</span>
									<span class="preview-separator"
										>{sceneSeparator || defaultSeparators[exportFormat] || '###'}</span
									>
									<span class="preview-text">Next scene begins...</span>
								</div>
							</div>

							{#if exportFormat === 'markdown'}
								<label class="checkbox-option">
									<input type="checkbox" bind:checked={includeSceneTitles} />
									Include scene titles
								</label>
							{/if}
						</div>
					{/if}

					{#if exportFormat === 'json'}
						<div class="json-summary">
							<span class="summary-chip">{appState.chapters.length} chapters</span>
							<span class="summary-chip"
								>{Array.from(appState.scenes.values()).flat().length} scenes</span
							>
							<span class="summary-chip">{appState.bibleEntries.length} bible entries</span>
						</div>
					{/if}

					<!-- CE1: Export presets -->
					<div class="presets-section">
						<h4>Presets</h4>
						{#if presets.length > 0}
							<div class="presets-list">
								{#each presets as preset (preset.name)}
									<div class="preset-chip">
										<button class="preset-apply" onclick={() => applyPreset(preset)}>
											{preset.name}
										</button>
										<button
											class="preset-delete"
											onclick={() => deletePreset(preset.name)}
											title="Delete"
										>
											<Icon name="close" size={12} />
										</button>
									</div>
								{/each}
							</div>
						{/if}
						<div class="preset-save">
							<input
								type="text"
								bind:value={newPresetName}
								placeholder="Preset name..."
								maxlength="30"
							/>
							<Button size="sm" onclick={savePreset} disabled={!newPresetName.trim()}>Save</Button>
						</div>
					</div>

					<!-- CE2: Live preview -->
					{#if previewContent && (exportFormat === 'markdown' || exportFormat === 'plaintext')}
						<div class="preview-section">
							<h4>Preview</h4>
							<pre class="preview-content">{previewContent}</pre>
						</div>
					{/if}

					{#if error}
						<div class="error-message">{error}</div>
					{/if}

					<!-- AA4, AM2: Progress indicator for export -->
					{#if isExporting}
						<div class="export-progress-message">
							<ProgressBar indeterminate label="Exporting..." showValue={false} />
							<p class="progress-hint">This may take a moment for large projects.</p>
						</div>
					{/if}

					<div class="dialog-actions">
						<Button variant="ghost" onclick={close}>Cancel</Button>
						<Button variant="primary" onclick={handleExport} disabled={isExporting}>
							{#if isExporting}
								Exporting...
							{:else}
								Export
							{/if}
						</Button>
					</div>
				{:else}
					<div class="export-result">
						<div class="result-header">
							<span class="success-icon">
								<Icon name="check" size={20} />
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
						<Button variant="secondary" onclick={copyToClipboard}>
							<Icon name="copy" size={16} />
							Copy
						</Button>
						<Button variant="primary" onclick={downloadFile}>
							<Icon name="download" size={16} />
							Download
						</Button>
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

	.json-summary {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-xs);
		padding: var(--spacing-sm) 0;
	}

	.summary-chip {
		font-size: var(--font-size-xs);
		padding: 2px var(--spacing-sm);
		background: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: 999px;
		color: var(--color-text-secondary);
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

	/* CE1: Presets section */
	.presets-section {
		margin-top: var(--spacing-lg);
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-md);
	}

	.presets-section h4 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		margin-bottom: var(--spacing-sm);
		color: var(--color-text-secondary);
	}

	.presets-list {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-xs);
		margin-bottom: var(--spacing-sm);
	}

	.preset-chip {
		display: flex;
		align-items: center;
		background-color: var(--color-bg-tertiary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		overflow: hidden;
	}

	.preset-apply {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-xs);
		cursor: pointer;
	}

	.preset-apply:hover {
		background-color: var(--color-bg-hover);
	}

	.preset-delete {
		display: flex;
		align-items: center;
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-left: 1px solid var(--color-border);
		cursor: pointer;
	}

	.preset-delete:hover {
		color: var(--color-error);
		background-color: var(--color-error-light, rgba(239, 68, 68, 0.1));
	}

	.preset-save {
		display: flex;
		gap: var(--spacing-sm);
	}

	.preset-save input {
		flex: 1;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
	}

	/* CE2: Preview section */
	.preview-section {
		margin-top: var(--spacing-lg);
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-md);
	}

	.preview-section h4 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		margin-bottom: var(--spacing-sm);
		color: var(--color-text-secondary);
	}

	.preview-content {
		max-height: 150px;
		overflow-y: auto;
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-family: var(--font-family-mono);
		font-size: var(--font-size-xs);
		white-space: pre-wrap;
		color: var(--color-text-secondary);
	}

	.checkbox-option {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-xs) 0;
		font-size: var(--font-size-sm);
	}

	.checkbox-option input[type='checkbox'] {
		width: 16px;
		height: 16px;
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

	.scope-option {
		display: flex;
		gap: var(--spacing-md);
		margin-bottom: var(--spacing-sm);
	}

	.radio-option {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-sm);
	}

	.chapter-select {
		margin-bottom: var(--spacing-sm);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		max-height: 150px;
		overflow-y: auto;
	}

	.chapter-select-actions {
		display: flex;
		gap: var(--spacing-sm);
		margin-bottom: var(--spacing-xs);
	}

	.link-btn {
		background: none;
		border: none;
		color: var(--color-accent);
		font-size: var(--font-size-xs);
		padding: 0;
		text-decoration: underline;
	}

	.link-btn:hover {
		color: var(--color-accent-hover, var(--color-accent));
	}

	.text-option {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--spacing-sm);
		margin: var(--spacing-sm) 0;
	}

	.text-option label {
		font-size: var(--font-size-sm);
		white-space: nowrap;
	}

	.text-option input[type='text'] {
		flex: 1;
		min-width: 120px;
		padding: var(--spacing-xs) var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
		color: var(--color-text-primary);
		font-size: var(--font-size-sm);
		font-family: var(--font-mono);
	}

	/* AZ5: Separator preview */
	.separator-preview {
		width: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-sm);
		margin-top: var(--spacing-xs);
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-xs);
	}

	.preview-text {
		color: var(--color-text-muted);
		font-style: italic;
	}

	.preview-separator {
		font-family: var(--font-mono);
		color: var(--color-text-secondary);
		font-weight: 500;
	}

	/* AA4, AM2: Export progress with ProgressBar */
	.export-progress-message {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-md);
		margin-bottom: var(--spacing-md);
	}

	.progress-hint {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		margin: 0;
		text-align: center;
	}

	/* Legacy spinner - removed in favor of ProgressBar component */
</style>
