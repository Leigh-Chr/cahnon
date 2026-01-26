import { beforeEach, describe, expect, it, vi } from 'vitest';

import { clearRecoveryDraft, getRecoveryDraft, saveRecoveryDraft } from './recovery';

describe('recovery', () => {
	beforeEach(() => {
		localStorage.clear();
		vi.restoreAllMocks();
	});

	describe('saveRecoveryDraft', () => {
		it('should save a draft to localStorage', () => {
			saveRecoveryDraft('scene-1', 'Hello world');
			const stored = localStorage.getItem('cahnon_crash_recovery');
			expect(stored).not.toBeNull();

			const parsed = JSON.parse(stored!);
			expect(parsed.sceneId).toBe('scene-1');
			expect(parsed.text).toBe('Hello world');
			expect(parsed.timestamp).toBeTypeOf('number');
		});

		it('should overwrite existing draft', () => {
			saveRecoveryDraft('scene-1', 'First version');
			saveRecoveryDraft('scene-2', 'Second version');

			const stored = localStorage.getItem('cahnon_crash_recovery');
			const parsed = JSON.parse(stored!);
			expect(parsed.sceneId).toBe('scene-2');
			expect(parsed.text).toBe('Second version');
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
			expect(getRecoveryDraft()).toBeNull();
		});

		it('should return the saved draft', () => {
			saveRecoveryDraft('scene-1', 'Saved text');
			const draft = getRecoveryDraft();

			expect(draft).not.toBeNull();
			expect(draft!.sceneId).toBe('scene-1');
			expect(draft!.text).toBe('Saved text');
		});

		it('should return null for drafts older than 24 hours', () => {
			const oldDraft = {
				sceneId: 'scene-1',
				text: 'Old text',
				timestamp: Date.now() - 25 * 60 * 60 * 1000, // 25 hours ago
			};
			localStorage.setItem('cahnon_crash_recovery', JSON.stringify(oldDraft));

			expect(getRecoveryDraft()).toBeNull();
			// Should also clear the old draft
			expect(localStorage.getItem('cahnon_crash_recovery')).toBeNull();
		});

		it('should return draft within 24 hours', () => {
			const recentDraft = {
				sceneId: 'scene-1',
				text: 'Recent text',
				timestamp: Date.now() - 23 * 60 * 60 * 1000, // 23 hours ago
			};
			localStorage.setItem('cahnon_crash_recovery', JSON.stringify(recentDraft));

			const draft = getRecoveryDraft();
			expect(draft).not.toBeNull();
			expect(draft!.text).toBe('Recent text');
		});

		it('should return null for invalid JSON', () => {
			localStorage.setItem('cahnon_crash_recovery', 'invalid json {{{');
			expect(getRecoveryDraft()).toBeNull();
		});

		it('should not throw on localStorage error', () => {
			vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
				throw new Error('SecurityError');
			});

			expect(getRecoveryDraft()).toBeNull();
		});
	});

	describe('clearRecoveryDraft', () => {
		it('should remove the draft from localStorage', () => {
			saveRecoveryDraft('scene-1', 'text');
			expect(localStorage.getItem('cahnon_crash_recovery')).not.toBeNull();

			clearRecoveryDraft();
			expect(localStorage.getItem('cahnon_crash_recovery')).toBeNull();
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
