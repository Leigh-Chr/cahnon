<!--
  Vertical splitter handle for resizable panels.
  Uses Pointer Events with setPointerCapture for reliable dragging.
-->
<script lang="ts">
	let {
		position,
		min = 180,
		max = 500,
		side = 'left',
		onresize,
	}: {
		position: number;
		min?: number;
		max?: number;
		side?: 'left' | 'right';
		onresize: (newWidth: number) => void;
	} = $props();

	let isDragging = $state(false);
	let startX = 0;
	let startWidth = 0;

	function handlePointerDown(event: PointerEvent) {
		isDragging = true;
		startX = event.clientX;
		startWidth = position;
		(event.target as HTMLElement).setPointerCapture(event.pointerId);
		event.preventDefault();
	}

	function handlePointerMove(event: PointerEvent) {
		if (!isDragging) return;
		const delta = side === 'left' ? event.clientX - startX : startX - event.clientX;
		const newWidth = Math.round(Math.min(max, Math.max(min, startWidth + delta)));
		onresize(newWidth);
	}

	function handlePointerUp(event: PointerEvent) {
		if (!isDragging) return;
		isDragging = false;
		(event.target as HTMLElement).releasePointerCapture(event.pointerId);
	}
</script>

<div
	class="splitter"
	class:dragging={isDragging}
	onpointerdown={handlePointerDown}
	onpointermove={handlePointerMove}
	onpointerup={handlePointerUp}
	role="separator"
	aria-orientation="vertical"
	aria-valuenow={position}
	aria-valuemin={min}
	aria-valuemax={max}
></div>

<style>
	.splitter {
		width: 4px;
		cursor: col-resize;
		background-color: transparent;
		position: relative;
		flex-shrink: 0;
		z-index: 10;
	}

	/* Larger hit area */
	.splitter::before {
		content: '';
		position: absolute;
		top: 0;
		bottom: 0;
		left: -2px;
		right: -2px;
	}

	.splitter:hover {
		background-color: var(--accent-default);
		opacity: 0.4;
	}

	.splitter.dragging {
		background-color: var(--accent-default);
		opacity: 0.6;
	}
</style>
