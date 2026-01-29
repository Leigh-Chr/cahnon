import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
	bibleEntryTypes,
	bibleStatuses,
	byteCount,
	byteOffsetToCharOffset,
	chapterStatuses,
	charCount,
	charOffsetToByteOffset,
	countWords,
	debounce,
	formatDate,
	formatDateTime,
	formatRelativeTime,
	formatShortcut,
	formatWordCount,
	generateId,
	isModKey,
	sceneStatuses,
	statusColors,
	stripHtml,
	truncate,
} from './index';

describe('countWords', () => {
	it('should return 0 for empty string', () => {
		expect(countWords('')).toBe(0);
	});

	it('should return 0 for whitespace only', () => {
		expect(countWords('   ')).toBe(0);
		expect(countWords('\n\t  \n')).toBe(0);
	});

	it('should count words correctly', () => {
		expect(countWords('Hello world')).toBe(2);
		expect(countWords('One two three four five')).toBe(5);
		expect(countWords('Single')).toBe(1);
	});

	it('should handle multiple spaces', () => {
		expect(countWords('Hello   world')).toBe(2);
		expect(countWords('  Hello  world  ')).toBe(2);
	});

	it('should strip HTML tags', () => {
		expect(countWords('<p>Hello world</p>')).toBe(2);
		expect(countWords('<div><span>One</span> <strong>two</strong> three</div>')).toBe(3);
	});

	it('should handle null/undefined', () => {
		expect(countWords(null as unknown as string)).toBe(0);
		expect(countWords(undefined as unknown as string)).toBe(0);
	});
});

describe('formatWordCount', () => {
	it('should return number as string for counts under 1000', () => {
		expect(formatWordCount(0)).toBe('0');
		expect(formatWordCount(100)).toBe('100');
		expect(formatWordCount(999)).toBe('999');
	});

	it('should format thousands with k suffix', () => {
		expect(formatWordCount(1000)).toBe('1.0k');
		expect(formatWordCount(1500)).toBe('1.5k');
		expect(formatWordCount(10000)).toBe('10.0k');
		expect(formatWordCount(50500)).toBe('50.5k');
	});
});

describe('stripHtml', () => {
	it('should remove all HTML tags', () => {
		expect(stripHtml('<p>Hello</p>')).toBe('Hello');
		expect(stripHtml('<div><span>Test</span></div>')).toBe('Test');
		expect(stripHtml('<a href="test">Link</a>')).toBe('Link');
	});

	it('should handle self-closing tags', () => {
		expect(stripHtml('Before<br/>After')).toBe('BeforeAfter');
		expect(stripHtml('Before<br>After')).toBe('BeforeAfter');
	});

	it('should handle plain text', () => {
		expect(stripHtml('No HTML here')).toBe('No HTML here');
	});
});

describe('truncate', () => {
	it('should not truncate short strings', () => {
		expect(truncate('Hello', 10)).toBe('Hello');
		expect(truncate('Short', 5)).toBe('Short');
	});

	it('should truncate long strings with ellipsis', () => {
		expect(truncate('Hello World', 8)).toBe('Hello...');
		expect(truncate('This is a long text', 10)).toBe('This is...');
	});

	it('should handle exact length', () => {
		expect(truncate('Hello', 5)).toBe('Hello');
	});
});

describe('generateId', () => {
	it('should return a non-empty string', () => {
		const id = generateId();
		expect(typeof id).toBe('string');
		expect(id.length).toBeGreaterThan(0);
	});

	it('should generate unique IDs', () => {
		const ids = new Set<string>();
		for (let i = 0; i < 100; i++) {
			ids.add(generateId());
		}
		expect(ids.size).toBe(100);
	});
});

describe('debounce', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});

	it('should delay function execution', () => {
		const fn = vi.fn();
		const debouncedFn = debounce(fn, 100);

		debouncedFn();
		expect(fn).not.toHaveBeenCalled();

		vi.advanceTimersByTime(100);
		expect(fn).toHaveBeenCalledTimes(1);
	});

	it('should only call once for multiple rapid calls', () => {
		const fn = vi.fn();
		const debouncedFn = debounce(fn, 100);

		debouncedFn();
		debouncedFn();
		debouncedFn();

		vi.advanceTimersByTime(100);
		expect(fn).toHaveBeenCalledTimes(1);
	});

	it('should pass arguments to the function', () => {
		const fn = vi.fn();
		const debouncedFn = debounce(fn, 100);

		debouncedFn('arg1', 'arg2');

		vi.advanceTimersByTime(100);
		expect(fn).toHaveBeenCalledWith('arg1', 'arg2');
	});

	it('should reset timer on subsequent calls', () => {
		const fn = vi.fn();
		const debouncedFn = debounce(fn, 100);

		debouncedFn();
		vi.advanceTimersByTime(50);
		debouncedFn();
		vi.advanceTimersByTime(50);

		expect(fn).not.toHaveBeenCalled();

		vi.advanceTimersByTime(50);
		expect(fn).toHaveBeenCalledTimes(1);
	});
});

describe('formatDate', () => {
	it('should format ISO date string', () => {
		const date = '2024-01-15T10:30:00Z';
		const formatted = formatDate(date);
		// The exact format depends on locale, but it should contain the date parts
		expect(formatted).toContain('2024');
		expect(formatted).toContain('15');
	});
});

describe('formatDateTime', () => {
	it('should format ISO date string with time', () => {
		const date = '2024-01-15T10:30:00Z';
		const formatted = formatDateTime(date);
		// The exact format depends on locale, but it should contain date parts
		expect(formatted).toContain('2024');
		expect(formatted).toContain('15');
	});

	it('should return original string for invalid date', () => {
		expect(formatDateTime('invalid')).toBe('invalid');
	});
});

describe('formatRelativeTime', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		vi.setSystemTime(new Date('2024-01-15T12:00:00Z'));
	});

	it('should return "Just now" for very recent times', () => {
		const date = new Date('2024-01-15T11:59:45Z').toISOString();
		expect(formatRelativeTime(date)).toBe('Just now');
	});

	it('should return minutes ago', () => {
		const date = new Date('2024-01-15T11:55:00Z').toISOString();
		expect(formatRelativeTime(date)).toBe('5m ago');
	});

	it('should return hours ago', () => {
		const date = new Date('2024-01-15T09:00:00Z').toISOString();
		expect(formatRelativeTime(date)).toBe('3h ago');
	});

	it('should return days ago', () => {
		const date = new Date('2024-01-13T12:00:00Z').toISOString();
		expect(formatRelativeTime(date)).toBe('2d ago');
	});
});

describe('Status constants', () => {
	it('should have all scene statuses defined', () => {
		expect(sceneStatuses).toHaveLength(6);
		expect(sceneStatuses.map((s) => s.value)).toContain('planned');
		expect(sceneStatuses.map((s) => s.value)).toContain('draft');
		expect(sceneStatuses.map((s) => s.value)).toContain('done');
	});

	it('should have all chapter statuses defined', () => {
		expect(chapterStatuses).toHaveLength(5);
		expect(chapterStatuses.map((s) => s.value)).toContain('planned');
		expect(chapterStatuses.map((s) => s.value)).toContain('done');
	});

	it('should have all bible entry types defined', () => {
		expect(bibleEntryTypes).toHaveLength(6);
		expect(bibleEntryTypes.map((t) => t.value)).toContain('character');
		expect(bibleEntryTypes.map((t) => t.value)).toContain('location');
		expect(bibleEntryTypes.map((t) => t.value)).toContain('object');
	});

	it('should have all bible statuses defined', () => {
		expect(bibleStatuses).toHaveLength(3);
		expect(bibleStatuses.map((s) => s.value)).toContain('draft');
		expect(bibleStatuses.map((s) => s.value)).toContain('canon');
	});

	it('should have status colors for all statuses', () => {
		expect(statusColors['planned']).toBeDefined();
		expect(statusColors['draft']).toBeDefined();
		expect(statusColors['done']).toBeDefined();
	});
});

describe('isModKey', () => {
	const originalPlatform = navigator.platform;

	afterEach(() => {
		Object.defineProperty(navigator, 'platform', {
			value: originalPlatform,
			configurable: true,
		});
	});

	it('should return metaKey on Mac', () => {
		Object.defineProperty(navigator, 'platform', {
			value: 'MacIntel',
			configurable: true,
		});
		const event = new KeyboardEvent('keydown', { metaKey: true, ctrlKey: false });
		expect(isModKey(event)).toBe(true);
	});

	it('should not return ctrlKey as mod on Mac', () => {
		Object.defineProperty(navigator, 'platform', {
			value: 'MacIntel',
			configurable: true,
		});
		const event = new KeyboardEvent('keydown', { metaKey: false, ctrlKey: true });
		expect(isModKey(event)).toBe(false);
	});

	it('should return ctrlKey on non-Mac', () => {
		Object.defineProperty(navigator, 'platform', {
			value: 'Win32',
			configurable: true,
		});
		const event = new KeyboardEvent('keydown', { metaKey: false, ctrlKey: true });
		expect(isModKey(event)).toBe(true);
	});

	it('should not return metaKey as mod on non-Mac', () => {
		Object.defineProperty(navigator, 'platform', {
			value: 'Linux x86_64',
			configurable: true,
		});
		const event = new KeyboardEvent('keydown', { metaKey: true, ctrlKey: false });
		expect(isModKey(event)).toBe(false);
	});

	it('should return false when no modifier is pressed', () => {
		Object.defineProperty(navigator, 'platform', {
			value: 'Win32',
			configurable: true,
		});
		const event = new KeyboardEvent('keydown', { metaKey: false, ctrlKey: false });
		expect(isModKey(event)).toBe(false);
	});
});

describe('formatShortcut', () => {
	const originalPlatform = navigator.platform;

	afterEach(() => {
		Object.defineProperty(navigator, 'platform', {
			value: originalPlatform,
			configurable: true,
		});
	});

	it('should format with Ctrl on non-Mac', () => {
		Object.defineProperty(navigator, 'platform', {
			value: 'Win32',
			configurable: true,
		});
		expect(formatShortcut('s')).toBe('Ctrl+S');
	});

	it('should format with command symbol on Mac', () => {
		Object.defineProperty(navigator, 'platform', {
			value: 'MacIntel',
			configurable: true,
		});
		expect(formatShortcut('s')).toBe('\u2318S');
	});

	it('should include shift on non-Mac', () => {
		Object.defineProperty(navigator, 'platform', {
			value: 'Win32',
			configurable: true,
		});
		expect(formatShortcut('z', true, true)).toBe('Ctrl+Shift+Z');
	});

	it('should include shift symbol on Mac', () => {
		Object.defineProperty(navigator, 'platform', {
			value: 'MacIntel',
			configurable: true,
		});
		expect(formatShortcut('z', true, true)).toBe('\u2318\u21E7Z');
	});

	it('should format without mod key', () => {
		Object.defineProperty(navigator, 'platform', {
			value: 'Win32',
			configurable: true,
		});
		expect(formatShortcut('f1', false)).toBe('F1');
	});

	it('should uppercase the key', () => {
		Object.defineProperty(navigator, 'platform', {
			value: 'Win32',
			configurable: true,
		});
		expect(formatShortcut('a')).toBe('Ctrl+A');
		expect(formatShortcut('A')).toBe('Ctrl+A');
	});

	it('should handle shift without mod', () => {
		Object.defineProperty(navigator, 'platform', {
			value: 'Win32',
			configurable: true,
		});
		expect(formatShortcut('tab', false, true)).toBe('Shift+TAB');
	});
});

// =============================================================================
// Text encoding utilities tests
// =============================================================================

describe('charCount', () => {
	it('should count ASCII characters correctly', () => {
		expect(charCount('hello')).toBe(5);
		expect(charCount('')).toBe(0);
		expect(charCount('a')).toBe(1);
	});

	it('should count accented characters correctly', () => {
		expect(charCount('café')).toBe(4);
		expect(charCount('naïve')).toBe(5);
		expect(charCount('résumé')).toBe(6);
	});

	it('should count emoji as single characters', () => {
		expect(charCount('👋')).toBe(1);
		expect(charCount('hi👋')).toBe(3);
		expect(charCount('👋👋👋')).toBe(3);
	});

	it('should handle ZWJ emoji sequences', () => {
		// Family emoji: 👨‍👩‍👧 is actually multiple code points joined by ZWJ
		// The spread operator counts each code point
		const family = '👨‍👩‍👧';
		// Note: ZWJ sequences are counted as multiple characters
		// because [...str] splits on code points, not grapheme clusters
		expect(charCount(family)).toBeGreaterThan(1);
	});

	it('should count Chinese characters correctly', () => {
		expect(charCount('中文')).toBe(2);
		expect(charCount('你好世界')).toBe(4);
	});

	it('should handle mixed content', () => {
		expect(charCount('Hello 世界 👋')).toBe(10); // H-e-l-l-o-space-世-界-space-👋
	});
});

describe('byteCount', () => {
	it('should count ASCII bytes correctly', () => {
		expect(byteCount('hello')).toBe(5);
		expect(byteCount('')).toBe(0);
		expect(byteCount('a')).toBe(1);
	});

	it('should count UTF-8 bytes for accented characters', () => {
		// 'é' is 2 bytes in UTF-8 (c3 a9)
		expect(byteCount('é')).toBe(2);
		expect(byteCount('café')).toBe(5); // c-a-f-é = 1+1+1+2 = 5
	});

	it('should count UTF-8 bytes for emoji', () => {
		// Emoji are typically 4 bytes in UTF-8
		expect(byteCount('👋')).toBe(4);
		expect(byteCount('hi👋')).toBe(6); // h-i-👋 = 1+1+4 = 6
	});

	it('should count UTF-8 bytes for Chinese characters', () => {
		// Chinese characters are typically 3 bytes in UTF-8
		expect(byteCount('中')).toBe(3);
		expect(byteCount('中文')).toBe(6);
	});

	it('should handle mixed content', () => {
		// "café👋" = c(1) + a(1) + f(1) + é(2) + 👋(4) = 9 bytes
		expect(byteCount('café👋')).toBe(9);
	});
});

describe('byteOffsetToCharOffset', () => {
	it('should handle ASCII text', () => {
		expect(byteOffsetToCharOffset('hello', 0)).toBe(0);
		expect(byteOffsetToCharOffset('hello', 2)).toBe(2);
		expect(byteOffsetToCharOffset('hello', 5)).toBe(5);
	});

	it('should handle empty string', () => {
		expect(byteOffsetToCharOffset('', 0)).toBe(0);
		expect(byteOffsetToCharOffset('', 5)).toBe(0);
	});

	it('should handle negative offset', () => {
		expect(byteOffsetToCharOffset('hello', -1)).toBe(0);
	});

	it('should handle accented characters', () => {
		const text = 'café';
		// Byte positions: c=0, a=1, f=2, é=3-4 (2 bytes)
		expect(byteOffsetToCharOffset(text, 0)).toBe(0); // before 'c'
		expect(byteOffsetToCharOffset(text, 1)).toBe(1); // before 'a'
		expect(byteOffsetToCharOffset(text, 2)).toBe(2); // before 'f'
		expect(byteOffsetToCharOffset(text, 3)).toBe(3); // before 'é'
		expect(byteOffsetToCharOffset(text, 5)).toBe(4); // after 'é' (end)
	});

	it('should handle emoji', () => {
		const text = 'hi👋';
		// Byte positions: h=0, i=1, 👋=2-5 (4 bytes)
		expect(byteOffsetToCharOffset(text, 0)).toBe(0); // before 'h'
		expect(byteOffsetToCharOffset(text, 1)).toBe(1); // before 'i'
		expect(byteOffsetToCharOffset(text, 2)).toBe(2); // before emoji
		expect(byteOffsetToCharOffset(text, 6)).toBe(3); // after emoji (end)
	});

	it('should handle byte offset in middle of multi-byte char', () => {
		const text = 'café';
		// 'é' spans bytes 3-4 (2 bytes in UTF-8)
		// Byte offset 4 is past the start of 'é', so we've consumed it
		// The function returns the next character position (after 'é')
		expect(byteOffsetToCharOffset(text, 4)).toBe(4);
		// Byte offset 3 is at the start of 'é', so returns position 3
		expect(byteOffsetToCharOffset(text, 3)).toBe(3);
	});

	it('should handle byte offset beyond string length', () => {
		expect(byteOffsetToCharOffset('hi', 10)).toBe(2);
	});

	it('should handle Chinese characters', () => {
		const text = '中文';
		// Each Chinese char is 3 bytes
		expect(byteOffsetToCharOffset(text, 0)).toBe(0); // before '中'
		expect(byteOffsetToCharOffset(text, 3)).toBe(1); // before '文'
		expect(byteOffsetToCharOffset(text, 6)).toBe(2); // end
	});

	it('should handle mixed ASCII and multi-byte', () => {
		const text = 'a中b';
		// a=0 (1 byte), 中=1-3 (3 bytes), b=4 (1 byte)
		expect(byteOffsetToCharOffset(text, 0)).toBe(0); // before 'a'
		expect(byteOffsetToCharOffset(text, 1)).toBe(1); // before '中'
		expect(byteOffsetToCharOffset(text, 4)).toBe(2); // before 'b'
		expect(byteOffsetToCharOffset(text, 5)).toBe(3); // end
	});
});

describe('charOffsetToByteOffset', () => {
	it('should handle ASCII text', () => {
		expect(charOffsetToByteOffset('hello', 0)).toBe(0);
		expect(charOffsetToByteOffset('hello', 2)).toBe(2);
		expect(charOffsetToByteOffset('hello', 5)).toBe(5);
	});

	it('should handle empty string', () => {
		expect(charOffsetToByteOffset('', 0)).toBe(0);
		expect(charOffsetToByteOffset('', 5)).toBe(0);
	});

	it('should handle negative offset', () => {
		expect(charOffsetToByteOffset('hello', -1)).toBe(0);
	});

	it('should handle accented characters', () => {
		const text = 'café';
		// Char positions: c=0, a=1, f=2, é=3
		// Byte positions: c=0, a=1, f=2, é=3 (start), end=5
		expect(charOffsetToByteOffset(text, 0)).toBe(0);
		expect(charOffsetToByteOffset(text, 1)).toBe(1);
		expect(charOffsetToByteOffset(text, 2)).toBe(2);
		expect(charOffsetToByteOffset(text, 3)).toBe(3);
		expect(charOffsetToByteOffset(text, 4)).toBe(5); // end
	});

	it('should handle emoji', () => {
		const text = 'hi👋';
		expect(charOffsetToByteOffset(text, 0)).toBe(0);
		expect(charOffsetToByteOffset(text, 1)).toBe(1);
		expect(charOffsetToByteOffset(text, 2)).toBe(2);
		expect(charOffsetToByteOffset(text, 3)).toBe(6); // after emoji
	});

	it('should handle Chinese characters', () => {
		const text = '中文';
		expect(charOffsetToByteOffset(text, 0)).toBe(0);
		expect(charOffsetToByteOffset(text, 1)).toBe(3);
		expect(charOffsetToByteOffset(text, 2)).toBe(6);
	});

	it('should be inverse of byteOffsetToCharOffset', () => {
		const text = 'café👋中文';

		// Test round-trip: char -> byte -> char
		for (let i = 0; i <= charCount(text); i++) {
			const byteOff = charOffsetToByteOffset(text, i);
			const charOff = byteOffsetToCharOffset(text, byteOff);
			expect(charOff).toBe(i);
		}
	});
});

describe('spellcheck position mapping scenarios', () => {
	// These tests simulate the actual spellcheck flow:
	// Rust returns byte offsets, JavaScript converts to char offsets

	it('should correctly map ASCII misspelling', () => {
		const text = 'hello wrld there';
		// "wrld" starts at byte 6, ends at byte 10
		const startChar = byteOffsetToCharOffset(text, 6);
		const endChar = byteOffsetToCharOffset(text, 10);

		expect(startChar).toBe(6);
		expect(endChar).toBe(10);
		expect(text.substring(startChar, endChar)).toBe('wrld');
	});

	it('should correctly map misspelling after accented word', () => {
		const text = 'café wrld';
		// "café" = 5 bytes (c=1, a=1, f=1, é=2)
		// space = 1 byte (position 5)
		// "wrld" starts at byte 6
		const startChar = byteOffsetToCharOffset(text, 6);
		const endChar = byteOffsetToCharOffset(text, 10);

		expect(startChar).toBe(5); // char position after "café "
		expect(endChar).toBe(9);
		expect([...text].slice(startChar, endChar).join('')).toBe('wrld');
	});

	it('should correctly map misspelling after emoji', () => {
		const text = '👋 wrld';
		// "👋" = 4 bytes
		// space = 1 byte (position 4)
		// "wrld" starts at byte 5
		const startChar = byteOffsetToCharOffset(text, 5);
		const endChar = byteOffsetToCharOffset(text, 9);

		expect(startChar).toBe(2); // char position after "👋 "
		expect(endChar).toBe(6);
		expect([...text].slice(startChar, endChar).join('')).toBe('wrld');
	});

	it('should correctly map misspelling with newlines (multi-paragraph)', () => {
		const text = 'First paragraph\nSecond wrld here';
		// "First paragraph" = 15 bytes
		// "\n" = 1 byte (position 15)
		// "Second " = 7 bytes (positions 16-22)
		// "wrld" starts at byte 23

		const startChar = byteOffsetToCharOffset(text, 23);
		const endChar = byteOffsetToCharOffset(text, 27);

		expect(text.substring(startChar, endChar)).toBe('wrld');
	});

	it('should handle French text with accents', () => {
		const text = 'Voilà une errur ici';
		// "Voilà " (with à = 2 bytes) = 7 bytes
		// "une " = 4 bytes
		// "errur" starts at byte 11

		// Find where "errur" is
		const errurByteStart = byteCount('Voilà une ');
		const errurByteEnd = errurByteStart + byteCount('errur');

		const startChar = byteOffsetToCharOffset(text, errurByteStart);
		const endChar = byteOffsetToCharOffset(text, errurByteEnd);

		expect(text.substring(startChar, endChar)).toBe('errur');
	});
});
