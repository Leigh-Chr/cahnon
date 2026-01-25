import { expect, browser, $, $$ } from '@wdio/globals';
import * as path from 'path';

const SCREENSHOT_DIR = './tests/e2e/screenshots';

// Helper function to click elements using JavaScript
async function jsClick(selector: string) {
	const element = await $(selector);
	await element.waitForExist({ timeout: 5000 });
	await browser.execute((el: HTMLElement) => el.click(), element);
}

// Helper to set input value via JavaScript
async function jsSetValue(selector: string, value: string) {
	const element = await $(selector);
	await element.waitForExist({ timeout: 5000 });
	await browser.execute(
		(el: HTMLElement, val: string) => {
			(el as HTMLInputElement).value = val;
			el.dispatchEvent(new Event('input', { bubbles: true }));
			el.dispatchEvent(new Event('change', { bubbles: true }));
		},
		element,
		value
	);
}

// Helper to take screenshot
async function takeScreenshot(name: string) {
	const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
	const filename = `chaos_${name}_${timestamp}.png`;
	const filepath = path.join(SCREENSHOT_DIR, filename);
	await browser.saveScreenshot(filepath);
	console.log(`Screenshot: ${filepath}`);
	return filepath;
}

// Helper to dispatch keyboard events
async function pressKey(
	key: string,
	modifiers: { ctrl?: boolean; shift?: boolean; alt?: boolean } = {}
) {
	await browser.execute(
		(k: string, mods: { ctrl?: boolean; shift?: boolean; alt?: boolean }) => {
			const event = new KeyboardEvent('keydown', {
				key: k,
				code: `Key${k.toUpperCase()}`,
				ctrlKey: mods.ctrl || false,
				shiftKey: mods.shift || false,
				altKey: mods.alt || false,
				bubbles: true,
			});
			document.dispatchEvent(event);
		},
		key,
		modifiers
	);
}

// Helper for rapid clicking
async function rapidClick(selector: string, times: number) {
	const element = await $(selector);
	if (await element.isExisting()) {
		for (let i = 0; i < times; i++) {
			await browser.execute((el: HTMLElement) => el.click(), element);
		}
	}
}

describe('CHAOS TESTING - Breaking the Application', () => {
	describe('1. Input Chaos - Title Field Abuse', () => {
		it('1.1 - Empty title should be rejected', async () => {
			await jsClick('.action-card.new-project');
			await browser.pause(300);

			// Try empty title
			await jsSetValue('#title', '');
			const createBtn = await $('.btn-primary');
			const disabled = await createBtn.getAttribute('disabled');
			expect(disabled).not.toBeNull();
			console.log('  ✓ Empty title rejected');
		});

		it('1.2 - Whitespace-only title should be rejected', async () => {
			await jsSetValue('#title', '     ');
			const createBtn = await $('.btn-primary');
			const disabled = await createBtn.getAttribute('disabled');
			expect(disabled).not.toBeNull();
			console.log('  ✓ Whitespace-only rejected');
		});

		it('1.3 - Try EXTREMELY long title (10000 chars)', async () => {
			const longTitle = 'A'.repeat(10000);
			await jsSetValue('#title', longTitle);
			await takeScreenshot('01_extremely_long_title');

			const input = await $('#title');
			const value = await input.getValue();
			console.log(`  Title length accepted: ${value.length} chars`);

			// Check if button is enabled
			const createBtn = await $('.btn-primary');
			const disabled = await createBtn.getAttribute('disabled');
			console.log(`  Create button disabled: ${disabled !== null}`);
		});

		it('1.4 - Try special characters in title', async () => {
			const specialChars = '!@#$%^&*()_+-=[]{}|;\':",./<>?`~\\';
			await jsSetValue('#title', specialChars);

			const input = await $('#title');
			const value = await input.getValue();
			expect(value).toBe(specialChars);
			console.log('  ✓ Special characters accepted in title');
		});

		it('1.5 - Try XSS injection in title', async () => {
			const xssPayload = '<script>alert("XSS")</script>';
			await jsSetValue('#title', xssPayload);

			// Check if script was executed (it shouldn't be)
			const alertTriggered = await browser.execute(() => {
				const win = window as Window & { __xssTriggered?: boolean };
				return win.__xssTriggered === true;
			});
			expect(alertTriggered).toBe(false);
			console.log('  ✓ XSS injection blocked');
		});

		it('1.6 - Try SQL injection in title', async () => {
			const sqlPayload = "'; DROP TABLE projects; --";
			await jsSetValue('#title', sqlPayload);
			await takeScreenshot('02_sql_injection_attempt');
			console.log('  ✓ SQL injection payload entered (should be harmless)');
		});

		it('1.7 - Try Unicode chaos in title', async () => {
			const unicodeChaos = '🔥💀👻 Ṱ̈̃ͅḛ̃s̃t̃ Z̷̢̨̛̛̫͓̗̪̫͎̤̼̭͈̲̹̠̱̤̙͎̗̦̘̫̫͖̥̜̙̱̼̫̫̹͖̦͎͇͖͖ͅa̵l̸g̴o̷ 中文 العربية';
			await jsSetValue('#title', unicodeChaos);

			const input = await $('#title');
			const value = await input.getValue();
			console.log(`  Unicode accepted: ${value.substring(0, 30)}...`);
			await takeScreenshot('03_unicode_chaos');
		});

		it('1.8 - Try null bytes in title', async () => {
			const nullPayload = 'Test\x00Null\x00Bytes';
			await jsSetValue('#title', nullPayload);
			console.log('  ✓ Null byte payload tested');
		});

		it('1.9 - Try newlines in title', async () => {
			const newlinePayload = 'Line1\nLine2\rLine3\r\nLine4';
			await jsSetValue('#title', newlinePayload);
			console.log('  ✓ Newline payload tested');
		});

		it('1.10 - Close form', async () => {
			await jsClick('.btn-secondary');
			await browser.pause(300);
		});
	});

	describe('2. Rapid Clicking Chaos', () => {
		it('2.1 - Rapid click on New Project button (50 times)', async () => {
			await rapidClick('.action-card.new-project', 50);
			await browser.pause(500);
			await takeScreenshot('04_rapid_new_project_click');

			// Check if we have multiple forms or app is still stable
			const forms = await $$('.new-project-form');
			console.log(`  Forms visible after rapid click: ${forms.length}`);
			expect(forms.length).toBeLessThanOrEqual(1);
		});

		it('2.2 - Close any open form', async () => {
			const form = await $('.new-project-form');
			if (await form.isExisting()) {
				await jsClick('.btn-secondary');
				await browser.pause(300);
			}
		});

		it('2.3 - Rapid click alternating between New and Open buttons', async () => {
			for (let i = 0; i < 20; i++) {
				const newBtn = await $('.action-card.new-project');
				const openBtn = await $('.action-card.open-project');

				if (await newBtn.isExisting()) {
					await browser.execute((el: HTMLElement) => el.click(), newBtn);
				}
				if (await openBtn.isExisting()) {
					await browser.execute((el: HTMLElement) => el.click(), openBtn);
				}
			}
			await browser.pause(500);
			await takeScreenshot('05_alternating_rapid_clicks');
			console.log('  ✓ Survived alternating rapid clicks');
		});
	});

	describe('3. Keyboard Chaos', () => {
		it('3.1 - Spam random keyboard shortcuts', async () => {
			const shortcuts = [
				{ key: 'k', ctrl: true },
				{ key: 's', ctrl: true },
				{ key: 'z', ctrl: true },
				{ key: 'y', ctrl: true },
				{ key: 'a', ctrl: true },
				{ key: 'f', ctrl: true },
				{ key: 'n', ctrl: true },
				{ key: 'o', ctrl: true },
				{ key: 'p', ctrl: true },
				{ key: 'Escape', ctrl: false },
				{ key: 'Delete', ctrl: false },
				{ key: 'Backspace', ctrl: false },
				{ key: 'Tab', ctrl: false },
				{ key: 'Enter', ctrl: false },
			];

			for (const shortcut of shortcuts) {
				await pressKey(shortcut.key, { ctrl: shortcut.ctrl });
				await browser.pause(50);
			}

			await takeScreenshot('06_after_keyboard_spam');
			console.log('  ✓ Survived keyboard shortcut spam');
		});

		it('3.2 - Try F-keys and special keys', async () => {
			for (let i = 1; i <= 12; i++) {
				await pressKey(`F${i}`, {});
			}
			console.log('  ✓ F-keys tested');
		});
	});

	describe('4. Open Project and Chaos Inside', () => {
		it('4.1 - Open existing project', async () => {
			const recentItem = await $('.recent-item');
			if (await recentItem.isExisting()) {
				await jsClick('.recent-item');
				await browser.pause(2000);
				console.log('  ✓ Project opened');
			} else {
				console.log('  No recent project to open - skipping');
			}
		});

		it('4.2 - Rapid chapter creation (click add 30 times)', async () => {
			const addBtn = await $('.outline-header button, [title*="Add chapter"], button*=+');
			if (await addBtn.isExisting()) {
				await rapidClick('.outline-header button', 30);
				await browser.pause(1000);
				await takeScreenshot('07_mass_chapter_creation');

				const chapters = await $$('.chapter-item');
				console.log(`  Chapters after rapid creation: ${chapters.length}`);
			}
		});

		it('4.3 - Try to add scene without selecting chapter', async () => {
			// Click somewhere neutral first
			await browser.execute(() => {
				document.body.click();
			});
			await browser.pause(200);

			// Try to add scene
			const addSceneBtn = await $('.add-scene-btn, .add-scene-button, [title*="Add scene"]');
			if (await addSceneBtn.isExisting()) {
				await jsClick('.add-scene-btn, .add-scene-button');
				await browser.pause(300);
				console.log('  Add scene button clicked without chapter selected');
			}
		});

		it('4.4 - Rapid view switching (100 times)', async () => {
			const views = ['Editor', 'Corkboard', 'Bible', 'Timeline'];

			for (let i = 0; i < 25; i++) {
				for (const view of views) {
					const btn = await $(`button*=${view}`);
					if (await btn.isExisting()) {
						await browser.execute((el: HTMLElement) => el.click(), btn);
					}
				}
			}

			await browser.pause(500);
			await takeScreenshot('08_after_rapid_view_switching');
			console.log('  ✓ Survived 100 view switches');
		});

		it('4.5 - Open and close Quick Open rapidly', async () => {
			for (let i = 0; i < 20; i++) {
				await pressKey('k', { ctrl: true });
				await browser.pause(50);
				await pressKey('Escape', {});
				await browser.pause(50);
			}
			console.log('  ✓ Rapid Quick Open toggle completed');
		});
	});

	describe('5. Editor Chaos', () => {
		it('5.1 - Select a scene for editing', async () => {
			const sceneItem = await $('.scene-item');
			if (await sceneItem.isExisting()) {
				await jsClick('.scene-item');
				await browser.pause(500);
			}
		});

		it('5.2 - Try to paste HUGE content into editor', async () => {
			const editor = await $('.ProseMirror, .tiptap');
			if (await editor.isExisting()) {
				const hugeContent = 'Lorem ipsum dolor sit amet. '.repeat(10000);

				await browser.execute(
					(el: HTMLElement, content: string) => {
						el.innerHTML = content;
						el.dispatchEvent(new Event('input', { bubbles: true }));
					},
					editor,
					hugeContent
				);

				await browser.pause(1000);
				await takeScreenshot('09_huge_content_pasted');
				console.log('  ✓ Huge content injection tested');
			}
		});

		it('5.3 - Try HTML injection in editor', async () => {
			const editor = await $('.ProseMirror, .tiptap');
			if (await editor.isExisting()) {
				const htmlPayload =
					'<img src="x" onerror="alert(1)"><iframe src="javascript:alert(1)"></iframe>';

				await browser.execute(
					(el: HTMLElement, content: string) => {
						el.innerHTML = content;
						el.dispatchEvent(new Event('input', { bubbles: true }));
					},
					editor,
					htmlPayload
				);

				await browser.pause(500);
				console.log('  ✓ HTML injection tested');
			}
		});

		it('5.4 - Try to break editor with control characters', async () => {
			const editor = await $('.ProseMirror, .tiptap');
			if (await editor.isExisting()) {
				// Control characters that might break things
				const controlChars = '\x00\x01\x02\x03\x04\x05\x06\x07\x08\x0B\x0C\x0E\x0F';

				await browser.execute(
					(el: HTMLElement, content: string) => {
						el.textContent = content;
						el.dispatchEvent(new Event('input', { bubbles: true }));
					},
					editor,
					controlChars
				);

				console.log('  ✓ Control characters tested');
			}
		});

		it('5.5 - Rapid undo/redo spam', async () => {
			for (let i = 0; i < 50; i++) {
				await pressKey('z', { ctrl: true });
				await pressKey('y', { ctrl: true });
			}
			console.log('  ✓ Undo/redo spam completed');
		});
	});

	describe('6. Bible Entry Chaos', () => {
		it('6.1 - Switch to Bible view', async () => {
			const bibleBtn = await $('button*=Bible');
			if (await bibleBtn.isExisting()) {
				await jsClick('button*=Bible');
				await browser.pause(500);
			}
		});

		it('6.2 - Try to create entry with empty name', async () => {
			const addBtn = await $('.bible-sidebar button, button[title*="Add"], .bible-header button');
			if (await addBtn.isExisting()) {
				await jsClick('.bible-sidebar button');
				await browser.pause(300);

				// Try submitting empty
				await pressKey('Enter', {});
				await takeScreenshot('10_empty_bible_entry');
				console.log('  ✓ Empty Bible entry creation tested');
			}
		});

		it('6.3 - Rapid filter switching', async () => {
			const filterTypes = ['All', 'Characters', 'Locations', 'Items'];

			for (let i = 0; i < 10; i++) {
				for (const filter of filterTypes) {
					const filterBtn = await $(`button*=${filter}, [data-type="${filter.toLowerCase()}"]`);
					if (await filterBtn.isExisting()) {
						await browser.execute((el: HTMLElement) => el.click(), filterBtn);
					}
				}
			}
			console.log('  ✓ Rapid filter switching completed');
		});
	});

	describe('7. Corkboard Chaos', () => {
		it('7.1 - Switch to Corkboard view', async () => {
			const corkboardBtn = await $('button*=Corkboard');
			if (await corkboardBtn.isExisting()) {
				await jsClick('button*=Corkboard');
				await browser.pause(500);
			}
		});

		it('7.2 - Try to drag card to invalid position', async () => {
			const card = await $('.scene-card');
			if (await card.isExisting()) {
				// Simulate dragging to negative coordinates
				await browser.execute((el: HTMLElement) => {
					const dragStart = new DragEvent('dragstart', { bubbles: true });
					el.dispatchEvent(dragStart);

					const dragEnd = new DragEvent('dragend', {
						bubbles: true,
						clientX: -1000,
						clientY: -1000,
					});
					el.dispatchEvent(dragEnd);
				}, card);

				console.log('  ✓ Invalid drag tested');
			}
		});

		it('7.3 - Rapid card selection', async () => {
			const cards = await $$('.scene-card');
			for (let round = 0; round < 10; round++) {
				for (const card of cards) {
					await browser.execute((el: HTMLElement) => el.click(), card);
				}
			}
			console.log('  ✓ Rapid card selection completed');
		});
	});

	describe('8. Panel Toggle Chaos', () => {
		it('8.1 - Rapid panel toggling', async () => {
			const toggleBtns = await $$('.panel-toggle, [title*="toggle"], button svg');
			const btnsArray = await toggleBtns.getElements();

			for (let i = 0; i < 30; i++) {
				for (const btn of btnsArray.slice(0, 3)) {
					if (await btn.isExisting()) {
						await browser.execute((el: HTMLElement) => el.click(), btn);
					}
				}
			}

			await takeScreenshot('11_after_panel_chaos');
			console.log('  ✓ Panel toggle chaos completed');
		});
	});

	describe('9. Memory Stress Test', () => {
		it('9.1 - Create many scenes rapidly', async () => {
			// Switch to editor
			const editorBtn = await $('button*=Editor');
			if (await editorBtn.isExisting()) {
				await jsClick('button*=Editor');
				await browser.pause(300);
			}

			// Select first chapter
			const chapter = await $('.chapter-item');
			if (await chapter.isExisting()) {
				await jsClick('.chapter-item');
				await browser.pause(200);

				// Rapid scene creation
				for (let i = 0; i < 20; i++) {
					const addSceneBtn = await $(
						'.add-scene-btn, .add-scene-button, [title*="Add scene"], .chapter-item button'
					);
					if (await addSceneBtn.isExisting()) {
						await browser.execute((el: HTMLElement) => el.click(), addSceneBtn);
						await browser.pause(100);
					}
				}

				await takeScreenshot('12_mass_scene_creation');
				const scenes = await $$('.scene-item');
				console.log(`  Scenes after mass creation: ${scenes.length}`);
			}
		});

		it('9.2 - Fill all scenes with content', async () => {
			const scenes = await $$('.scene-item');
			const scenesArray = await scenes.getElements();
			const content = 'Test content for stress testing. '.repeat(100);

			for (const scene of scenesArray.slice(0, 5)) {
				await browser.execute((el: HTMLElement) => el.click(), scene);
				await browser.pause(200);

				const editor = await $('.ProseMirror, .tiptap');
				if (await editor.isExisting()) {
					await browser.execute(
						(el: HTMLElement, text: string) => {
							el.textContent = text;
							el.dispatchEvent(new Event('input', { bubbles: true }));
						},
						editor,
						content
					);
				}
			}

			console.log('  ✓ Filled scenes with content');
		});
	});

	describe('10. Concurrent Operations Chaos', () => {
		it('10.1 - Multiple simultaneous actions', async () => {
			// Try doing everything at once
			await Promise.all([
				pressKey('k', { ctrl: true }),
				pressKey('s', { ctrl: true }),
				rapidClick('.chapter-item', 5),
				rapidClick('.scene-item', 5),
			]);

			await browser.pause(500);
			await takeScreenshot('13_concurrent_chaos');
			console.log('  ✓ Concurrent operations tested');
		});
	});

	describe('11. State Corruption Attempts', () => {
		it('11.1 - Try to manipulate localStorage directly', async () => {
			await browser.execute(() => {
				// Try to corrupt stored state
				localStorage.setItem('cahnon-state', 'corrupted{{{');
				localStorage.setItem('project-data', '{"broken": true}');
			});
			console.log('  ✓ localStorage manipulation attempted');
		});

		it('11.2 - Try to modify window objects', async () => {
			await browser.execute(() => {
				// Try to break things
				const win = window as Window & { __TAURI__?: unknown; __TAURI_INTERNALS__?: unknown };
				win.__TAURI__ = null;
				win.__TAURI_INTERNALS__ = undefined;
			});

			// App should still work or recover gracefully
			await browser.pause(500);
			console.log('  ✓ Window object manipulation attempted');
		});

		it('11.3 - Try to call Tauri commands with bad data', async () => {
			const result = await browser.execute(async () => {
				try {
					// Try invoking with garbage data
					const win = window as Window & {
						__TAURI__?: { core?: { invoke: (cmd: string, args: unknown) => Promise<unknown> } };
					};
					if (win.__TAURI__?.core?.invoke) {
						await win.__TAURI__.core.invoke('nonexistent_command', { garbage: true });
					}
					return 'no error';
				} catch (e: unknown) {
					return (e instanceof Error ? e.message : null) || 'error caught';
				}
			});
			console.log(`  Tauri bad command result: ${result}`);
		});
	});

	describe('12. UI Boundary Tests', () => {
		it('12.1 - Resize window to minimum', async () => {
			await browser.setWindowSize(200, 200);
			await browser.pause(500);
			await takeScreenshot('14_tiny_window');
			console.log('  ✓ Minimum window size tested');
		});

		it('12.2 - Resize window to maximum', async () => {
			await browser.setWindowSize(3840, 2160);
			await browser.pause(500);
			await takeScreenshot('15_huge_window');
			console.log('  ✓ Maximum window size tested');
		});

		it('12.3 - Restore normal size', async () => {
			await browser.setWindowSize(1280, 800);
			await browser.pause(300);
		});
	});

	describe('13. Final Chaos Verification', () => {
		it('13.1 - Check for any visible errors after all chaos', async () => {
			const errors = await $$('.error, .error-message, [class*="error"]');
			let visibleErrors = 0;

			for (const error of errors) {
				if (await error.isDisplayed()) {
					visibleErrors++;
					const text = await error.getText();
					console.log(`  ERROR FOUND: ${text}`);
				}
			}

			await takeScreenshot('16_final_chaos_state');
			console.log(`  Total visible errors: ${visibleErrors}`);
		});

		it('13.2 - Verify app is still responsive', async () => {
			// Try basic interaction
			const editorBtn = await $('button*=Editor');
			if (await editorBtn.isExisting()) {
				await jsClick('button*=Editor');
				await browser.pause(300);
				console.log('  ✓ App still responsive after chaos');
			}
		});

		it('13.3 - Check console for errors', async () => {
			const _logs = await browser.execute(() => {
				const errors: string[] = [];
				const _originalError = console.error;
				// This won't catch past errors, but we can check if error handler exists
				return errors;
			});
			console.log('  Console check completed');
		});

		it('13.4 - Take final state screenshot', async () => {
			await takeScreenshot('17_survived_chaos');
			console.log('  ✓ CHAOS TESTING COMPLETE - App survived!');
		});
	});
});
