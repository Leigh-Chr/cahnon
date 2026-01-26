import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// The toast module uses Svelte 5 $state runes internally, and the `toStore`
// wrapper doesn't propagate reactive updates outside a Svelte component context.
// We test the toast logic by recreating the core class without $state,
// following the same pattern as export.test.ts.

interface Toast {
	id: string;
	type: 'info' | 'success' | 'warning' | 'error';
	message: string;
	duration?: number;
	action?: {
		label: string;
		onClick: () => void;
	};
}

class TestToastState {
	items: Toast[] = [];
	private static readonly MAX_TOASTS = 10;

	add(toast: Omit<Toast, 'id'>): string {
		const id = crypto.randomUUID();
		const newToast: Toast = { ...toast, id };

		if (this.items.length >= TestToastState.MAX_TOASTS) {
			this.items = [...this.items.slice(-(TestToastState.MAX_TOASTS - 1)), newToast];
		} else {
			this.items = [...this.items, newToast];
		}

		const duration = toast.duration ?? (toast.type === 'error' ? 8000 : 5000);
		if (duration > 0) {
			setTimeout(() => this.remove(id), duration);
		}

		return id;
	}

	remove(id: string) {
		this.items = this.items.filter((toast) => toast.id !== id);
	}
}

describe('toast notification system', () => {
	let state: TestToastState;

	beforeEach(() => {
		vi.useFakeTimers();
		state = new TestToastState();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	describe('add toast', () => {
		it('should add a success toast', () => {
			state.add({ type: 'success', message: 'Done!' });
			expect(state.items.length).toBe(1);
			expect(state.items[0].type).toBe('success');
			expect(state.items[0].message).toBe('Done!');
		});

		it('should add an error toast', () => {
			state.add({ type: 'error', message: 'Failed' });
			expect(state.items.length).toBe(1);
			expect(state.items[0].type).toBe('error');
			expect(state.items[0].message).toBe('Failed');
		});

		it('should add a warning toast', () => {
			state.add({ type: 'warning', message: 'Careful' });
			expect(state.items[0].type).toBe('warning');
		});

		it('should add an info toast', () => {
			state.add({ type: 'info', message: 'FYI' });
			expect(state.items[0].type).toBe('info');
		});

		it('should generate a unique ID for each toast', () => {
			state.add({ type: 'info', message: 'A' });
			state.add({ type: 'info', message: 'B' });
			state.add({ type: 'info', message: 'C' });

			const ids = new Set(state.items.map((t) => t.id));
			expect(ids.size).toBe(3);
		});

		it('should return the toast ID', () => {
			const id = state.add({ type: 'success', message: 'test' });
			expect(typeof id).toBe('string');
			expect(id.length).toBeGreaterThan(0);
			expect(state.items[0].id).toBe(id);
		});

		it('should include action in toast', () => {
			const onClick = vi.fn();
			state.add({ type: 'success', message: 'With action', action: { label: 'Undo', onClick } });

			expect(state.items[0].action).toBeDefined();
			expect(state.items[0].action!.label).toBe('Undo');
			state.items[0].action!.onClick();
			expect(onClick).toHaveBeenCalledTimes(1);
		});
	});

	describe('auto-remove timing', () => {
		it('should auto-remove success toast after 5 seconds', () => {
			state.add({ type: 'success', message: 'Temp' });
			expect(state.items.length).toBe(1);

			vi.advanceTimersByTime(4999);
			expect(state.items.length).toBe(1);

			vi.advanceTimersByTime(1);
			expect(state.items.length).toBe(0);
		});

		it('should auto-remove info toast after 5 seconds', () => {
			state.add({ type: 'info', message: 'Temp' });
			vi.advanceTimersByTime(5000);
			expect(state.items.length).toBe(0);
		});

		it('should auto-remove warning toast after 5 seconds', () => {
			state.add({ type: 'warning', message: 'Temp' });
			vi.advanceTimersByTime(5000);
			expect(state.items.length).toBe(0);
		});

		it('should auto-remove error toast after 8 seconds (longer duration)', () => {
			state.add({ type: 'error', message: 'Error' });
			expect(state.items.length).toBe(1);

			vi.advanceTimersByTime(5000);
			expect(state.items.length).toBe(1); // Still there at 5s

			vi.advanceTimersByTime(2999);
			expect(state.items.length).toBe(1); // Still there at 7.999s

			vi.advanceTimersByTime(1);
			expect(state.items.length).toBe(0); // Gone at 8s
		});

		it('should respect custom duration', () => {
			state.add({ type: 'success', message: 'Custom', duration: 2000 });
			vi.advanceTimersByTime(1999);
			expect(state.items.length).toBe(1);

			vi.advanceTimersByTime(1);
			expect(state.items.length).toBe(0);
		});

		it('should not auto-remove when duration is 0', () => {
			state.add({ type: 'info', message: 'Persistent', duration: 0 });
			vi.advanceTimersByTime(60000);
			expect(state.items.length).toBe(1);
		});

		it('should handle negative duration as no auto-remove', () => {
			state.add({ type: 'info', message: 'Negative', duration: -1 });
			vi.advanceTimersByTime(60000);
			expect(state.items.length).toBe(1);
		});
	});

	describe('remove toast', () => {
		it('should remove a toast by ID', () => {
			const id = state.add({ type: 'success', message: 'Remove me' });
			expect(state.items.length).toBe(1);

			state.remove(id);
			expect(state.items.length).toBe(0);
		});

		it('should not throw when removing non-existent ID', () => {
			expect(() => state.remove('non-existent')).not.toThrow();
			expect(state.items.length).toBe(0);
		});

		it('should only remove the specified toast', () => {
			state.add({ type: 'success', message: 'First' });
			state.add({ type: 'error', message: 'Second' });
			state.add({ type: 'info', message: 'Third' });

			const toRemove = state.items[1].id;
			state.remove(toRemove);

			expect(state.items.length).toBe(2);
			expect(state.items[0].message).toBe('First');
			expect(state.items[1].message).toBe('Third');
		});

		it('should handle removing already-removed toast', () => {
			const id = state.add({ type: 'info', message: 'Once' });
			state.remove(id);
			expect(() => state.remove(id)).not.toThrow();
			expect(state.items.length).toBe(0);
		});
	});

	describe('capacity eviction', () => {
		it('should hold up to 10 toasts', () => {
			for (let i = 0; i < 10; i++) {
				state.add({ type: 'info', message: `Toast ${i}` });
			}
			expect(state.items.length).toBe(10);
		});

		it('should evict oldest toast when at capacity', () => {
			for (let i = 0; i < 10; i++) {
				state.add({ type: 'info', message: `Toast ${i}` });
			}
			expect(state.items.length).toBe(10);

			// 11th toast should evict Toast 0
			state.add({ type: 'info', message: 'Toast 10' });
			expect(state.items.length).toBe(10);
			expect(state.items[0].message).toBe('Toast 1');
			expect(state.items[9].message).toBe('Toast 10');
		});

		it('should evict multiple when adding beyond capacity', () => {
			for (let i = 0; i < 12; i++) {
				state.add({ type: 'info', message: `Toast ${i}` });
			}
			expect(state.items.length).toBe(10);
			expect(state.items[0].message).toBe('Toast 2');
			expect(state.items[9].message).toBe('Toast 11');
		});

		it('should preserve newest toasts during eviction', () => {
			for (let i = 0; i < 10; i++) {
				state.add({ type: 'info', message: `Old ${i}` });
			}
			state.add({ type: 'error', message: 'Important new toast' });

			const lastToast = state.items[state.items.length - 1];
			expect(lastToast.message).toBe('Important new toast');
			expect(lastToast.type).toBe('error');
		});
	});

	describe('multiple toasts interaction', () => {
		it('should handle mixed types simultaneously', () => {
			state.add({ type: 'success', message: 'OK' });
			state.add({ type: 'error', message: 'Bad' });
			state.add({ type: 'warning', message: 'Hmm' });
			state.add({ type: 'info', message: 'FYI' });

			expect(state.items.length).toBe(4);
			expect(state.items.map((t) => t.type)).toEqual(['success', 'error', 'warning', 'info']);
		});

		it('should auto-remove each toast independently', () => {
			state.add({ type: 'success', message: 'Quick', duration: 1000 });
			state.add({ type: 'error', message: 'Slow' }); // 8s default

			vi.advanceTimersByTime(1000);
			expect(state.items.length).toBe(1);
			expect(state.items[0].message).toBe('Slow');

			vi.advanceTimersByTime(7000);
			expect(state.items.length).toBe(0);
		});

		it('should handle rapid add and remove', () => {
			const id1 = state.add({ type: 'info', message: '1' });
			const id2 = state.add({ type: 'info', message: '2' });
			state.remove(id1);
			state.add({ type: 'info', message: '3' });
			state.remove(id2);

			expect(state.items.length).toBe(1);
			expect(state.items[0].message).toBe('3');
		});
	});
});

// Also verify that the exported convenience functions exist and have correct signatures
describe('toast module exports', () => {
	it('should export showSuccess, showError, showWarning, showInfo', async () => {
		const mod = await import('./toast.svelte');
		expect(typeof mod.showSuccess).toBe('function');
		expect(typeof mod.showError).toBe('function');
		expect(typeof mod.showWarning).toBe('function');
		expect(typeof mod.showInfo).toBe('function');
	});

	it('should export removeToast', async () => {
		const mod = await import('./toast.svelte');
		expect(typeof mod.removeToast).toBe('function');
	});

	it('should export toasts store with subscribe method', async () => {
		const mod = await import('./toast.svelte');
		expect(mod.toasts).toBeDefined();
		expect(typeof mod.toasts.subscribe).toBe('function');
	});
});
