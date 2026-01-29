import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
	clearRecoveryDraft,
	clearRecoveryDraftForScene,
	getAllRecoveryDrafts,
	getExpiringDrafts,
	getRecoveryDraft,
	hasRecoveryDrafts,
	saveRecoveryDraft,
} from './recovery';

describe('recovery', () => {
	beforeEach(() => {
		localStorage.clear();
		vi.restoreAllMocks();
	});

	describe('saveRecoveryDraft', () => {
		it('should save a draft to localStorage', () => {
			saveRecoveryDraft('scene-1', 'Hello world');
			const stored = localStorage.getItem('cahnon_recovery:scene-1');
			expect(stored).not.toBeNull();

			const parsed = JSON.parse(stored!);
			expect(parsed.sceneId).toBe('scene-1');
			expect(parsed.text).toBe('Hello world');
			expect(parsed.timestamp).toBeTypeOf('number');
		});

		it('should save separate drafts per scene', () => {
			saveRecoveryDraft('scene-1', 'First scene');
			saveRecoveryDraft('scene-2', 'Second scene');

			const draft1 = getRecoveryDraft('scene-1');
			const draft2 = getRecoveryDraft('scene-2');
			expect(draft1!.text).toBe('First scene');
			expect(draft2!.text).toBe('Second scene');
		});

		it('should overwrite existing draft for same scene', () => {
			saveRecoveryDraft('scene-1', 'First version');
			saveRecoveryDraft('scene-1', 'Second version');

			const draft = getRecoveryDraft('scene-1');
			expect(draft!.text).toBe('Second version');
		});

		it('should not throw on localStorage error', () => {
			vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
				throw new Error('QuotaExceededError');
			});

			expect(() => saveRecoveryDraft('scene-1', 'text')).not.toThrow();
		});
	});

	describe('getRecoveryDraft', () => {
		it('should return null when no draft exists', () => {
			expect(getRecoveryDraft('scene-1')).toBeNull();
		});

		it('should return the saved draft', () => {
			saveRecoveryDraft('scene-1', 'Saved text');
			const draft = getRecoveryDraft('scene-1');

			expect(draft).not.toBeNull();
			expect(draft!.sceneId).toBe('scene-1');
			expect(draft!.text).toBe('Saved text');
		});

		it('should return null for drafts older than 72 hours', () => {
			const oldDraft = {
				sceneId: 'scene-1',
				text: 'Old text',
				timestamp: Date.now() - 73 * 60 * 60 * 1000, // 73 hours ago
			};
			localStorage.setItem('cahnon_recovery:scene-1', JSON.stringify(oldDraft));

			expect(getRecoveryDraft('scene-1')).toBeNull();
			// Should also clear the old draft
			expect(localStorage.getItem('cahnon_recovery:scene-1')).toBeNull();
		});

		it('should return draft within 72 hours', () => {
			const recentDraft = {
				sceneId: 'scene-1',
				text: 'Recent text',
				timestamp: Date.now() - 71 * 60 * 60 * 1000, // 71 hours ago
			};
			localStorage.setItem('cahnon_recovery:scene-1', JSON.stringify(recentDraft));

			const draft = getRecoveryDraft('scene-1');
			expect(draft).not.toBeNull();
			expect(draft!.text).toBe('Recent text');
		});

		it('should return null for invalid JSON', () => {
			localStorage.setItem('cahnon_recovery:scene-1', 'invalid json {{{');
			expect(getRecoveryDraft('scene-1')).toBeNull();
		});

		it('should not throw on localStorage error', () => {
			vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
				throw new Error('SecurityError');
			});

			expect(getRecoveryDraft('scene-1')).toBeNull();
		});
	});

	describe('getAllRecoveryDrafts', () => {
		it('should return all drafts sorted by newest first', () => {
			// Use explicit timestamps to guarantee ordering
			const older = { sceneId: 'scene-1', text: 'First', timestamp: Date.now() - 5000 };
			const newer = { sceneId: 'scene-2', text: 'Second', timestamp: Date.now() };
			localStorage.setItem('cahnon_recovery:scene-1', JSON.stringify(older));
			localStorage.setItem('cahnon_recovery:scene-2', JSON.stringify(newer));

			const drafts = getAllRecoveryDrafts();
			expect(drafts).toHaveLength(2);
			expect(drafts[0].sceneId).toBe('scene-2');
		});

		it('should return empty array when no drafts exist', () => {
			expect(getAllRecoveryDrafts()).toEqual([]);
		});
	});

	describe('hasRecoveryDrafts', () => {
		it('should return false when no drafts exist', () => {
			expect(hasRecoveryDrafts()).toBe(false);
		});

		it('should return true when drafts exist', () => {
			saveRecoveryDraft('scene-1', 'text');
			expect(hasRecoveryDrafts()).toBe(true);
		});
	});

	describe('clearRecoveryDraftForScene', () => {
		it('should remove only the specified scene draft', () => {
			saveRecoveryDraft('scene-1', 'text1');
			saveRecoveryDraft('scene-2', 'text2');

			clearRecoveryDraftForScene('scene-1');
			expect(getRecoveryDraft('scene-1')).toBeNull();
			expect(getRecoveryDraft('scene-2')).not.toBeNull();
		});

		it('should not throw when no draft exists', () => {
			expect(() => clearRecoveryDraftForScene('nonexistent')).not.toThrow();
		});
	});

	describe('getExpiringDrafts', () => {
		it('should return drafts between 48 and 72 hours old', () => {
			const expiring = {
				sceneId: 'scene-1',
				text: 'Expiring text',
				timestamp: Date.now() - 50 * 60 * 60 * 1000, // 50 hours ago
			};
			localStorage.setItem('cahnon_recovery:scene-1', JSON.stringify(expiring));

			const drafts = getExpiringDrafts();
			expect(drafts).toHaveLength(1);
			expect(drafts[0].sceneId).toBe('scene-1');
		});

		it('should not return drafts less than 48 hours old', () => {
			const recent = {
				sceneId: 'scene-1',
				text: 'Recent text',
				timestamp: Date.now() - 24 * 60 * 60 * 1000, // 24 hours ago
			};
			localStorage.setItem('cahnon_recovery:scene-1', JSON.stringify(recent));

			expect(getExpiringDrafts()).toHaveLength(0);
		});

		it('should not return drafts older than 72 hours', () => {
			const expired = {
				sceneId: 'scene-1',
				text: 'Expired text',
				timestamp: Date.now() - 73 * 60 * 60 * 1000, // 73 hours ago
			};
			localStorage.setItem('cahnon_recovery:scene-1', JSON.stringify(expired));

			expect(getExpiringDrafts()).toHaveLength(0);
		});
	});

	describe('clearRecoveryDraft', () => {
		it('should remove all drafts from localStorage', () => {
			saveRecoveryDraft('scene-1', 'text1');
			saveRecoveryDraft('scene-2', 'text2');

			clearRecoveryDraft();
			expect(hasRecoveryDrafts()).toBe(false);
		});

		it('should not throw when no draft exists', () => {
			expect(() => clearRecoveryDraft()).not.toThrow();
		});

		it('should not throw on localStorage error', () => {
			vi.spyOn(Storage.prototype, 'removeItem').mockImplementation(() => {
				throw new Error('SecurityError');
			});

			expect(() => clearRecoveryDraft()).not.toThrow();
		});
	});
});
