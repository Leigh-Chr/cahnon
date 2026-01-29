<script lang="ts" module>
	export type FindReplaceScope = 'scene' | 'chapter' | 'manuscript';

	export interface FindReplaceHandle {
		updateMatchInfo: (current: number, total: number) => void;
	}
</script>

<script lang="ts">
	import { Button, Dialog } from './ui';

	interface Props {
		isOpen?: boolean;
		showReplace?: boolean;
		scope?: FindReplaceScope;
		onfind?: (data: { query: string; caseSensitive: boolean; wholeWord: boolean }) => void;
		onreplace?: (data: {
			find: string;
			replace: string;
			caseSensitive: boolean;
			wholeWord: boolean;
		}) => void;
		onreplaceAll?: (data: {
			find: string;
			replace: string;
			caseSensitive: boolean;
			wholeWord: boolean;
			scope: FindReplaceScope;
		}) => void;
		onclose?: () => void;
		onnext?: () => void;
		onprev?: () => void;
		handle?: FindReplaceHandle;
	}

	let {
		isOpen = $bindable(false),
		showReplace = $bindable(false),
		scope = $bindable<FindReplaceScope>('scene'),
		onfind,
		onreplace,
		onreplaceAll,
		onclose,
		onnext,
		onprev,
		handle = $bindable(),
	}: Props = $props();

	let findQuery = $state('');
	let replaceQuery = $state('');
	let caseSensitive = $state(false);
	let wholeWord = $state(false);
	let matchCount = $state(0);
	let currentMatch = $state(0);

	let findInput = $state<HTMLInputElement | null>(null);

	// AX6: Confirmation dialog state (replaces nativeConfirm)
	let showReplaceAllConfirm = $state(false);
	let pendingReplaceAllData = $state<{
		find: string;
		replace: string;
		caseSensitive: boolean;
		wholeWord: boolean;
		scope: FindReplaceScope;
	} | null>(null);

	$effect(() => {
		if (isOpen && findInput) {
			findInput.focus();
		}
	});

	// Expose methods through handle prop
	$effect(() => {
		handle = {
			updateMatchInfo(current: number, total: number) {
				currentMatch = current;
				matchCount = total;
			},
		};
	});

	function handleFind() {
		if (findQuery) {
			onfind?.({ query: findQuery, caseSensitive, wholeWord });
		}
	}

	function handleReplace() {
		if (findQuery) {
			onreplace?.({ find: findQuery, replace: replaceQuery, caseSensitive, wholeWord });
		}
	}

	// AX6: Use Dialog instead of nativeConfirm for Replace All warning
	function handleReplaceAll() {
		if (!findQuery) return;

		// S4: Warn before destructive Replace All on chapter/manuscript scope
		if (scope !== 'scene') {
			pendingReplaceAllData = {
				find: findQuery,
				replace: replaceQuery,
				caseSensitive,
				wholeWord,
				scope,
			};
			showReplaceAllConfirm = true;
			return;
		}

		onreplaceAll?.({ find: findQuery, replace: replaceQuery, caseSensitive, wholeWord, scope });
	}

	function confirmReplaceAll() {
		if (pendingReplaceAllData) {
			onreplaceAll?.(pendingReplaceAllData);
		}
		showReplaceAllConfirm = false;
		pendingReplaceAllData = null;
	}

	function cancelReplaceAll() {
		showReplaceAllConfirm = false;
		pendingReplaceAllData = null;
	}

	function handleNext() {
		onnext?.();
	}

	function handlePrev() {
		onprev?.();
	}

	function close() {
		isOpen = false;
		onclose?.();
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			close();
		} else if (event.key === 'Enter') {
			if (event.shiftKey) {
				handlePrev();
			} else {
				handleFind();
				handleNext();
			}
		}
	}
</script>

{#if isOpen}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div class="find-replace" onkeydown={handleKeydown} role="search">
		<div class="find-row">
			<div class="input-wrapper">
				<!-- AP6: Add visual cue when no results -->
				<input
					bind:this={findInput}
					type="text"
					bind:value={findQuery}
					placeholder="Find..."
					oninput={handleFind}
					class="find-input"
					class:no-results={findQuery && matchCount === 0}
				/>
				{#if findQuery}
					<span class="match-count" class:no-results={matchCount === 0}>
						{#if matchCount > 0}
							{currentMatch} of {matchCount}
						{:else}
							<!-- AP6: More visible "No results" -->
							No results
						{/if}
					</span>
				{/if}
			</div>

			<div class="find-actions">
				<button
					class="option-btn"
					class:active={caseSensitive}
					onclick={() => {
						caseSensitive = !caseSensitive;
						handleFind();
					}}
					title="Case sensitive"
				>
					Aa
				</button>
				<button
					class="option-btn"
					class:active={wholeWord}
					onclick={() => {
						wholeWord = !wholeWord;
						handleFind();
					}}
					title="Whole word"
				>
					W
				</button>

				<div class="separator"></div>

				<button
					class="nav-btn"
					onclick={handlePrev}
					title="Previous match (Shift+Enter)"
					disabled={matchCount === 0}
				>
					<svg
						width="14"
						height="14"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
					>
						<polyline points="18 15 12 9 6 15" />
					</svg>
				</button>
				<button
					class="nav-btn"
					onclick={handleNext}
					title="Next match (Enter)"
					disabled={matchCount === 0}
				>
					<svg
						width="14"
						height="14"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
					>
						<polyline points="6 9 12 15 18 9" />
					</svg>
				</button>

				<div class="separator"></div>

				<button
					class="toggle-btn"
					class:active={showReplace}
					onclick={() => (showReplace = !showReplace)}
					title="Toggle replace"
				>
					<svg
						width="14"
						height="14"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
					>
						<path d="M12 5v14M5 12h14" />
					</svg>
				</button>

				<button class="close-btn" onclick={close} title="Close">
					<svg
						width="14"
						height="14"
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
		</div>

		{#if findQuery}
			<span class="keyboard-hint">Enter: next &middot; Shift+Enter: prev</span>
		{/if}

		<!-- V4: Scope selector always visible -->
		<div class="scope-row">
			<span class="scope-label">Scope:</span>
			<select bind:value={scope} class="scope-select">
				<option value="scene">Scene</option>
				<option value="chapter">Chapter</option>
				<option value="manuscript">Manuscript</option>
			</select>
		</div>

		{#if showReplace}
			<div class="replace-row">
				<input
					type="text"
					bind:value={replaceQuery}
					placeholder="Replace..."
					class="replace-input"
				/>
				<div class="replace-actions">
					<button class="action-btn" onclick={handleReplace} disabled={matchCount === 0}>
						Replace
					</button>
					<button class="action-btn" onclick={handleReplaceAll} disabled={matchCount === 0}>
						Replace All
					</button>
				</div>
			</div>
		{/if}
	</div>
{/if}

<!-- AX6: Confirmation dialog for Replace All (instead of native dialog) -->
<Dialog isOpen={showReplaceAllConfirm} title="Replace All" size="sm" onclose={cancelReplaceAll}>
	{@const scopeLabel = scope === 'chapter' ? 'the entire chapter' : 'the entire manuscript'}
	<p class="confirm-message">
		Replace all occurrences in {scopeLabel}?
	</p>
	<p class="confirm-warning">This will modify multiple scenes and cannot be easily undone.</p>

	{#snippet footer()}
		<Button variant="ghost" onclick={cancelReplaceAll}>Cancel</Button>
		<Button variant="danger" onclick={confirmReplaceAll}>Replace All</Button>
	{/snippet}
</Dialog>

<style>
	.find-replace {
		position: absolute;
		top: var(--spacing-md);
		right: var(--spacing-md);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		box-shadow: var(--shadow-lg);
		padding: var(--spacing-sm);
		z-index: 100;
		min-width: 320px;
	}

	.find-row,
	.replace-row {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
	}

	.replace-row {
		margin-top: var(--spacing-xs);
	}

	.input-wrapper {
		flex: 1;
		position: relative;
	}

	.find-input,
	.replace-input {
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
	}

	.find-input:focus,
	.replace-input:focus {
		border-color: var(--color-accent);
		outline: none;
	}

	/* AP6, AZ9: Visual cue when no results found with shake animation */
	.find-input.no-results {
		border-color: var(--color-warning);
		background-color: color-mix(in srgb, var(--color-warning) 5%, var(--color-bg-secondary));
		animation: no-results-shake 0.4s ease-out;
	}

	.find-input.no-results:focus {
		border-color: var(--color-warning);
	}

	@keyframes no-results-shake {
		0%,
		100% {
			transform: translateX(0);
		}
		20%,
		60% {
			transform: translateX(-4px);
		}
		40%,
		80% {
			transform: translateX(4px);
		}
	}

	.match-count {
		position: absolute;
		right: var(--spacing-sm);
		top: 50%;
		transform: translateY(-50%);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	/* AP6: More visible "No results" state */
	.match-count.no-results {
		color: var(--color-warning);
		font-weight: 500;
	}

	.find-actions,
	.replace-actions {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.option-btn,
	.nav-btn,
	.toggle-btn,
	.close-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-xs);
		font-weight: 500;
	}

	.option-btn:hover,
	.nav-btn:hover,
	.toggle-btn:hover,
	.close-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.option-btn.active,
	.toggle-btn.active {
		background-color: var(--color-accent-light);
		color: var(--color-accent);
	}

	.nav-btn:disabled,
	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.separator {
		width: 1px;
		height: 16px;
		background-color: var(--color-border);
		margin: 0 2px;
	}

	.action-btn {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		border-radius: var(--border-radius-sm);
	}

	.action-btn:hover:not(:disabled) {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.replace-input {
		flex: 1;
	}

	.scope-row {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		margin-top: var(--spacing-xs);
		padding-left: var(--spacing-xs);
	}

	.scope-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.scope-select {
		font-size: var(--font-size-xs);
		padding: 2px var(--spacing-xs);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
		color: var(--color-text-primary);
	}

	/* T4: Keyboard hints */
	.keyboard-hint {
		display: block;
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		padding: var(--spacing-xs) var(--spacing-sm);
	}

	/* AX6: Confirmation dialog styles */
	.confirm-message {
		margin: 0 0 var(--spacing-sm);
		font-size: var(--font-size-base);
		color: var(--color-text-primary);
	}

	.confirm-warning {
		margin: 0;
		font-size: var(--font-size-sm);
		color: var(--color-warning);
	}
</style>
