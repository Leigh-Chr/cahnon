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

// Convenience functions
export function showSuccess(message: string, action?: Toast['action']) {
	return showToast({ type: 'success', message, action });
}

export function showError(message: string, action?: Toast['action']) {
	return showToast({ type: 'error', message, action });
}

export function showWarning(message: string, action?: Toast['action']) {
	return showToast({ type: 'warning', message, action });
}

export function showInfo(message: string, action?: Toast['action']) {
	return showToast({ type: 'info', message, action });
}
