/**
 * Crash recovery utilities.
 *
 * Supports per-scene recovery drafts stored in localStorage.
 *
 * @module
 */

const RECOVERY_PREFIX = 'cahnon_recovery:';
const MAX_RECOVERY_DRAFTS = 10;

export interface RecoveryDraft {
	sceneId: string;
	text: string;
	timestamp: number;
}

/**
 * Saves a recovery draft to localStorage for a specific scene.
 * Used to recover unsaved work after a crash.
 */
export function saveRecoveryDraft(sceneId: string, text: string) {
	try {
		const draft: RecoveryDraft = {
			sceneId,
			text,
			timestamp: Date.now(),
		};
		localStorage.setItem(RECOVERY_PREFIX + sceneId, JSON.stringify(draft));
		enforceMaxDrafts();
	} catch (_e) {
		// Ignore localStorage errors (quota, etc.)
	}
}

/**
 * Retrieves the recovery draft for a specific scene.
 * Returns null if no draft exists or if the draft is older than 72 hours.
 */
export function getRecoveryDraft(sceneId: string): RecoveryDraft | null {
	try {
		const data = localStorage.getItem(RECOVERY_PREFIX + sceneId);
		if (!data) return null;

		const draft: RecoveryDraft = JSON.parse(data);

		// Only return drafts from the last 72 hours
		const THREE_DAYS = 72 * 60 * 60 * 1000;
		if (Date.now() - draft.timestamp > THREE_DAYS) {
			clearRecoveryDraftForScene(sceneId);
			return null;
		}

		return draft;
	} catch (_e) {
		return null;
	}
}

/**
 * Returns all stored recovery drafts across all scenes.
 */
export function getAllRecoveryDrafts(): RecoveryDraft[] {
	const drafts: RecoveryDraft[] = [];
	const THREE_DAYS = 72 * 60 * 60 * 1000;
	try {
		for (let i = 0; i < localStorage.length; i++) {
			const key = localStorage.key(i);
			if (!key || !key.startsWith(RECOVERY_PREFIX)) continue;
			const data = localStorage.getItem(key);
			if (!data) continue;
			const draft: RecoveryDraft = JSON.parse(data);
			if (Date.now() - draft.timestamp > THREE_DAYS) {
				localStorage.removeItem(key);
			} else {
				drafts.push(draft);
			}
		}
	} catch (_e) {
		// Ignore
	}
	return drafts.sort((a, b) => b.timestamp - a.timestamp);
}

/**
 * Returns recovery drafts that are between 48 and 72 hours old (expiring soon).
 */
export function getExpiringDrafts(): RecoveryDraft[] {
	const TWO_DAYS = 48 * 60 * 60 * 1000;
	const THREE_DAYS = 72 * 60 * 60 * 1000;
	const drafts: RecoveryDraft[] = [];
	try {
		for (let i = 0; i < localStorage.length; i++) {
			const key = localStorage.key(i);
			if (!key || !key.startsWith(RECOVERY_PREFIX)) continue;
			const data = localStorage.getItem(key);
			if (!data) continue;
			const draft: RecoveryDraft = JSON.parse(data);
			const age = Date.now() - draft.timestamp;
			if (age > TWO_DAYS && age <= THREE_DAYS) {
				drafts.push(draft);
			}
		}
	} catch (_e) {
		// Ignore
	}
	return drafts;
}

/**
 * Returns true if there are any recovery drafts stored.
 */
export function hasRecoveryDrafts(): boolean {
	try {
		for (let i = 0; i < localStorage.length; i++) {
			const key = localStorage.key(i);
			if (key && key.startsWith(RECOVERY_PREFIX)) return true;
		}
	} catch (_e) {
		// Ignore
	}
	return false;
}

/**
 * Clears the recovery draft for a specific scene.
 */
export function clearRecoveryDraftForScene(sceneId: string) {
	try {
		localStorage.removeItem(RECOVERY_PREFIX + sceneId);
	} catch (_e) {
		// Ignore
	}
}

/**
 * Clears all recovery drafts.
 */
export function clearRecoveryDraft() {
	try {
		const keysToRemove: string[] = [];
		for (let i = 0; i < localStorage.length; i++) {
			const key = localStorage.key(i);
			if (key && key.startsWith(RECOVERY_PREFIX)) {
				keysToRemove.push(key);
			}
		}
		for (const key of keysToRemove) {
			localStorage.removeItem(key);
		}
	} catch (_e) {
		// Ignore
	}
}

/**
 * Enforces MAX_RECOVERY_DRAFTS limit, removing oldest drafts first.
 */
function enforceMaxDrafts() {
	try {
		const drafts = getAllRecoveryDrafts();
		if (drafts.length <= MAX_RECOVERY_DRAFTS) return;
		// Remove oldest drafts beyond the limit
		const toRemove = drafts.slice(MAX_RECOVERY_DRAFTS);
		for (const draft of toRemove) {
			localStorage.removeItem(RECOVERY_PREFIX + draft.sceneId);
		}
	} catch (_e) {
		// Ignore
	}
}
