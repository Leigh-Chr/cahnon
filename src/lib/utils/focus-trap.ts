/**
 * Focus trap utility for modals and dialogs.
 * Keeps keyboard focus within a container element, cycling through focusable elements.
 */

const FOCUSABLE_SELECTOR =
	'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

function createFocusTrap(container: HTMLElement) {
	let previouslyFocused: Element | null = null;

	function getFocusableElements(): HTMLElement[] {
		return Array.from(container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)).filter(
			(el) => !el.hasAttribute('disabled') && el.offsetParent !== null
		);
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key !== 'Tab') return;

		const focusable = getFocusableElements();
		if (focusable.length === 0) {
			event.preventDefault();
			return;
		}

		const first = focusable[0];
		const last = focusable[focusable.length - 1];

		if (event.shiftKey) {
			if (document.activeElement === first || !container.contains(document.activeElement)) {
				event.preventDefault();
				last.focus();
			}
		} else {
			if (document.activeElement === last || !container.contains(document.activeElement)) {
				event.preventDefault();
				first.focus();
			}
		}
	}

	function activate() {
		previouslyFocused = document.activeElement;
		container.addEventListener('keydown', handleKeydown);
		// Focus the first focusable element or the container itself
		const focusable = getFocusableElements();
		if (focusable.length > 0) {
			focusable[0].focus();
		} else {
			container.focus();
		}
	}

	function deactivate() {
		container.removeEventListener('keydown', handleKeydown);
		if (previouslyFocused instanceof HTMLElement) {
			previouslyFocused.focus();
		}
	}

	return { activate, deactivate };
}

/**
 * Svelte action for focus trapping.
 * Usage: use:trapFocus={{ onEscape: () => closeModal() }}
 */
export function trapFocus(
	node: HTMLElement,
	params?: { onEscape?: () => void }
): { update: (params?: { onEscape?: () => void }) => void; destroy: () => void } {
	let currentParams = params;
	const trap = createFocusTrap(node);

	function handleEscape(event: KeyboardEvent) {
		if (event.key === 'Escape' && currentParams?.onEscape) {
			event.preventDefault();
			event.stopPropagation();
			currentParams.onEscape();
		}
	}

	node.addEventListener('keydown', handleEscape);
	trap.activate();

	return {
		update(newParams?: { onEscape?: () => void }) {
			currentParams = newParams;
		},
		destroy() {
			node.removeEventListener('keydown', handleEscape);
			trap.deactivate();
		},
	};
}
