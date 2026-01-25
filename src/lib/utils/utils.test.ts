import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
	countWords,
	formatWordCount,
	stripHtml,
	truncate,
	debounce,
	generateId,
	formatDate,
	formatRelativeTime,
	statusColors,
	sceneStatuses,
	chapterStatuses,
	bibleEntryTypes,
	bibleStatuses,
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
