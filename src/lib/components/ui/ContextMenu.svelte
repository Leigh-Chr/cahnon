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

	// X2: Save focus to restore on close
	let previouslyFocused: Element | null = null;

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

	function getMenuItems(): HTMLElement[] {
		if (!menuEl) return [];
		return Array.from(
			menuEl.querySelectorAll<HTMLElement>('[role="menuitem"]:not([aria-disabled="true"])')
		);
	}

	let activeIndex = $state(-1);

	function focusItem(index: number) {
		const items = getMenuItems();
		if (items.length === 0) return;
		activeIndex = ((index % items.length) + items.length) % items.length;
		items[activeIndex]?.focus();
	}

	function handleKeydown(event: KeyboardEvent) {
		switch (event.key) {
			case 'Escape':
				event.preventDefault();
				event.stopPropagation();
				onclose();
				break;
			case 'ArrowDown':
				event.preventDefault();
				event.stopPropagation();
				focusItem(activeIndex + 1);
				break;
			case 'ArrowUp':
				event.preventDefault();
				event.stopPropagation();
				focusItem(activeIndex - 1);
				break;
			case 'Home':
				event.preventDefault();
				event.stopPropagation();
				focusItem(0);
				break;
			case 'End': {
				event.preventDefault();
				event.stopPropagation();
				const items = getMenuItems();
				focusItem(items.length - 1);
				break;
			}
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
		// X2: Save focus to restore on close
		previouslyFocused = document.activeElement;

		// Use capture to intercept before other handlers
		window.addEventListener('mousedown', handleClickOutside, true);
		window.addEventListener('keydown', handleKeydown, true);
		window.addEventListener('resize', handleResize);
		// Auto-focus the first menu item
		requestAnimationFrame(() => focusItem(0));
		return () => {
			window.removeEventListener('mousedown', handleClickOutside, true);
			window.removeEventListener('keydown', handleKeydown, true);
			window.removeEventListener('resize', handleResize);

			// X2: Restore focus on close
			if (previouslyFocused instanceof HTMLElement) {
				previouslyFocused.focus();
			}
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
	/* Phase 3.4: Enhanced context menu with entrance animation */
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
		animation: context-menu-enter 0.15s ease-out;
		transform-origin: top left;
	}

	@keyframes context-menu-enter {
		from {
			opacity: 0;
			transform: scale(0.95);
		}
		to {
			opacity: 1;
			transform: scale(1);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.context-menu {
			animation: none;
		}
	}
</style>
