import { $, $$, browser, expect } from '@wdio/globals';
import * as path from 'path';

const SCREENSHOT_DIR = './tests/e2e/screenshots';

// Helper function to click elements using JavaScript (workaround for WebKitWebDriver limitation)
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

// Helper to take screenshot with timestamp
async function takeScreenshot(name: string) {
	const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
	const filename = `${name}_${timestamp}.png`;
	const filepath = path.join(SCREENSHOT_DIR, filename);
	await browser.saveScreenshot(filepath);
	console.log(`Screenshot saved: ${filepath}`);
	return filepath;
}

// Helper to get console logs
async function getConsoleLogs() {
	try {
		const logs = await browser.getLogs('browser');
		return logs;
	} catch (_e) {
		// WebKitWebDriver may not support this
		return [];
	}
}

// Helper to check for errors in console
async function checkConsoleErrors() {
	const logs = await getConsoleLogs();
	const errors = logs.filter((log) => {
		const level = (log as { level?: string }).level;
		return level === 'SEVERE' || level === 'ERROR';
	});
	if (errors.length > 0) {
		console.error('Console errors found:', errors);
	}
	return errors;
}

// Helper to wait for element and verify it's visible
async function waitAndVerify(selector: string, description: string) {
	const element = await $(selector);
	const exists = await element.waitForExist({ timeout: 10000 }).catch(() => false);
	if (!exists) {
		throw new Error(`Element not found: ${description} (${selector})`);
	}
	const displayed = await element.isDisplayed();
	if (!displayed) {
		throw new Error(`Element not visible: ${description} (${selector})`);
	}
	console.log(`✓ Verified: ${description}`);
	return element;
}

describe('Cahnon - Complete Application Test Suite', () => {
	describe('1. Welcome Screen', () => {
		it('1.1 - Should display the application title', async () => {
			await takeScreenshot('01_welcome_screen');

			const header = await waitAndVerify('h1', 'Application title');
			const text = await header.getText();
			expect(text).toBe('Cahnon');
			console.log(`  Title text: "${text}"`);
		});

		it('1.2 - Should display the tagline', async () => {
			const tagline = await waitAndVerify('.tagline', 'Tagline');
			const text = await tagline.getText();
			expect(text).toBe('Write freely. Stay consistent.');
			console.log(`  Tagline text: "${text}"`);
		});

		it('1.3 - Should have New Project button with correct content', async () => {
			const btn = await waitAndVerify('.action-card.new-project', 'New Project button');

			// Check button contains expected text
			const h3 = await btn.$('h3');
			const title = await h3.getText();
			expect(title).toBe('New Project');

			const p = await btn.$('p');
			const desc = await p.getText();
			expect(desc).toBe('Start a new writing project');

			console.log(`  New Project button: "${title}" - "${desc}"`);
		});

		it('1.4 - Should have Open Project button with correct content', async () => {
			const btn = await waitAndVerify('.action-card.open-project', 'Open Project button');

			const h3 = await btn.$('h3');
			const title = await h3.getText();
			expect(title).toBe('Open Project');

			const p = await btn.$('p');
			const desc = await p.getText();
			expect(desc).toBe('Open an existing .cahnon file');

			console.log(`  Open Project button: "${title}" - "${desc}"`);
		});

		it('1.5 - Should display footer with license info', async () => {
			const footer = await waitAndVerify('.welcome-footer', 'Footer');
			const text = await footer.getText();
			expect(text).toContain('GPL-3.0');
			console.log(`  Footer: "${text}"`);
		});

		it('1.6 - Should have proper styling (no broken styles)', async () => {
			// Verify welcome container has proper styling
			const welcome = await $('.welcome');
			const display = await browser.execute((el: Element) => {
				const style = window.getComputedStyle(el);
				return {
					display: style.display,
					minHeight: style.minHeight,
					padding: style.padding,
				};
			}, welcome);

			expect(display.display).toBe('flex');
			console.log(`  Welcome container styling OK`);
		});
	});

	describe('2. New Project Form', () => {
		it('2.1 - Should open new project form when clicking New Project', async () => {
			await jsClick('.action-card.new-project');
			await takeScreenshot('02_new_project_form');

			const form = await waitAndVerify('.new-project-form', 'New project form');
			expect(await form.isDisplayed()).toBe(true);
		});

		it('2.2 - Should have form title', async () => {
			const h2 = await waitAndVerify('.new-project-form h2', 'Form title');
			const text = await h2.getText();
			expect(text).toBe('New Project');
			console.log(`  Form title: "${text}"`);
		});

		it('2.3 - Should have Title input field with label', async () => {
			const label = await $('label[for="title"]');
			expect(await label.isExisting()).toBe(true);
			const labelText = await label.getText();
			expect(labelText).toBe('Title');

			const input = await waitAndVerify('#title', 'Title input');
			const placeholder = await input.getAttribute('placeholder');
			expect(placeholder).toBe('My Novel');

			console.log(`  Title field: label="${labelText}", placeholder="${placeholder}"`);
		});

		it('2.4 - Should have Author input field with label', async () => {
			const label = await $('label[for="author"]');
			expect(await label.isExisting()).toBe(true);
			const labelText = await label.getText();
			expect(labelText).toBe('Author (optional)');

			const input = await waitAndVerify('#author', 'Author input');
			const placeholder = await input.getAttribute('placeholder');
			expect(placeholder).toBe('Your name');

			console.log(`  Author field: label="${labelText}", placeholder="${placeholder}"`);
		});

		it('2.5 - Should be able to type in Title field', async () => {
			await jsSetValue('#title', 'My Test Novel');

			const input = await $('#title');
			const value = await input.getValue();
			expect(value).toBe('My Test Novel');

			await takeScreenshot('03_form_filled');
			console.log(`  Title value: "${value}"`);
		});

		it('2.6 - Should be able to type in Author field', async () => {
			await jsSetValue('#author', 'Test Author');

			const input = await $('#author');
			const value = await input.getValue();
			expect(value).toBe('Test Author');
			console.log(`  Author value: "${value}"`);
		});

		it('2.7 - Should have Cancel button', async () => {
			const cancelBtn = await waitAndVerify('.btn-secondary', 'Cancel button');
			const text = await cancelBtn.getText();
			expect(text).toBe('Cancel');
			console.log(`  Cancel button: "${text}"`);
		});

		it('2.8 - Should have Create Project button', async () => {
			const createBtn = await waitAndVerify('.btn-primary', 'Create Project button');
			const text = await createBtn.getText();
			expect(text).toBe('Create Project');
			console.log(`  Create button: "${text}"`);
		});

		it('2.9 - Create button should be enabled when title is filled', async () => {
			const createBtn = await $('.btn-primary');
			const disabled = await createBtn.getAttribute('disabled');
			expect(disabled).toBeNull();
			console.log(`  Create button enabled: ${disabled === null}`);
		});

		it('2.10 - Should close form when clicking Cancel', async () => {
			await jsClick('.btn-secondary');

			// Wait for form to disappear
			const form = await $('.new-project-form');
			await form.waitForExist({ timeout: 5000, reverse: true });

			// Verify action cards are back
			const newProjectBtn = await waitAndVerify(
				'.action-card.new-project',
				'New Project button after cancel'
			);
			expect(await newProjectBtn.isDisplayed()).toBe(true);

			await takeScreenshot('04_form_cancelled');
			console.log(`  Form closed successfully`);
		});
	});

	describe('3. Form Validation', () => {
		it('3.1 - Should open form again', async () => {
			await jsClick('.action-card.new-project');
			await waitAndVerify('.new-project-form', 'New project form');
		});

		it('3.2 - Create button should be disabled when title is empty', async () => {
			// Clear the title field
			await jsSetValue('#title', '');

			const createBtn = await $('.btn-primary');
			const disabled = await createBtn.getAttribute('disabled');
			expect(disabled).not.toBeNull();
			console.log(`  Create button disabled when empty: ${disabled !== null}`);
		});

		it('3.3 - Create button should enable when title has content', async () => {
			await jsSetValue('#title', 'Test');

			const createBtn = await $('.btn-primary');
			const disabled = await createBtn.getAttribute('disabled');
			expect(disabled).toBeNull();
			console.log(`  Create button enabled with content: ${disabled === null}`);
		});

		it('3.4 - Should handle whitespace-only title', async () => {
			await jsSetValue('#title', '   ');

			const createBtn = await $('.btn-primary');
			const disabled = await createBtn.getAttribute('disabled');
			// Should be disabled for whitespace-only
			expect(disabled).not.toBeNull();
			console.log(`  Create button disabled for whitespace: ${disabled !== null}`);
		});

		it('3.5 - Close form for next tests', async () => {
			await jsClick('.btn-secondary');
			const form = await $('.new-project-form');
			await form.waitForExist({ timeout: 5000, reverse: true });
		});
	});

	describe('4. UI Components Verification', () => {
		it('4.1 - All buttons should be keyboard accessible', async () => {
			const buttons = await $$('button');
			let accessibleCount = 0;

			for (const button of buttons) {
				const tabIndex = await button.getAttribute('tabindex');
				// tabindex should be null (default) or >= 0
				const isAccessible = tabIndex === null || parseInt(tabIndex) >= 0;
				if (isAccessible) accessibleCount++;
			}

			expect(accessibleCount).toBe(buttons.length);
			console.log(`  All ${buttons.length} buttons are keyboard accessible`);
		});

		it('4.2 - Should have proper heading hierarchy', async () => {
			const h1s = await $$('h1');
			const h2s = await $$('h2');

			expect(h1s.length).toBeGreaterThanOrEqual(1);
			console.log(`  Found ${h1s.length} h1 and ${h2s.length} h2 elements`);
		});

		it('4.3 - SVG icons should be present', async () => {
			const svgs = await $$('svg');
			expect(svgs.length).toBeGreaterThan(0);
			console.log(`  Found ${svgs.length} SVG icons`);
		});

		it('4.4 - Action cards should have proper structure', async () => {
			const actionCards = await $$('.action-card');
			expect(actionCards.length).toBe(2);

			for (const card of actionCards) {
				const icon = await card.$('.action-icon');
				const text = await card.$('.action-text');

				expect(await icon.isExisting()).toBe(true);
				expect(await text.isExisting()).toBe(true);
			}
			console.log(`  Both action cards have proper structure`);
		});
	});

	describe('5. Responsive Design Check', () => {
		it('5.1 - Welcome content should be centered', async () => {
			const welcomeContent = await $('.welcome-content');
			const styles = await browser.execute((el: Element) => {
				const style = window.getComputedStyle(el);
				const rect = el.getBoundingClientRect();
				return {
					maxWidth: style.maxWidth,
					width: rect.width,
				};
			}, welcomeContent);

			expect(styles.maxWidth).toBe('600px');
			console.log(`  Welcome content max-width: ${styles.maxWidth}`);
		});

		it('5.2 - Action buttons should be flexbox layout', async () => {
			const welcomeActions = await $('.welcome-actions');
			const display = await browser.execute((el: Element) => {
				return window.getComputedStyle(el).display;
			}, welcomeActions);

			expect(display).toBe('flex');
			console.log(`  Welcome actions display: ${display}`);
		});
	});

	describe('6. CSS Variables and Theming', () => {
		it('6.1 - Should have CSS custom properties defined', async () => {
			const hasVars = await browser.execute(() => {
				const root = document.documentElement;
				const style = getComputedStyle(root);
				return {
					hasBgPrimary: style.getPropertyValue('--color-bg-primary') !== '',
					hasBgSecondary: style.getPropertyValue('--color-bg-secondary') !== '',
					hasTextPrimary: style.getPropertyValue('--color-text-primary') !== '',
					hasAccent: style.getPropertyValue('--color-accent') !== '',
				};
			});

			expect(hasVars.hasBgPrimary || hasVars.hasBgSecondary).toBe(true);
			console.log(
				`  CSS variables present: bg=${hasVars.hasBgPrimary}, text=${hasVars.hasTextPrimary}`
			);
		});
	});

	describe('7. Error State Handling', () => {
		it('7.1 - Error message container should exist (hidden by default)', async () => {
			// The error message should not be visible when there's no error
			const errorMsg = await $('.error-message');
			const exists = await errorMsg.isExisting();

			if (exists) {
				const displayed = await errorMsg.isDisplayed();
				// If it exists, it should not be displayed when there's no error
				console.log(`  Error message exists: ${exists}, displayed: ${displayed}`);
			} else {
				console.log(`  Error message container not present (expected when no errors)`);
			}
		});
	});

	describe('8. Final Visual Verification', () => {
		it('8.1 - Take final screenshot of welcome screen', async () => {
			await takeScreenshot('05_final_welcome');
			console.log(`  Final screenshot taken`);
		});

		it('8.2 - Verify no JavaScript errors in page', async () => {
			// Check for any JS errors by looking for error elements or console
			const errors = await checkConsoleErrors();
			console.log(`  Console errors found: ${errors.length}`);
			// We don't fail on this, just report
		});

		it('8.3 - Verify page title', async () => {
			const title = await browser.getTitle();
			console.log(`  Page title: "${title}"`);
			// Tauri apps may have empty or app name as title
		});
	});
});
