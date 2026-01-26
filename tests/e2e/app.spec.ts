import { $, browser, expect } from '@wdio/globals';

// Helper function to click elements using JavaScript (workaround for WebKitWebDriver limitation)
async function jsClick(selector: string) {
	const element = await $(selector);
	await element.waitForExist({ timeout: 5000 });
	await browser.execute((el: HTMLElement) => el.click(), element);
}

describe('Cahnon Application', () => {
	describe('Welcome Screen', () => {
		it('should display the welcome screen on startup', async () => {
			// Wait for the app to load
			const header = await $('h1');
			await header.waitForExist({ timeout: 10000 });

			const headerText = await header.getText();
			expect(headerText).toBe('Cahnon');
		});

		it('should display the tagline', async () => {
			const tagline = await $('.tagline');
			await tagline.waitForExist({ timeout: 5000 });

			const taglineText = await tagline.getText();
			expect(taglineText).toBe('Write freely. Stay consistent.');
		});

		it('should have New Project and Open Project buttons', async () => {
			const newProjectBtn = await $('.action-card.new-project');
			const openProjectBtn = await $('.action-card.open-project');

			expect(await newProjectBtn.isExisting()).toBe(true);
			expect(await openProjectBtn.isExisting()).toBe(true);
		});
	});

	describe('Project Creation', () => {
		it('should show the new project form when clicking New Project', async () => {
			// Use JavaScript click to work around WebKitWebDriver limitation
			await jsClick('.action-card.new-project');

			const form = await $('.new-project-form');
			await form.waitForExist({ timeout: 5000 });

			expect(await form.isDisplayed()).toBe(true);
		});

		it('should have title and author input fields', async () => {
			const titleInput = await $('#title');
			const authorInput = await $('#author');

			await titleInput.waitForExist({ timeout: 5000 });

			expect(await titleInput.isExisting()).toBe(true);
			expect(await authorInput.isExisting()).toBe(true);
		});

		it('should be able to enter a project title', async () => {
			const titleInput = await $('#title');
			await titleInput.waitForExist({ timeout: 5000 });

			// Use JavaScript to set value (more reliable with WebKitWebDriver)
			await browser.execute(
				(el: HTMLElement, value: string) => {
					(el as HTMLInputElement).value = value;
					el.dispatchEvent(new Event('input', { bubbles: true }));
				},
				titleInput,
				'Test Novel'
			);

			const value = await titleInput.getValue();
			expect(value).toBe('Test Novel');
		});

		it('should be able to cancel project creation', async () => {
			await jsClick('.btn-secondary');

			// Wait for form to disappear
			const form = await $('.new-project-form');
			await form.waitForExist({ timeout: 5000, reverse: true });

			// Should be back to the action cards
			const newProjectBtn = await $('.action-card.new-project');
			await newProjectBtn.waitForExist({ timeout: 5000 });
			expect(await newProjectBtn.isDisplayed()).toBe(true);
		});
	});

	// Note: Testing actual project creation requires
	// file dialog interaction which is handled by the OS
});

describe('UI Components', () => {
	describe('Accessibility', () => {
		it('should have proper heading structure', async () => {
			const h1 = await $('h1');
			expect(await h1.isExisting()).toBe(true);
		});

		it('should have keyboard accessible buttons', async () => {
			const buttons = await $$('button');
			for (const button of buttons) {
				// Buttons should be focusable
				const tabIndex = await button.getAttribute('tabindex');
				expect(tabIndex === null || parseInt(tabIndex) >= 0).toBe(true);
			}
		});
	});
});
