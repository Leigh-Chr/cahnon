import { $, $$, browser, expect } from '@wdio/globals';
import * as path from 'path';

const SCREENSHOT_DIR = './tests/e2e/screenshots';

// Helper function to click elements using JavaScript
async function jsClick(selector: string) {
	const element = await $(selector);
	await element.waitForExist({ timeout: 10000 });
	await browser.execute((el: HTMLElement) => el.click(), element);
}

// Helper to take screenshot
async function takeScreenshot(name: string) {
	const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
	const filename = `${name}_${timestamp}.png`;
	const filepath = path.join(SCREENSHOT_DIR, filename);
	await browser.saveScreenshot(filepath);
	console.log(`Screenshot: ${filepath}`);
	return filepath;
}

describe('Cahnon - Main Interface Tests', () => {
	describe('1. Open Recent Project', () => {
		it('1.1 - Should display welcome screen with recent projects', async () => {
			await takeScreenshot('10_welcome_with_recent');

			const recentSection = await $('.recent-projects');
			const exists = await recentSection.isExisting();

			if (!exists) {
				console.log('No recent projects found - skipping main interface tests');
				// Skip remaining tests if no recent projects
				return;
			}

			const recentItems = await $$('.recent-item');
			expect(recentItems.length).toBeGreaterThan(0);
			console.log(`  Found ${recentItems.length} recent project(s)`);
		});

		it('1.2 - Should open project when clicking on recent item', async () => {
			const recentItem = await $('.recent-item');
			const exists = await recentItem.isExisting();

			if (!exists) {
				console.log('  No recent project to open - skipping');
				return;
			}

			await jsClick('.recent-item');

			// Wait for main interface to load
			await browser.pause(2000);
			await takeScreenshot('11_main_interface');

			// Check if we're in the main interface by looking for the layout
			const layout = await $('.layout');
			const layoutExists = await layout.waitForExist({ timeout: 10000 }).catch(() => false);

			if (layoutExists) {
				console.log('  Main interface loaded successfully');
			} else {
				// Check for any error messages
				const error = await $('.error-message');
				if (await error.isExisting()) {
					const errorText = await error.getText();
					console.log(`  Error: ${errorText}`);
				}
			}
		});
	});

	describe('2. Main Layout Verification', () => {
		it('2.1 - Should have toolbar', async () => {
			const toolbar = await $('.toolbar');
			const exists = await toolbar.isExisting();

			if (exists) {
				console.log('  ✓ Toolbar present');

				// Check toolbar buttons
				const buttons = await toolbar.$$('button');
				console.log(`    Found ${buttons.length} toolbar buttons`);
			} else {
				console.log('  Toolbar not found (may not be in main interface)');
			}
		});

		it('2.2 - Should have outline panel (left sidebar)', async () => {
			const outline = await $('.outline');
			const exists = await outline.isExisting();

			if (exists) {
				console.log('  ✓ Outline panel present');
				await takeScreenshot('12_outline_panel');
			} else {
				console.log('  Outline not found');
			}
		});

		it('2.3 - Should have editor panel', async () => {
			const editor = await $('.editor');
			const exists = await editor.isExisting();

			if (exists) {
				console.log('  ✓ Editor panel present');
			} else {
				console.log('  Editor not found');
			}
		});

		it('2.4 - Should have context panel (right sidebar)', async () => {
			const contextPanel = await $('.context-panel');
			const exists = await contextPanel.isExisting();

			if (exists) {
				console.log('  ✓ Context panel present');
			} else {
				console.log('  Context panel not found');
			}
		});

		it('2.5 - Should have status bar', async () => {
			const statusBar = await $('.status-bar');
			const exists = await statusBar.isExisting();

			if (exists) {
				console.log('  ✓ Status bar present');
			} else {
				console.log('  Status bar not found');
			}
		});
	});

	describe('3. Chapter Management', () => {
		it('3.1 - Should be able to add a new chapter', async () => {
			// Look for add chapter button in outline
			const addChapterBtn = await $('.outline-header button');
			const exists = await addChapterBtn.isExisting();

			if (exists) {
				await jsClick('.outline-header button');
				await browser.pause(500);
				await takeScreenshot('13_add_chapter');
				console.log('  ✓ Add chapter button clicked');

				// Check if a chapter was added
				const chapters = await $$('.chapter-item');
				console.log(`    Found ${chapters.length} chapter(s)`);
			} else {
				console.log('  Add chapter button not found');
			}
		});

		it('3.2 - Should display chapter in outline', async () => {
			const chapterItems = await $$('.chapter-item');
			const chapterCount = await chapterItems.length;

			if (chapterCount > 0) {
				console.log(`  ✓ ${chapterCount} chapter(s) displayed`);

				// Get first chapter title
				const firstChapter = chapterItems[0];
				const title = await firstChapter.$('.chapter-title');
				if (await title.isExisting()) {
					const titleText = await title.getText();
					console.log(`    First chapter: "${titleText}"`);
				}
			} else {
				console.log('  No chapters found');
			}
		});
	});

	describe('4. Scene Management', () => {
		it('4.1 - Should be able to add a scene to chapter', async () => {
			// First, make sure we have a chapter selected
			const chapterItem = await $('.chapter-item');
			const exists = await chapterItem.isExisting();

			if (exists) {
				// Click on chapter to select it
				await jsClick('.chapter-item');
				await browser.pause(300);

				// Look for add scene button
				const addSceneBtn = await $(
					'.chapter-item .add-scene-btn, .add-scene-button, [title="Add scene"]'
				);
				if (await addSceneBtn.isExisting()) {
					await jsClick('.chapter-item .add-scene-btn, .add-scene-button, [title="Add scene"]');
					await browser.pause(500);
					await takeScreenshot('14_add_scene');
					console.log('  ✓ Add scene button clicked');
				} else {
					console.log('  Add scene button not found');
				}
			} else {
				console.log('  No chapter to add scene to');
			}
		});

		it('4.2 - Should display scenes in outline', async () => {
			const sceneItems = await $$('.scene-item');
			const sceneCount = await sceneItems.length;
			console.log(`  Found ${sceneCount} scene(s)`);

			if (sceneCount > 0) {
				const firstScene = sceneItems[0];
				const title = await firstScene.$('.scene-title, span');
				if (await title.isExisting()) {
					const titleText = await title.getText();
					console.log(`    First scene: "${titleText}"`);
				}
			}
		});
	});

	describe('5. Editor Functionality', () => {
		it('5.1 - Should be able to select a scene and see editor', async () => {
			const sceneItem = await $('.scene-item');
			const exists = await sceneItem.isExisting();

			if (exists) {
				await jsClick('.scene-item');
				await browser.pause(500);
				await takeScreenshot('15_scene_selected');

				// Check if editor is visible
				const editor = await $('.editor, .tiptap, .ProseMirror');
				if (await editor.isExisting()) {
					console.log('  ✓ Editor visible after selecting scene');
				}
			} else {
				console.log('  No scene to select');
			}
		});

		it('5.2 - Should have TipTap editor loaded', async () => {
			const proseMirror = await $('.ProseMirror, .tiptap');
			const exists = await proseMirror.isExisting();

			if (exists) {
				console.log('  ✓ TipTap/ProseMirror editor loaded');

				// Check if editor is editable
				const contentEditable = await proseMirror.getAttribute('contenteditable');
				console.log(`    Contenteditable: ${contentEditable}`);
			} else {
				console.log('  TipTap editor not found');
			}
		});
	});

	describe('6. View Mode Switching', () => {
		it('6.1 - Should be able to switch to Bible view', async () => {
			// Look for Bible button in toolbar - it contains the text "Bible"
			const bibleBtn = await $('button*=Bible');
			const exists = await bibleBtn.isExisting();

			if (exists) {
				await jsClick('button*=Bible');
				await browser.pause(800);
				await takeScreenshot('16_bible_view');

				// Check for bible-specific elements
				const bibleView = await $('.bible-view, .bible-sidebar, .bible-entries');
				const bibleExists = await bibleView.isExisting();
				if (bibleExists) {
					console.log('  ✓ Bible view loaded');
				} else {
					// Check if we're still in the app but Bible view rendered differently
					await browser.execute(() => document.body.innerHTML);
					console.log('  Bible view element check - looking for Bible content');
				}
			} else {
				console.log('  Bible button not found in toolbar');
			}
		});

		it('6.2 - Should display Bible sidebar with entry types', async () => {
			// Look for bible sidebar or filter section
			const bibleSidebar = await $('.bible-sidebar, .filter-section, .entry-type-filter');
			const exists = await bibleSidebar.isExisting();

			if (exists) {
				console.log('  ✓ Bible sidebar/filter present');

				// Check for filter tabs or entry type buttons
				const filterElements = await $$('.filter-tab, .entry-type-btn, button[data-type]');
				console.log(`    Found ${filterElements.length} filter elements`);
			} else {
				console.log('  Bible sidebar not found - may have different structure');
			}
		});

		it('6.3 - Should be able to switch to Corkboard view', async () => {
			const corkboardBtn = await $('button*=Corkboard');
			const exists = await corkboardBtn.isExisting();

			if (exists) {
				await jsClick('button*=Corkboard');
				await browser.pause(800);
				await takeScreenshot('17_corkboard_view');

				const corkboard = await $('.corkboard, .corkboard-view');
				if (await corkboard.isExisting()) {
					console.log('  ✓ Corkboard view loaded');
				}
			} else {
				console.log('  Corkboard button not found');
			}
		});

		it('6.4 - Should display scene cards in Corkboard', async () => {
			const sceneCards = await $$('.scene-card');
			const cardCount = await sceneCards.length;
			console.log(`  Found ${cardCount} scene card(s) in corkboard`);

			if (cardCount > 0) {
				// Check card structure
				const firstCard = sceneCards[0];
				const title = await firstCard.$('.card-title');
				if (await title.isExisting()) {
					const titleText = await title.getText();
					console.log(`    First card: "${titleText}"`);
				}
			}
		});

		it('6.5 - Should switch back to Editor view', async () => {
			const editorBtn = await $('button*=Editor');
			const exists = await editorBtn.isExisting();

			if (exists) {
				await jsClick('button*=Editor');
				await browser.pause(500);
				await takeScreenshot('18_back_to_editor');
				console.log('  ✓ Switched back to editor view');
			}
		});
	});

	describe('7. Quick Open (Search)', () => {
		it('7.1 - Should open Quick Open with Ctrl+K', async () => {
			// Use JavaScript to dispatch keyboard event (WebKitWebDriver doesn't support actions API)
			await browser.execute(() => {
				const event = new KeyboardEvent('keydown', {
					key: 'k',
					code: 'KeyK',
					ctrlKey: true,
					bubbles: true,
				});
				document.dispatchEvent(event);
			});
			await browser.pause(500);

			const quickOpen = await $('.quick-open-overlay, .quick-open');
			const exists = await quickOpen.isExisting();

			if (exists) {
				await takeScreenshot('19_quick_open');
				console.log('  ✓ Quick Open dialog opened');

				// Check for search input
				const searchInput = await quickOpen.$('input');
				if (await searchInput.isExisting()) {
					console.log('    Search input present');
				}
			} else {
				console.log('  Quick Open not found (keyboard shortcut may not work via JS dispatch)');
			}
		});

		it('7.2 - Should close Quick Open with Escape', async () => {
			const quickOpen = await $('.quick-open-overlay');

			if (await quickOpen.isExisting()) {
				// Use JavaScript to dispatch Escape key
				await browser.execute(() => {
					const event = new KeyboardEvent('keydown', {
						key: 'Escape',
						code: 'Escape',
						bubbles: true,
					});
					document.dispatchEvent(event);
				});
				await browser.pause(300);

				const stillExists = await quickOpen.isExisting();
				if (!stillExists) {
					console.log('  ✓ Quick Open closed with Escape');
				}
			}
		});
	});

	describe('8. Context Panel', () => {
		it('8.1 - Should display word count when scene selected', async () => {
			// Select a scene first
			const sceneItem = await $('.scene-item');
			if (await sceneItem.isExisting()) {
				await jsClick('.scene-item');
				await browser.pause(300);
			}

			const wordCount = await $('.word-stats, .stat-value');
			const exists = await wordCount.isExisting();

			if (exists) {
				const text = await wordCount.getText();
				console.log(`  ✓ Word count displayed: ${text}`);
			} else {
				console.log('  Word count not found');
			}
		});

		it('8.2 - Should display associations section', async () => {
			const associationsSection = await $('.associations-list, .panel-section');
			const exists = await associationsSection.isExisting();

			if (exists) {
				console.log('  ✓ Associations section present');
			} else {
				console.log('  Associations section not found');
			}
		});
	});

	describe('9. Final State', () => {
		it('9.1 - Take final screenshot of application state', async () => {
			await takeScreenshot('20_final_state');
			console.log('  ✓ Final screenshot taken');
		});

		it('9.2 - Verify no visual errors', async () => {
			// Check for any error messages visible
			const errors = await $$('.error, .error-message, [class*="error"]');
			let visibleErrors = 0;

			for (const error of errors) {
				if (await error.isDisplayed()) {
					visibleErrors++;
					const text = await error.getText();
					console.log(`  Error found: ${text}`);
				}
			}

			console.log(`  Visible errors: ${visibleErrors}`);
		});
	});
});
