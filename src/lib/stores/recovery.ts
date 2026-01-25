/**
 * Crash recovery utilities.
 *
 * @module
 */

const RECOVERY_KEY = 'cahnon_crash_recovery';

export interface RecoveryDraft {
	sceneId: string;
	text: string;
	timestamp: number;
}

/**
 * Saves a recovery draft to localStorage.
 * Used to recover unsaved work after a crash.
 */
export function saveRecoveryDraft(sceneId: string, text: string) {
	try {
		const draft: RecoveryDraft = {
			sceneId,
			text,
			timestamp: Date.now(),
		};
		localStorage.setItem(RECOVERY_KEY, JSON.stringify(draft));
	} catch (_e) {
		// Ignore localStorage errors (quota, etc.)
	}
}

/**
 * Retrieves the recovery draft from localStorage.
 * Returns null if no draft exists or if the draft is older than 24 hours.
 */
export function getRecoveryDraft(): RecoveryDraft | null {
	try {
		const data = localStorage.getItem(RECOVERY_KEY);
		if (!data) return null;

		const draft: RecoveryDraft = JSON.parse(data);

		// Only return drafts from the last 24 hours
		const ONE_DAY = 24 * 60 * 60 * 1000;
		if (Date.now() - draft.timestamp > ONE_DAY) {
			clearRecoveryDraft();
			return null;
		}

		return draft;
	} catch (_e) {
		return null;
	}
}

/**
 * Clears the recovery draft from localStorage.
 */
export function clearRecoveryDraft() {
	try {
		localStorage.removeItem(RECOVERY_KEY);
	} catch (_e) {
		// Ignore
	}
}
