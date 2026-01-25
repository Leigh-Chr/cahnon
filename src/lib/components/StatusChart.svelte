<script lang="ts">
	import { appState } from '$lib/stores';
	import { statusColors } from '$lib/utils';
	import { EmptyState } from './ui';

	let statusData = $derived(appState.wordCounts?.by_status || []);
	let total = $derived(
		statusData.reduce((sum: number, s: { scene_count: number }) => sum + s.scene_count, 0)
	);

	// Calculate pie chart segments
	let segments = $derived(calculateSegments(statusData));

	function calculateSegments(data: typeof statusData) {
		if (data.length === 0) return [];

		const total = data.reduce((sum: number, s: { scene_count: number }) => sum + s.scene_count, 0);
		if (total === 0) return [];

		let currentAngle = 0;
		return data
			.map((item) => {
				const percentage = (item.scene_count / total) * 100;
				const angle = (item.scene_count / total) * 360;
				const startAngle = currentAngle;
				currentAngle += angle;

				return {
					status: item.status,
					count: item.scene_count,
					wordCount: item.word_count,
					percentage,
					startAngle,
					endAngle: currentAngle,
					color: statusColors[item.status] || 'var(--color-text-muted)',
				};
			})
			.filter((s) => s.count > 0);
	}

	function getArcPath(startAngle: number, endAngle: number, radius: number = 40): string {
		const start = polarToCartesian(50, 50, radius, endAngle);
		const end = polarToCartesian(50, 50, radius, startAngle);
		const largeArcFlag = endAngle - startAngle > 180 ? 1 : 0;

		return [
			'M',
			50,
			50,
			'L',
			start.x,
			start.y,
			'A',
			radius,
			radius,
			0,
			largeArcFlag,
			0,
			end.x,
			end.y,
			'Z',
		].join(' ');
	}

	function polarToCartesian(cx: number, cy: number, radius: number, angle: number) {
		const rad = ((angle - 90) * Math.PI) / 180;
		return {
			x: cx + radius * Math.cos(rad),
			y: cy + radius * Math.sin(rad),
		};
	}

	function formatStatus(status: string): string {
		return status.replace(/_/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase());
	}
</script>

<div class="status-chart">
	<h4>Scenes by Status</h4>

	{#if segments.length === 0}
		<EmptyState title="No scenes yet" />
	{:else}
		<div class="chart-container">
			<svg viewBox="0 0 100 100" class="pie-chart">
				{#each segments as segment (segment.status)}
					<path
						d={getArcPath(segment.startAngle, segment.endAngle)}
						fill={segment.color}
						class="segment"
					>
						<title
							>{formatStatus(segment.status)}: {segment.count} scenes ({segment.percentage.toFixed(
								1
							)}%)</title
						>
					</path>
				{/each}
				<!-- Center hole for donut effect -->
				<circle cx="50" cy="50" r="20" fill="var(--color-bg-primary)" />
				<text x="50" y="50" text-anchor="middle" dominant-baseline="middle" class="total-text">
					{total}
				</text>
			</svg>

			<div class="legend">
				{#each segments as segment (segment.status)}
					<div class="legend-item">
						<span class="legend-color" style="background-color: {segment.color}"></span>
						<span class="legend-label">{formatStatus(segment.status)}</span>
						<span class="legend-count">{segment.count}</span>
					</div>
				{/each}
			</div>
		</div>

		<div class="stats-summary">
			<div class="stat">
				<span class="stat-value">{total}</span>
				<span class="stat-label">Total Scenes</span>
			</div>
			<div class="stat">
				<span class="stat-value"
					>{statusData
						.reduce((sum: number, s: { word_count: number }) => sum + s.word_count, 0)
						.toLocaleString()}</span
				>
				<span class="stat-label">Total Words</span>
			</div>
		</div>
	{/if}
</div>

<style>
	.status-chart {
		padding: var(--spacing-md);
	}

	h4 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-md);
	}

	.chart-container {
		display: flex;
		align-items: center;
		gap: var(--spacing-lg);
	}

	.pie-chart {
		width: 120px;
		height: 120px;
		flex-shrink: 0;
	}

	.segment {
		transition: opacity var(--transition-fast);
		cursor: pointer;
	}

	.segment:hover {
		opacity: 0.8;
	}

	.total-text {
		font-size: 12px;
		font-weight: 600;
		fill: var(--color-text-primary);
	}

	.legend {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
		flex: 1;
	}

	.legend-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		font-size: var(--font-size-xs);
	}

	.legend-color {
		width: 12px;
		height: 12px;
		border-radius: 2px;
		flex-shrink: 0;
	}

	.legend-label {
		flex: 1;
		color: var(--color-text-secondary);
		text-transform: capitalize;
	}

	.legend-count {
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.stats-summary {
		display: flex;
		gap: var(--spacing-lg);
		margin-top: var(--spacing-md);
		padding-top: var(--spacing-md);
		border-top: 1px solid var(--color-border-light);
	}

	.stat {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.stat-value {
		font-size: var(--font-size-lg);
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.stat-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}
</style>
