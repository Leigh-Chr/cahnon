/**
 * Shared annotation type metadata and helpers.
 *
 * Used by both Editor (tooltip rendering) and AnnotationsPanel (list rendering).
 */

export interface AnnotationTypeInfo {
	value: string;
	label: string;
	icon: string;
	color: string;
}

export const annotationTypes: AnnotationTypeInfo[] = [
	{ value: 'comment', label: 'Comment', icon: '\u{1F4AC}', color: '#e6b422' },
	{ value: 'question', label: 'Question', icon: '\u{2753}', color: '#4a90d9' },
	{ value: 'todo', label: 'TODO', icon: '\u{2705}', color: '#e08a2b' },
	{ value: 'research', label: 'Research', icon: '\u{1F50D}', color: '#9b6ed0' },
	{ value: 'revision', label: 'Revision', icon: '\u{270F}\u{FE0F}', color: '#4caf7c' },
];

export function getAnnotationType(type: string): AnnotationTypeInfo {
	return (
		annotationTypes.find((t) => t.value === type) || {
			value: type,
			label: type,
			icon: '\u{1F4DD}',
			color: '#888',
		}
	);
}

export function getAnnotationStatusColor(status: string): string {
	const colors: Record<string, string> = {
		open: 'var(--color-warning)',
		in_progress: 'var(--color-info)',
		resolved: 'var(--color-success)',
	};
	return colors[status] || 'var(--color-text-muted)';
}
