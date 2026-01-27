import { invoke } from '@tauri-apps/api/core';

import type { WritingSession } from './types';

export const writingSessionApi = {
	create: (date: string, wordsStart: number) =>
		invoke<WritingSession>('create_writing_session', {
			request: { date, words_start: wordsStart },
		}),

	getAll: () => invoke<WritingSession[]>('get_writing_sessions'),

	getByDate: (date: string) =>
		invoke<WritingSession | null>('get_writing_session_by_date', { date }),

	update: (
		id: string,
		request: { words_end?: number; duration_minutes?: number; scenes_edited?: string }
	) => invoke<WritingSession>('update_writing_session', { id, request }),

	delete: (id: string) => invoke<void>('delete_writing_session', { id }),
};
