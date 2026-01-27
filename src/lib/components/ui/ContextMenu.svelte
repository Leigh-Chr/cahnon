<!--
  Reusable context menu container.
  Positions itself at cursor location, auto-adjusts to viewport edges.
  Closes on click-outside, Escape, or item selection.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		x,
		y,
		onclose,
		children,
	}: {
		x: number;
		y: number;
		onclose: () => void;
		children: Snippet;
	} = $props();

	let menuEl: HTMLDivElement | undefined = $state();
	let menuRect = $state<{ width: number; height: number } | null>(null);

	// Measure the menu after it renders
	$effect(() => {
		if (menuEl) {
			const rect = menuEl.getBoundingClientRect();
			menuRect = { width: rect.width, height: rect.height };
		}
	});

	// Adjust position to stay within viewport (reactive to resize)
	let adjustedX = $derived.by(() => {
		if (!menuRect) return x;
		const vw = viewportSize.w;
		return x + menuRect.width > vw ? vw - menuRect.width - 4 : x;
	});

	let adjustedY = $derived.by(() => {
		if (!menuRect) return y;
		const vh = viewportSize.h;
		return y + menuRect.height > vh ? vh - menuRect.height - 4 : y;
	});

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			event.stopPropagation();
			onclose();
		}
	}

	function handleClickOutside(event: MouseEvent) {
		if (menuEl && !menuEl.contains(event.target as Node)) {
			onclose();
		}
	}

	// Track viewport size for recalculating position on resize
	let viewportSize = $state({ w: window.innerWidth, h: window.innerHeight });
	function handleResize() {
		viewportSize = { w: window.innerWidth, h: window.innerHeight };
	}

	$effect(() => {
		// Use capture to intercept before other handlers
		window.addEventListener('mousedown', handleClickOutside, true);
		window.addEventListener('keydown', handleKeydown, true);
		window.addEventListener('resize', handleResize);
		return () => {
			window.removeEventListener('mousedown', handleClickOutside, true);
			window.removeEventListener('keydown', handleKeydown, true);
			window.removeEventListener('resize', handleResize);
		};
	});
</script>

<div
	class="context-menu"
	bind:this={menuEl}
	style="left: {adjustedX}px; top: {adjustedY}px"
	role="menu"
>
	{@render children()}
</div>

<style>
	.context-menu {
		position: fixed;
		z-index: 9999;
		min-width: 180px;
		background-color: var(--surface-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--border-radius-sm);
		box-shadow: var(--shadow-lg);
		padding: var(--spacing-xs) 0;
		font-size: var(--font-size-sm);
	}
</style>
