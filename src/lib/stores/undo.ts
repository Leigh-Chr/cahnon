/**
 * Global Undo Stack (UD1, UD2)
 *
 * Provides application-level undo/redo for destructive operations
 * like scene deletion, chapter deletion, and bible entry deletion.
 */

import { showError, showInfo, showSuccess } from '$lib/toast.svelte';

export interface UndoAction {
	type: 'scene' | 'chapter' | 'bible';
	action: 'create' | 'update' | 'delete';
	entityId: string;
	entityName: string;
	previousState: unknown;
	timestamp: number;
	/** Function to restore the previous state */
	restore: () => Promise<void>;
}

const MAX_UNDO_STACK_SIZE = 50;

class UndoStack {
	private stack: UndoAction[] = [];

	/** Push a new action to the undo stack. */
	push(action: UndoAction) {
		this.stack.push(action);
		// Limit stack size
		if (this.stack.length > MAX_UNDO_STACK_SIZE) {
			this.stack.shift();
		}
	}

	/** Undo the last action. Returns true if an action was undone. */
	async undo(): Promise<boolean> {
		const action = this.stack.pop();
		if (!action) {
			showInfo('Nothing to undo');
			return false;
		}

		try {
			await action.restore();
			showSuccess(`Restored "${action.entityName}"`);
			return true;
		} catch (e) {
			console.error('Failed to undo:', e);
			showError(`Failed to restore "${action.entityName}"`);
			// Put the action back on the stack
			this.stack.push(action);
			return false;
		}
	}

	/** Check if there are actions to undo. */
	get canUndo(): boolean {
		return this.stack.length > 0;
	}

	/** Clear the undo stack (e.g., when project changes). */
	clear() {
		this.stack = [];
	}
}

// Singleton instance
export const undoStack = new UndoStack();
