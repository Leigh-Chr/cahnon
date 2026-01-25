// Mocks for $app module
import { vi } from 'vitest';

export const goto = vi.fn();
export const invalidate = vi.fn();
export const invalidateAll = vi.fn();

export const page = {
	subscribe: vi.fn((fn: (value: { url: URL; params: Record<string, string> }) => void) => {
		fn({ url: new URL('http://localhost'), params: {} });
		return () => {};
	}),
};

export const navigating = {
	subscribe: vi.fn((fn: (value: null) => void) => {
		fn(null);
		return () => {};
	}),
};

export const updated = {
	subscribe: vi.fn((fn: (value: boolean) => void) => {
		fn(false);
		return () => {};
	}),
	check: vi.fn(),
};
