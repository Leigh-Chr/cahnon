<script lang="ts">
	import { projectApi } from '$lib/api';
	import { appState } from '$lib/stores';
	import { resetOnboarding } from '$lib/stores/onboarding';
	import type { KeyboardShortcuts, ShortcutBinding } from '$lib/stores/types';
	import { shortcutLabels } from '$lib/stores/types';
	import { showError, showInfo } from '$lib/toast';
	import { isModKey } from '$lib/utils';
	import { trapFocus } from '$lib/utils/focus-trap';

	import { Button, Icon } from './ui';

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
			showError('Failed to update project settings');
		}
	}

	// Phase 2.7: Updated font options with new web fonts
	const fontFamilies = [
		{ value: '"Source Serif 4", Charter, Georgia, serif', label: 'Source Serif 4 (Recommended)' },
		{ value: 'Georgia, serif', label: 'Georgia' },
		{ value: '"Times New Roman", Times, serif', label: 'Times New Roman' },
		{ value: '"Palatino Linotype", Palatino, serif', label: 'Palatino' },
		{ value: '"Libre Baskerville", serif', label: 'Libre Baskerville' },
		{ value: '"Merriweather", serif', label: 'Merriweather' },
		{
			value: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
			label: 'Inter (Sans)',
		},
		{
			value: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
			label: 'System Sans',
		},
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

	// CA4: Editor theme options
	import type { EditorTheme } from '$lib/stores/types';

	const editorThemes: { value: EditorTheme; label: string }[] = [
		{ value: 'default', label: 'Default' },
		{ value: 'sepia', label: 'Sepia' },
		{ value: 'dark', label: 'Dark' },
		{ value: 'low-contrast', label: 'Low Contrast' },
	];

	function handleThemeChange(event: Event) {
		const value = (event.target as HTMLSelectElement).value as EditorTheme;
		appState.editorSettings = { ...settings, theme: value };
	}

	function resetToDefaults() {
		appState.editorSettings = {
			// Phase 2.7: Default to new Source Serif 4 font
			fontFamily: '"Source Serif 4", Charter, Georgia, serif',
			fontSize: 18,
			lineHeight: 1.8,
			textWidth: 700,
			theme: 'default',
		};
		appState.resetShortcuts();
	}

	// Keyboard shortcut recording
	let recordingAction = $state<keyof KeyboardShortcuts | null>(null);

	// V2: Conflict detection
	let shortcutConflict = $state<{ action: keyof KeyboardShortcuts; conflictsWith: string } | null>(
		null
	);

	function startRecording(action: keyof KeyboardShortcuts) {
		recordingAction = action;
		shortcutConflict = null;
	}

	function bindingsMatch(a: ShortcutBinding, b: ShortcutBinding): boolean {
		return a.key === b.key && a.mod === b.mod && a.shift === b.shift;
	}

	function checkForConflict(
		action: keyof KeyboardShortcuts,
		binding: ShortcutBinding
	): keyof KeyboardShortcuts | null {
		const shortcuts = appState.keyboardShortcuts;
		for (const [key, existingBinding] of Object.entries(shortcuts)) {
			if (key !== action && bindingsMatch(existingBinding, binding)) {
				return key as keyof KeyboardShortcuts;
			}
		}
		return null;
	}

	function handleShortcutKeydown(event: KeyboardEvent) {
		if (!recordingAction) return;

		event.preventDefault();
		event.stopPropagation();

		// Ignore modifier-only presses
		if (['Control', 'Meta', 'Shift', 'Alt'].includes(event.key)) return;

		// Escape cancels recording
		if (event.key === 'Escape') {
			recordingAction = null;
			return;
		}

		const binding: ShortcutBinding = {
			key: event.key,
			mod: isModKey(event),
			shift: event.shiftKey,
		};

		// V2: Check for conflicts before setting
		const conflictAction = checkForConflict(recordingAction, binding);
		if (conflictAction) {
			shortcutConflict = {
				action: recordingAction,
				conflictsWith: shortcutLabels[conflictAction],
			};
		} else {
			shortcutConflict = null;
		}

		appState.setShortcut(recordingAction, binding);
		recordingAction = null;
	}

	function formatBinding(binding: ShortcutBinding): string {
		const isMac = typeof navigator !== 'undefined' && navigator.platform.includes('Mac');
		const parts: string[] = [];
		if (binding.mod) parts.push(isMac ? '⌘' : 'Ctrl');
		if (binding.shift) parts.push(isMac ? '⇧' : 'Shift');

		// Format the key nicely
		let keyLabel = binding.key;
		if (keyLabel === 'ArrowDown') keyLabel = '↓';
		else if (keyLabel === 'ArrowUp') keyLabel = '↑';
		else if (keyLabel === 'ArrowLeft') keyLabel = '←';
		else if (keyLabel === 'ArrowRight') keyLabel = '→';
		else if (keyLabel === '\\') keyLabel = '\\';
		else if (keyLabel.length === 1) keyLabel = keyLabel.toUpperCase();

		parts.push(keyLabel);
		return parts.join(isMac ? '' : '+');
	}

	const shortcutActions = Object.keys(shortcutLabels) as (keyof KeyboardShortcuts)[];

	function handleClose() {
		isOpen = false;
		recordingAction = null;
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
		if (projectWordTarget != null && projectWordTarget < 0) projectWordTarget = 0;
		updateProjectTarget('word_target', projectWordTarget);
	}

	function handleDailyTargetBlur() {
		if (projectDailyTarget != null && projectDailyTarget < 0) projectDailyTarget = 0;
		updateProjectTarget('daily_word_target', projectDailyTarget);
	}
</script>

{#if isOpen}
	<div class="dialog-overlay" onclick={handleOverlayClick} role="presentation">
		<!-- AY4: Add trapFocus for Escape key handling -->
		<div class="dialog-container modal-enter" use:trapFocus={{ onEscape: close }}>
			<div class="dialog-header">
				<h2>Settings</h2>
				<Button variant="icon" onclick={handleClose} title="Close">
					<Icon name="close" size={20} />
				</Button>
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

					<!-- CA4: Editor theme selector -->
					<div class="form-group">
						<label for="editor-theme">Editor Theme</label>
						<select id="editor-theme" value={settings.theme} onchange={handleThemeChange}>
							{#each editorThemes as theme (theme.value)}
								<option value={theme.value}>{theme.label}</option>
							{/each}
						</select>
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

				<section class="settings-section">
					<h3>Keyboard Shortcuts</h3>
					<p class="shortcuts-hint">Click a shortcut to reassign it. Press Escape to cancel.</p>
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="shortcuts-list" onkeydown={handleShortcutKeydown}>
						{#each shortcutActions as action (action)}
							<div class="shortcut-row">
								<span class="shortcut-label">{shortcutLabels[action]}</span>
								<button
									class="shortcut-key"
									class:recording={recordingAction === action}
									onclick={() => startRecording(action)}
								>
									{#if recordingAction === action}
										Press keys...
									{:else}
										{formatBinding(appState.keyboardShortcuts[action])}
									{/if}
								</button>
							</div>
						{/each}
					</div>
					{#if shortcutConflict}
						<div class="conflict-warning">
							Warning: This shortcut conflicts with "{shortcutConflict.conflictsWith}"
						</div>
					{/if}
					<div class="shortcuts-footer">
						<Button variant="ghost" size="sm" onclick={() => appState.resetShortcuts()}>
							Reset Shortcuts to Defaults
						</Button>
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
								min="0"
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
								min="0"
								placeholder="e.g., 1000"
								bind:value={projectDailyTarget}
								onblur={handleDailyTargetBlur}
							/>
							<span class="hint">Daily word count goal (progress shown in status bar)</span>
						</div>
					</section>
				{/if}

				<!-- UA3: Show Feature Tour -->
				<section class="settings-section">
					<h3>Help & Onboarding</h3>
					<Button
						variant="secondary"
						size="sm"
						onclick={() => {
							resetOnboarding();
							handleClose();
							showInfo(
								'Feature tour will show on next action. Refresh the page to see it immediately.'
							);
						}}
					>
						Show Feature Tour
					</Button>
				</section>

				<!-- BF5: Data Recovery Info -->
				<section class="settings-section">
					<h3>Data Recovery</h3>
					<p class="setting-info">
						Recovery drafts are automatically saved while you write and expire after 30 days of
						inactivity.
					</p>
					<p class="setting-info">
						In case of a crash or unexpected close, you'll be prompted to restore any unsaved work
						when you next open the project.
					</p>
				</section>
			</div>

			<div class="dialog-footer">
				<Button variant="ghost" onclick={resetToDefaults}>Reset to Defaults</Button>
				<Button variant="primary" onclick={handleClose}>Done</Button>
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

	/* BF5: Info text styling */
	.setting-info {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		margin: 0 0 var(--spacing-sm);
		line-height: 1.5;
	}

	.setting-info:last-child {
		margin-bottom: 0;
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

	.shortcuts-hint {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		margin-bottom: var(--spacing-md);
	}

	.shortcuts-list {
		max-height: 300px;
		overflow-y: auto;
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
	}

	.shortcut-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-xs) var(--spacing-sm);
		border-bottom: 1px solid var(--color-border);
	}

	.shortcut-row:last-child {
		border-bottom: none;
	}

	.shortcut-label {
		font-size: var(--font-size-sm);
	}

	.shortcut-key {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		font-family: monospace;
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		min-width: 80px;
		text-align: center;
		transition: all var(--transition-fast);
	}

	.shortcut-key:hover {
		background-color: var(--color-bg-hover);
		border-color: var(--color-accent);
	}

	.shortcut-key.recording {
		background-color: var(--color-accent-light);
		border-color: var(--color-accent);
		color: var(--color-accent);
		animation: pulse-border 1s infinite;
	}

	@keyframes pulse-border {
		0%,
		100% {
			border-color: var(--color-accent);
		}
		50% {
			border-color: transparent;
		}
	}

	.shortcuts-footer {
		margin-top: var(--spacing-sm);
	}

	/* V2: Conflict warning */
	.conflict-warning {
		margin-top: var(--spacing-sm);
		padding: var(--spacing-sm);
		background-color: var(--warning-subtle, oklch(90% 0.08 80));
		border: 1px solid var(--warning-border, oklch(75% 0.12 80));
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	/* Responsive adjustments */
	@media (max-width: 900px) {
		.dialog-container {
			max-width: 90%;
			padding: var(--spacing-md);
		}
		.shortcuts-list {
			max-height: 200px;
		}
	}
</style>
