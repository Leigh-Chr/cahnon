export type { AnnotationTypeInfo } from './annotations';
export { annotationTypes, getAnnotationStatusColor, getAnnotationType } from './annotations';

// Word count utility
export function countWords(text: string): number {
	if (!text || text.trim().length === 0) return 0;
	// Strip HTML tags if present
	const plainText = text.replace(/<[^>]*>/g, ' ');
	// Count words
	return plainText.split(/\s+/).filter((word) => word.length > 0).length;
}

// Format word count for display
export function formatWordCount(count: number): string {
	if (count >= 1000) {
		return `${(count / 1000).toFixed(1)}k`;
	}
	return count.toString();
}

// Status colors
export const statusColors: Record<string, string> = {
	planned: 'var(--status-planned)',
	in_progress: 'var(--status-to-write)',
	to_write: 'var(--status-to-write)',
	draft: 'var(--status-draft)',
	in_revision: 'var(--status-in-revision)',
	revised: 'var(--status-in-revision)',
	done: 'var(--status-done)',
	to_cut: 'var(--status-to-cut)',
};

// Scene statuses
export const sceneStatuses = [
	{ value: 'planned', label: 'Planned' },
	{ value: 'to_write', label: 'To Write' },
	{ value: 'draft', label: 'Draft' },
	{ value: 'in_revision', label: 'In Revision' },
	{ value: 'done', label: 'Done' },
	{ value: 'to_cut', label: 'To Cut' },
];

// Chapter statuses
export const chapterStatuses = [
	{ value: 'planned', label: 'Planned' },
	{ value: 'in_progress', label: 'In Progress' },
	{ value: 'draft', label: 'Draft' },
	{ value: 'revised', label: 'Revised' },
	{ value: 'done', label: 'Done' },
];

// Bible entry types
export const bibleEntryTypes = [
	{ value: 'character', label: 'Character', icon: '👤' },
	{ value: 'location', label: 'Location', icon: '📍' },
	{ value: 'object', label: 'Object', icon: '🔮' },
	{ value: 'faction', label: 'Faction', icon: '⚔️' },
	{ value: 'concept', label: 'Concept/Rule', icon: '💡' },
	{ value: 'glossary', label: 'Glossary', icon: '📖' },
];

// Bible entry statuses
export const bibleStatuses = [
	{ value: 'draft', label: 'Draft' },
	{ value: 'canon', label: 'Canon' },
	{ value: 'tbd', label: 'TBD' },
];

// Keyboard shortcut helpers
export function isModKey(event: KeyboardEvent): boolean {
	return navigator.platform.includes('Mac') ? event.metaKey : event.ctrlKey;
}

export function formatShortcut(key: string, withMod = true, withShift = false): string {
	const isMac = navigator.platform.includes('Mac');
	const mod = isMac ? '⌘' : 'Ctrl';
	const shift = isMac ? '⇧' : 'Shift';

	const parts: string[] = [];
	if (withMod) parts.push(mod);
	if (withShift) parts.push(shift);
	parts.push(key.toUpperCase());

	return parts.join(isMac ? '' : '+');
}

// Debounce utility
export function debounce<T extends (...args: never[]) => unknown>(
	fn: T,
	delay: number
): ((...args: Parameters<T>) => void) & { cancel: () => void } {
	let timeoutId: ReturnType<typeof setTimeout>;
	const debounced = (...args: Parameters<T>) => {
		clearTimeout(timeoutId);
		timeoutId = setTimeout(() => fn(...args), delay);
	};
	debounced.cancel = () => {
		clearTimeout(timeoutId);
	};
	return debounced;
}

// Generate a simple ID (for local-only use)
export function generateId(): string {
	return Math.random().toString(36).substring(2, 15);
}

// Format date for display
export function formatDate(dateString: string): string {
	const date = new Date(dateString);
	if (isNaN(date.getTime())) return dateString;
	return date.toLocaleDateString(undefined, {
		year: 'numeric',
		month: 'short',
		day: 'numeric',
	});
}

// Format date and time for display
export function formatDateTime(dateString: string): string {
	const date = new Date(dateString);
	if (isNaN(date.getTime())) return dateString;
	return date.toLocaleString();
}

// Format relative time
export function formatRelativeTime(dateString: string): string {
	const date = new Date(dateString);
	if (isNaN(date.getTime())) return dateString;
	const now = new Date();
	const diffMs = now.getTime() - date.getTime();
	const diffMins = Math.floor(diffMs / 60000);
	const diffHours = Math.floor(diffMs / 3600000);
	const diffDays = Math.floor(diffMs / 86400000);

	if (diffMins < 1) return 'Just now';
	if (diffMins < 60) return `${diffMins}m ago`;
	if (diffHours < 24) return `${diffHours}h ago`;
	if (diffDays < 7) return `${diffDays}d ago`;
	return formatDate(dateString);
}

// Format time ago from Date object
export function formatTimeAgo(date: Date): string {
	if (isNaN(date.getTime())) return 'Unknown';
	const now = new Date();
	const diffMs = now.getTime() - date.getTime();
	const diffMins = Math.floor(diffMs / 60000);
	const diffHours = Math.floor(diffMs / 3600000);
	const diffDays = Math.floor(diffMs / 86400000);

	if (diffMins < 1) return 'Just now';
	if (diffMins < 60) return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;
	if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
	if (diffDays < 7) return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
	return date.toLocaleDateString(undefined, {
		year: 'numeric',
		month: 'short',
		day: 'numeric',
	});
}

// Strip HTML for plain text preview
export function stripHtml(html: string): string {
	return html.replace(/<[^>]*>/g, '');
}

// Truncate text
export function truncate(text: string, maxLength: number): string {
	if (text.length <= maxLength) return text;
	return text.substring(0, maxLength - 3) + '...';
}

// =============================================================================
// Text encoding utilities
// =============================================================================

/**
 * Returns the number of Unicode characters (code points) in a string.
 * Unlike string.length which returns UTF-16 code units, this correctly
 * handles emoji and other multi-byte characters.
 *
 * @example
 * charCount("hello") // 5
 * charCount("café")  // 4
 * charCount("👋")    // 1 (string.length would return 2)
 * charCount("👨‍👩‍👧")  // 5 (family emoji with ZWJ sequences)
 */
export function charCount(str: string): number {
	return [...str].length;
}

/**
 * Returns the number of UTF-8 bytes needed to encode a string.
 * Useful for matching Rust byte offsets.
 *
 * @example
 * byteCount("hello") // 5
 * byteCount("café")  // 5 (é is 2 bytes in UTF-8)
 * byteCount("👋")    // 4 (emoji is 4 bytes in UTF-8)
 */
export function byteCount(str: string): number {
	return new TextEncoder().encode(str).length;
}

/**
 * Converts a byte offset to a character offset in a UTF-8 string.
 * This is needed because Rust operates on byte offsets while
 * JavaScript/ProseMirror use character positions.
 *
 * @param text - The text string
 * @param byteOffset - The byte offset to convert
 * @returns The character offset corresponding to the byte position
 *
 * @example
 * byteOffsetToCharOffset("café", 0) // 0 (start)
 * byteOffsetToCharOffset("café", 3) // 3 (before 'é')
 * byteOffsetToCharOffset("café", 5) // 4 (end, 'é' is 2 bytes)
 * byteOffsetToCharOffset("👋hi", 4) // 1 (after emoji which is 4 bytes)
 */
export function byteOffsetToCharOffset(text: string, byteOffset: number): number {
	// Handle edge cases
	if (!text || byteOffset <= 0) return 0;

	const encoder = new TextEncoder();
	let charOffset = 0;
	let currentByteOffset = 0;

	for (const char of text) {
		// If we've reached or passed the target byte offset, return current char position
		if (currentByteOffset >= byteOffset) {
			return charOffset;
		}
		currentByteOffset += encoder.encode(char).length;
		charOffset++;
	}

	// Byte offset is at or beyond end of string
	return charOffset;
}

/**
 * Converts a character offset to a byte offset in a UTF-8 string.
 * This is the inverse of byteOffsetToCharOffset.
 *
 * @param text - The text string
 * @param charOffset - The character offset to convert
 * @returns The byte offset corresponding to the character position
 *
 * @example
 * charOffsetToByteOffset("café", 3) // 3 (before 'é')
 * charOffsetToByteOffset("café", 4) // 5 (end)
 * charOffsetToByteOffset("👋hi", 1) // 4 (after emoji)
 */
export function charOffsetToByteOffset(text: string, charOffset: number): number {
	if (!text || charOffset <= 0) return 0;

	const encoder = new TextEncoder();
	let currentCharOffset = 0;
	let byteOffset = 0;

	for (const char of text) {
		if (currentCharOffset >= charOffset) {
			return byteOffset;
		}
		byteOffset += encoder.encode(char).length;
		currentCharOffset++;
	}

	return byteOffset;
}
