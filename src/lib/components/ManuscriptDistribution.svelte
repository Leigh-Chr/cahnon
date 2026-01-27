<!--
  Manuscript Distribution visualization.

  Shows two SVG charts:
  1. Horizontal bar chart of word count per chapter.
  2. Pie chart of word count by status (draft/revision/done).
-->
<script lang="ts">
	import { appState } from '$lib/stores';

	// ---- Bar chart data: word count per chapter ----
	let chapterData = $derived(appState.wordCounts?.by_chapter || []);
	let maxChapterWords = $derived(chapterData.reduce((max, ch) => Math.max(max, ch.word_count), 0));

	// Bar chart dimensions
	const barPadding = { top: 10, right: 60, bottom: 10, left: 130 };
	const barRowHeight = 28;
	let barViewWidth = 500;
	let barViewHeight = $derived(
		barPadding.top + barPadding.bottom + chapterData.length * barRowHeight
	);
	let barChartWidth = $derived(barViewWidth - barPadding.left - barPadding.right);

	function barWidth(wordCount: number): number {
		if (maxChapterWords === 0) return 0;
		return (wordCount / maxChapterWords) * barChartWidth;
	}

	// ---- Pie chart data: word count by status ----
	let statusData = $derived(appState.wordCounts?.by_status || []);
	let totalWords = $derived(statusData.reduce((sum, s) => sum + s.word_count, 0));

	function statusColor(status: string): string {
		switch (status) {
			case 'draft':
				return 'var(--color-warning)';
			case 'revision':
				return 'var(--color-info)';
			case 'done':
				return 'var(--color-success)';
			default:
				return 'var(--color-text-muted)';
		}
	}

	// Pie chart geometry
	const pieRadius = 50;
	const pieCenter = 60;
	const pieViewSize = 120;

	interface PieSegment {
		status: string;
		wordCount: number;
		percentage: number;
		startAngle: number;
		endAngle: number;
		color: string;
	}

	let pieSegments = $derived.by((): PieSegment[] => {
		if (totalWords === 0) return [];
		let currentAngle = 0;
		return statusData
			.map((item) => {
				const percentage = (item.word_count / totalWords) * 100;
				const angle = (item.word_count / totalWords) * 360;
				const startAngle = currentAngle;
				currentAngle += angle;
				return {
					status: item.status,
					wordCount: item.word_count,
					percentage,
					startAngle,
					endAngle: currentAngle,
					color: statusColor(item.status),
				};
			})
			.filter((s) => s.wordCount > 0);
	});

	function polarToCartesian(cx: number, cy: number, radius: number, angle: number) {
		const rad = ((angle - 90) * Math.PI) / 180;
		return {
			x: cx + radius * Math.cos(rad),
			y: cy + radius * Math.sin(rad),
		};
	}

	function arcPath(startAngle: number, endAngle: number): string {
		const start = polarToCartesian(pieCenter, pieCenter, pieRadius, endAngle);
		const end = polarToCartesian(pieCenter, pieCenter, pieRadius, startAngle);
		const largeArcFlag = endAngle - startAngle > 180 ? 1 : 0;
		return [
			'M',
			pieCenter,
			pieCenter,
			'L',
			start.x,
			start.y,
			'A',
			pieRadius,
			pieRadius,
			0,
			largeArcFlag,
			0,
			end.x,
			end.y,
			'Z',
		].join(' ');
	}

	function formatStatus(status: string): string {
		return status.replace(/_/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase());
	}

	function formatNumber(n: number): string {
		return n.toLocaleString();
	}
</script>

<div class="manuscript-distribution">
	<div class="charts-row">
		<!-- Bar chart: words per chapter -->
		<div class="chart-section bar-section">
			<h4>Words by Chapter</h4>
			{#if chapterData.length === 0}
				<p class="empty-msg">No chapters yet.</p>
			{:else}
				<svg
					viewBox="0 0 {barViewWidth} {barViewHeight}"
					preserveAspectRatio="xMidYMid meet"
					class="bar-svg"
					role="img"
					aria-label="Word count per chapter bar chart"
				>
					{#each chapterData as chapter, i (chapter.chapter_id)}
						{@const y = barPadding.top + i * barRowHeight}
						{@const w = barWidth(chapter.word_count)}

						<!-- Chapter title -->
						<text
							x={barPadding.left - 6}
							y={y + barRowHeight / 2}
							text-anchor="end"
							dominant-baseline="middle"
							class="bar-label"
						>
							{chapter.chapter_title.length > 18
								? chapter.chapter_title.slice(0, 16) + '..'
								: chapter.chapter_title}
						</text>

						<!-- Bar -->
						<rect
							x={barPadding.left}
							y={y + 4}
							width={Math.max(w, 2)}
							height={barRowHeight - 8}
							rx="3"
							class="bar-rect"
						>
							<title>{chapter.chapter_title}: {formatNumber(chapter.word_count)} words</title>
						</rect>

						<!-- Word count label -->
						<text
							x={barPadding.left + w + 6}
							y={y + barRowHeight / 2}
							dominant-baseline="middle"
							class="bar-count"
						>
							{formatNumber(chapter.word_count)}
						</text>
					{/each}
				</svg>
			{/if}
		</div>

		<!-- Pie chart: words by status -->
		<div class="chart-section pie-section">
			<h4>Words by Status</h4>
			{#if pieSegments.length === 0}
				<p class="empty-msg">No data.</p>
			{:else}
				<div class="pie-container">
					<svg
						viewBox="0 0 {pieViewSize} {pieViewSize}"
						class="pie-svg"
						role="img"
						aria-label="Word count by status pie chart"
					>
						{#each pieSegments as seg (seg.status)}
							{#if pieSegments.length === 1}
								<!-- Single full circle -->
								<circle
									cx={pieCenter}
									cy={pieCenter}
									r={pieRadius}
									fill={seg.color}
									class="pie-segment"
								>
									<title
										>{formatStatus(seg.status)}: {formatNumber(seg.wordCount)} words ({seg.percentage.toFixed(
											1
										)}%)</title
									>
								</circle>
							{:else}
								<path
									d={arcPath(seg.startAngle, seg.endAngle)}
									fill={seg.color}
									class="pie-segment"
								>
									<title
										>{formatStatus(seg.status)}: {formatNumber(seg.wordCount)} words ({seg.percentage.toFixed(
											1
										)}%)</title
									>
								</path>
							{/if}
						{/each}
						<!-- Donut hole -->
						<circle cx={pieCenter} cy={pieCenter} r="22" fill="var(--color-bg-primary)" />
						<text
							x={pieCenter}
							y={pieCenter}
							text-anchor="middle"
							dominant-baseline="middle"
							class="pie-center-text"
						>
							{formatNumber(totalWords)}
						</text>
					</svg>

					<div class="pie-legend">
						{#each pieSegments as seg (seg.status)}
							<div class="legend-item">
								<span class="legend-dot" style="background-color: {seg.color}"></span>
								<span class="legend-label">{formatStatus(seg.status)}</span>
								<span class="legend-value">{seg.percentage.toFixed(0)}%</span>
							</div>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.manuscript-distribution {
		width: 100%;
	}

	.charts-row {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: var(--spacing-lg);
		align-items: start;
	}

	.chart-section h4 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
		margin: 0 0 var(--spacing-sm) 0;
	}

	.bar-section {
		min-width: 0;
	}

	.bar-svg {
		width: 100%;
		height: auto;
	}

	.bar-label {
		font-size: 9px;
		fill: var(--color-text-secondary);
	}

	.bar-rect {
		fill: var(--color-accent);
		opacity: 0.85;
		transition: opacity 0.15s ease;
	}

	.bar-rect:hover {
		opacity: 1;
	}

	.bar-count {
		font-size: 8px;
		fill: var(--color-text-muted);
	}

	.pie-section {
		min-width: 200px;
	}

	.pie-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.pie-svg {
		width: 120px;
		height: 120px;
	}

	.pie-segment {
		transition: opacity 0.15s ease;
	}

	.pie-segment:hover {
		opacity: 0.8;
	}

	.pie-center-text {
		font-size: 10px;
		font-weight: 600;
		fill: var(--color-text-primary);
	}

	.pie-legend {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.legend-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-xs);
	}

	.legend-dot {
		width: 10px;
		height: 10px;
		border-radius: 2px;
		flex-shrink: 0;
	}

	.legend-label {
		color: var(--color-text-secondary);
		flex: 1;
	}

	.legend-value {
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.empty-msg {
		text-align: center;
		color: var(--color-text-muted);
		padding: var(--spacing-md);
		font-size: var(--font-size-sm);
	}
</style>
