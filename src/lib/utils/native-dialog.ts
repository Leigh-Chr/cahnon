/**
 * Native dialog utilities using Tauri's dialog plugin.
 * Replaces browser confirm() with OS-native dialogs.
 * @module
 */

import { confirm } from '@tauri-apps/plugin-dialog';

/**
 * Show a native OS confirmation dialog.
 * Returns true if the user clicked OK/Yes, false for Cancel/No.
 */
export async function nativeConfirm(message: string, title?: string): Promise<boolean> {
	return confirm(message, { title: title ?? 'Cahnon', okLabel: 'OK', cancelLabel: 'Cancel' });
}
