<script lang="ts">
	import { appState } from '$lib/stores';
	import { projectApi } from '$lib/api';

	interface Props {
		isOpen?: boolean;
		onclose?: () => void;
	}

	let { isOpen = $bindable(false), onclose }: Props = $props();

	let projectWordTarget = $state<number | null>(null);
	let projectDailyTarget = $state<number | null>(null);

	$effect(() => {
		if (isOpen && appState.project) {
			projectWordTarget = appState.project.word_target;
			projectDailyTarget = appState.project.daily_word_target;
		}
	});

	async function updateProjectTarget(
		field: 'word_target' | 'daily_word_target',
		value: number | null
	) {
		try {
			const updated = await projectApi.update({ [field]: value });
			appState.project = updated;
		} catch (e) {
			console.error('Failed to update project:', e);
		}
	}

	const fontFamilies = [
		{ value: 'Georgia, serif', label: 'Georgia' },
		{ value: '"Times New Roman", Times, serif', label: 'Times New Roman' },
		{ value: '"Palatino Linotype", Palatino, serif', label: 'Palatino' },
		{ value: '"Libre Baskerville", serif', label: 'Libre Baskerville' },
		{ value: '"Merriweather", serif', label: 'Merriweather' },
		{
			value: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
			label: 'System Sans',
		},
		{ value: '"Source Sans Pro", sans-serif', label: 'Source Sans Pro' },
		{ value: '"Open Sans", sans-serif', label: 'Open Sans' },
		{ value: '"SF Mono", "Fira Code", monospace', label: 'Monospace' },
	];

	let settings = $derived({ ...appState.editorSettings });

	function handleFontFamilyChange(event: Event) {
		const value = (event.target as HTMLSelectElement).value;
		appState.editorSettings = { ...settings, fontFamily: value };
	}

	function handleFontSizeChange(event: Event) {
		const value = parseInt((event.target as HTMLInputElement).value);
		appState.editorSettings = { ...settings, fontSize: value };
	}

	function handleLineHeightChange(event: Event) {
		const value = parseFloat((event.target as HTMLInputElement).value);
		appState.editorSettings = { ...settings, lineHeight: value };
	}

	function handleTextWidthChange(event: Event) {
		const value = parseInt((event.target as HTMLInputElement).value);
		appState.editorSettings = { ...settings, textWidth: value };
	}

	function resetToDefaults() {
		appState.editorSettings = {
			fontFamily: 'Georgia, serif',
			fontSize: 18,
			lineHeight: 1.8,
			textWidth: 700,
		};
	}

	function handleClose() {
		isOpen = false;
		onclose?.();
	}

	function handleOverlayClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			handleClose();
		}
	}

	function handleColorModeChange(event: Event) {
		const value = (event.target as HTMLSelectElement).value as 'light' | 'dark' | 'system';
		appState.setColorMode(value);
	}

	function handleThemePaletteChange(event: Event) {
		const value = (event.target as HTMLSelectElement).value as 'cool' | 'warm';
		appState.setThemePalette(value);
	}

	function handleWordTargetBlur() {
		updateProjectTarget('word_target', projectWordTarget);
	}

	function handleDailyTargetBlur() {
		updateProjectTarget('daily_word_target', projectDailyTarget);
	}
</script>

{#if isOpen}
	<div class="dialog-overlay" onclick={handleOverlayClick} role="presentation">
		<div class="dialog-container">
			<div class="dialog-header">
				<h2>Settings</h2>
				<button class="close-btn" onclick={handleClose} aria-label="Close">
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
				<section class="settings-section">
					<h3>Editor Font</h3>

					<div class="form-group">
						<label for="font-family">Font Family</label>
						<select id="font-family" value={settings.fontFamily} onchange={handleFontFamilyChange}>
							{#each fontFamilies as font (font.value)}
								<option value={font.value} style="font-family: {font.value}">{font.label}</option>
							{/each}
						</select>
					</div>

					<div class="form-group">
						<label for="font-size">Font Size: {settings.fontSize}px</label>
						<input
							id="font-size"
							type="range"
							min="12"
							max="28"
							step="1"
							value={settings.fontSize}
							oninput={handleFontSizeChange}
						/>
					</div>

					<div class="form-group">
						<label for="line-height">Line Height: {settings.lineHeight.toFixed(1)}</label>
						<input
							id="line-height"
							type="range"
							min="1.2"
							max="2.4"
							step="0.1"
							value={settings.lineHeight}
							oninput={handleLineHeightChange}
						/>
					</div>

					<div class="form-group">
						<label for="text-width">Text Width: {settings.textWidth}px</label>
						<input
							id="text-width"
							type="range"
							min="500"
							max="1000"
							step="50"
							value={settings.textWidth}
							oninput={handleTextWidthChange}
						/>
					</div>

					<div class="preview">
						<p
							style="font-family: {settings.fontFamily}; font-size: {settings.fontSize}px; line-height: {settings.lineHeight}"
						>
							The quick brown fox jumps over the lazy dog. This is a preview of how your text will
							appear in the editor.
						</p>
					</div>
				</section>

				<section class="settings-section">
					<h3>Appearance</h3>

					<div class="form-group">
						<label for="color-mode">Color Mode</label>
						<select id="color-mode" value={appState.colorMode} onchange={handleColorModeChange}>
							<option value="system">System (Auto)</option>
							<option value="light">Light</option>
							<option value="dark">Dark</option>
						</select>
						<span class="hint">Choose light, dark, or follow your system preference</span>
					</div>

					<div class="form-group">
						<label for="theme-palette">Theme Palette</label>
						<select
							id="theme-palette"
							value={appState.themePalette}
							onchange={handleThemePaletteChange}
						>
							<option value="cool">Encre (Cool Blue)</option>
							<option value="warm">Ambre (Warm Terracotta)</option>
						</select>
						<span class="hint">Cool blue tones or warm earthy tones</span>
					</div>
				</section>

				{#if appState.project}
					<section class="settings-section">
						<h3>Project Goals</h3>

						<div class="form-group">
							<label for="manuscript-target">Manuscript Word Target</label>
							<input
								id="manuscript-target"
								type="number"
								placeholder="e.g., 80000"
								bind:value={projectWordTarget}
								onblur={handleWordTargetBlur}
							/>
							<span class="hint">Total word count goal for your manuscript</span>
						</div>

						<div class="form-group">
							<label for="daily-target">Daily Word Target</label>
							<input
								id="daily-target"
								type="number"
								placeholder="e.g., 1000"
								bind:value={projectDailyTarget}
								onblur={handleDailyTargetBlur}
							/>
							<span class="hint">Daily word count goal (progress shown in status bar)</span>
						</div>
					</section>
				{/if}
			</div>

			<div class="dialog-footer">
				<button class="reset-btn" onclick={resetToDefaults}>Reset to Defaults</button>
				<button class="done-btn" onclick={handleClose}>Done</button>
			</div>
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
		max-width: 500px;
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

	.close-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
		transition: all var(--transition-fast);
	}

	.close-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.dialog-content {
		padding: var(--spacing-lg);
		overflow-y: auto;
		flex: 1;
	}

	.settings-section {
		margin-bottom: var(--spacing-lg);
	}

	.settings-section:last-child {
		margin-bottom: 0;
	}

	.settings-section h3 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-muted);
		margin-bottom: var(--spacing-md);
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

	.form-group select {
		width: 100%;
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		background-color: var(--color-bg-primary);
	}

	.form-group input[type='range'] {
		width: 100%;
		accent-color: var(--color-accent);
	}

	.form-group input[type='number'] {
		width: 100%;
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		background-color: var(--color-bg-primary);
	}

	.hint {
		display: block;
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		margin-top: var(--spacing-xs);
	}

	.preview {
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-sm);
		margin-top: var(--spacing-md);
	}

	.preview p {
		margin: 0;
		color: var(--color-text-primary);
	}

	.dialog-footer {
		display: flex;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-top: 1px solid var(--color-border-light);
	}

	.reset-btn {
		padding: var(--spacing-sm) var(--spacing-md);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.reset-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.done-btn {
		padding: var(--spacing-sm) var(--spacing-lg);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		background-color: var(--color-accent);
		color: var(--text-on-accent);
	}

	.done-btn:hover {
		background-color: var(--color-accent-hover);
	}
</style>
