/**
 * Layout utilities for the graphical timeline view.
 * Positions scenes and events along a narrative axis based on time_point values.
 */

import type { Scene, TimelineEvent } from '$lib/api';

interface TimelineItem {
	id: string;
	title: string;
	type: 'scene' | 'event';
	timePoint: string;
	status?: string;
	lane: number;
}

interface TimelineColumn {
	timePoint: string;
	items: TimelineItem[];
}

/**
 * Extract unique time points from scenes and events, preserving narrative order.
 */
function extractTimePoints(scenes: Scene[], events: TimelineEvent[]): string[] {
	const seen = new Set<string>();
	const points: string[] = [];

	for (const scene of scenes) {
		if (scene.time_point && !seen.has(scene.time_point)) {
			seen.add(scene.time_point);
			points.push(scene.time_point);
		}
	}
	for (const event of events) {
		if (event.time_point && !seen.has(event.time_point)) {
			seen.add(event.time_point);
			points.push(event.time_point);
		}
	}

	return points;
}

/**
 * Build columns for the graphical timeline, grouping items by time_point.
 */
export function buildTimelineColumns(scenes: Scene[], events: TimelineEvent[]): TimelineColumn[] {
	const timePoints = extractTimePoints(scenes, events);
	const columns: TimelineColumn[] = [];

	for (const tp of timePoints) {
		const items: TimelineItem[] = [];

		for (const scene of scenes) {
			if (scene.time_point === tp) {
				items.push({
					id: scene.id,
					title: scene.title,
					type: 'scene',
					timePoint: tp,
					status: scene.status,
					lane: items.length,
				});
			}
		}

		for (const event of events) {
			if (event.time_point === tp) {
				items.push({
					id: event.id,
					title: event.title,
					type: 'event',
					timePoint: tp,
					lane: items.length,
				});
			}
		}

		if (items.length > 0) {
			columns.push({ timePoint: tp, items });
		}
	}

	return columns;
}
