/**
 * Toast notification system using Svelte 5 runes.
 * @module
 */

export interface Toast {
	id: string;
	type: 'info' | 'success' | 'warning' | 'error';
	message: string;
	duration?: number;
	action?: {
		label: string;
		onClick: () => void;
	};
}

/**
 * Toast state using Svelte 5 runes.
 */
class ToastState {
	items = $state<Toast[]>([]);

	private static readonly MAX_TOASTS = 10;

	add(toast: Omit<Toast, 'id'>): string {
		const id = crypto.randomUUID();
		const newToast: Toast = { ...toast, id };

		// Evict oldest toasts if at capacity
		if (this.items.length >= ToastState.MAX_TOASTS) {
			this.items = [...this.items.slice(-(ToastState.MAX_TOASTS - 1)), newToast];
		} else {
			this.items = [...this.items, newToast];
		}

		// Auto-remove after duration (default 5s, errors stay longer)
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

const toastState = new ToastState();

// Legacy-compatible store export using Svelte 5's native toStore
import { toStore } from 'svelte/store';

export const toasts = toStore(() => toastState.items);

function showToast(toast: Omit<Toast, 'id'>): string {
	return toastState.add(toast);
}

export function removeToast(id: string) {
	toastState.remove(id);
}

/** Options for toast convenience functions */
interface ToastOptions {
	action?: Toast['action'];
	duration?: number;
}

// Convenience functions
// Non-critical toasts without actions route to the status bar for a desktop feel.
export function showSuccess(message: string, options?: ToastOptions | Toast['action']) {
	// Support both old signature (action only) and new signature (options object)
	const opts: ToastOptions =
		options && typeof options === 'object' && 'label' in options
			? { action: options }
			: (options as ToastOptions) || {};

	if (!opts.action) {
		// Lazy import to avoid circular dependency
		import('$lib/stores').then(({ appState }) => {
			appState.showStatusMessage(message, 'success');
		});
		return '';
	}
	return showToast({ type: 'success', message, action: opts.action, duration: opts.duration });
}

export function showError(message: string, options?: ToastOptions | Toast['action']) {
	const opts: ToastOptions =
		options && typeof options === 'object' && 'label' in options
			? { action: options }
			: (options as ToastOptions) || {};
	return showToast({ type: 'error', message, action: opts.action, duration: opts.duration });
}

export function showWarning(message: string, options?: ToastOptions | Toast['action']) {
	const opts: ToastOptions =
		options && typeof options === 'object' && 'label' in options
			? { action: options }
			: (options as ToastOptions) || {};
	return showToast({ type: 'warning', message, action: opts.action, duration: opts.duration });
}

export function showInfo(message: string, options?: ToastOptions | Toast['action']) {
	const opts: ToastOptions =
		options && typeof options === 'object' && 'label' in options
			? { action: options }
			: (options as ToastOptions) || {};

	if (!opts.action) {
		import('$lib/stores').then(({ appState }) => {
			appState.showStatusMessage(message, 'info');
		});
		return '';
	}
	return showToast({ type: 'info', message, action: opts.action, duration: opts.duration });
}

/**
 * UB6: Show a celebration toast for achievements (goal reached, milestones).
 * Uses success type with longer duration for positive reinforcement.
 */
export function celebrate(message: string): string {
	return toastState.add({
		type: 'success',
		message,
		duration: 4000,
	});
}
