import { describe, expect, it } from 'vitest';

// We test the pure utility functions from export.ts.
// Since htmlToPlainText, escapeHtml, and sanitizeHtml are not exported,
// we test them indirectly or recreate the logic for testing.
// The export functions themselves depend on external libraries (docx, pdf-lib, mammoth).

// Test the helper logic that's used throughout the export module
describe('export utility logic', () => {
	describe('htmlToPlainText logic', () => {
		function htmlToPlainText(html: string): string {
			return html
				.replace(/<br\s*\/?>/gi, '\n')
				.replace(/<\/p>/gi, '\n\n')
				.replace(/<\/div>/gi, '\n')
				.replace(/<[^>]+>/g, '')
				.replace(/&nbsp;/g, ' ')
				.replace(/&amp;/g, '&')
				.replace(/&lt;/g, '<')
				.replace(/&gt;/g, '>')
				.replace(/&quot;/g, '"')
				.replace(/&#39;/g, "'")
				.trim();
		}

		it('should convert basic HTML to plain text', () => {
			expect(htmlToPlainText('<p>Hello world</p>')).toBe('Hello world');
		});

		it('should handle paragraphs', () => {
			const result = htmlToPlainText('<p>First</p><p>Second</p>');
			expect(result).toContain('First');
			expect(result).toContain('Second');
		});

		it('should handle br tags', () => {
			expect(htmlToPlainText('Line 1<br>Line 2')).toBe('Line 1\nLine 2');
			expect(htmlToPlainText('Line 1<br/>Line 2')).toBe('Line 1\nLine 2');
			expect(htmlToPlainText('Line 1<br />Line 2')).toBe('Line 1\nLine 2');
		});

		it('should decode HTML entities', () => {
			expect(htmlToPlainText('&amp; &lt; &gt; &quot; &#39;')).toBe('& < > " \'');
		});

		it('should handle nbsp', () => {
			expect(htmlToPlainText('Hello&nbsp;world')).toBe('Hello world');
		});

		it('should handle nested tags', () => {
			expect(htmlToPlainText('<div><span><strong>Bold text</strong></span></div>')).toBe(
				'Bold text'
			);
		});

		it('should handle empty string', () => {
			expect(htmlToPlainText('')).toBe('');
		});

		it('should trim whitespace', () => {
			expect(htmlToPlainText('  <p>  Hello  </p>  ')).toBe('Hello');
		});
	});

	describe('escapeHtml logic', () => {
		function escapeHtml(text: string): string {
			return text
				.replace(/&/g, '&amp;')
				.replace(/</g, '&lt;')
				.replace(/>/g, '&gt;')
				.replace(/"/g, '&quot;')
				.replace(/'/g, '&#39;');
		}

		it('should escape ampersands', () => {
			expect(escapeHtml('Tom & Jerry')).toBe('Tom &amp; Jerry');
		});

		it('should escape angle brackets', () => {
			expect(escapeHtml('<script>alert("xss")</script>')).toBe(
				'&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;'
			);
		});

		it('should escape quotes', () => {
			expect(escapeHtml('"Hello" \'World\'')).toBe('&quot;Hello&quot; &#39;World&#39;');
		});

		it('should handle plain text', () => {
			expect(escapeHtml('Hello World')).toBe('Hello World');
		});

		it('should handle empty string', () => {
			expect(escapeHtml('')).toBe('');
		});
	});

	describe('sanitizeHtml logic', () => {
		function sanitizeHtml(html: string): string {
			return html
				.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
				.replace(/\son\w+\s*=/gi, ' data-removed=');
		}

		it('should remove script tags', () => {
			expect(sanitizeHtml('<p>Hello</p><script>alert("xss")</script>')).toBe('<p>Hello</p>');
		});

		it('should remove inline event handlers', () => {
			const result = sanitizeHtml('<div onclick="alert(1)">Click</div>');
			expect(result).not.toContain('onclick');
			expect(result).toContain('data-removed');
		});

		it('should handle multiple event handlers', () => {
			const result = sanitizeHtml('<img onerror="alert(1)" onload="alert(2)" src="test.jpg">');
			expect(result).not.toContain('onerror');
			expect(result).not.toContain('onload');
		});

		it('should preserve safe HTML', () => {
			expect(sanitizeHtml('<p>Hello <strong>world</strong></p>')).toBe(
				'<p>Hello <strong>world</strong></p>'
			);
		});

		it('should handle empty string', () => {
			expect(sanitizeHtml('')).toBe('');
		});
	});

	describe('import parsing logic', () => {
		// Test the state machine logic used in import parsing
		interface ImportChapter {
			title: string;
			scenes: Array<{ title: string; text: string }>;
		}

		interface ImportParseState {
			chapters: ImportChapter[];
			currentChapter: ImportChapter | null;
			currentScene: { title: string; text: string } | null;
			currentText: string;
		}

		function finalizeScene(state: ImportParseState): void {
			if (state.currentScene && state.currentText.trim()) {
				state.currentScene.text = state.currentText.trim();
				state.currentChapter?.scenes.push(state.currentScene);
			}
		}

		function finalizeChapter(state: ImportParseState): void {
			if (state.currentChapter) {
				state.chapters.push(state.currentChapter);
			}
		}

		function handleChapterHeading(state: ImportParseState, title: string): void {
			finalizeScene(state);
			finalizeChapter(state);
			state.currentChapter = { title: title || 'Untitled Chapter', scenes: [] };
			state.currentScene = null;
			state.currentText = '';
		}

		function handleSceneHeading(state: ImportParseState, title: string): void {
			finalizeScene(state);
			if (!state.currentChapter) {
				state.currentChapter = { title: 'Chapter 1', scenes: [] };
			}
			state.currentScene = { title: title || 'Untitled Scene', text: '' };
			state.currentText = '';
		}

		function handleContent(state: ImportParseState, html: string, isParagraph: boolean): void {
			const separator = isParagraph ? '\n\n' : '\n';
			state.currentText += (state.currentText ? separator : '') + html;
		}

		it('should parse chapter and scene headings', () => {
			const state: ImportParseState = {
				chapters: [],
				currentChapter: null,
				currentScene: null,
				currentText: '',
			};

			handleChapterHeading(state, 'Chapter 1');
			handleSceneHeading(state, 'Scene 1');
			handleContent(state, 'Some text', true);
			finalizeScene(state);
			finalizeChapter(state);

			expect(state.chapters.length).toBe(1);
			expect(state.chapters[0].title).toBe('Chapter 1');
			expect(state.chapters[0].scenes.length).toBe(1);
			expect(state.chapters[0].scenes[0].title).toBe('Scene 1');
			expect(state.chapters[0].scenes[0].text).toBe('Some text');
		});

		it('should handle multiple scenes per chapter', () => {
			const state: ImportParseState = {
				chapters: [],
				currentChapter: null,
				currentScene: null,
				currentText: '',
			};

			handleChapterHeading(state, 'Chapter 1');
			handleSceneHeading(state, 'Scene A');
			handleContent(state, 'Text A', true);
			handleSceneHeading(state, 'Scene B');
			handleContent(state, 'Text B', true);
			finalizeScene(state);
			finalizeChapter(state);

			expect(state.chapters[0].scenes.length).toBe(2);
			expect(state.chapters[0].scenes[0].title).toBe('Scene A');
			expect(state.chapters[0].scenes[1].title).toBe('Scene B');
		});

		it('should handle multiple chapters', () => {
			const state: ImportParseState = {
				chapters: [],
				currentChapter: null,
				currentScene: null,
				currentText: '',
			};

			handleChapterHeading(state, 'Chapter 1');
			handleSceneHeading(state, 'Scene 1');
			handleContent(state, 'Text 1', true);

			handleChapterHeading(state, 'Chapter 2');
			handleSceneHeading(state, 'Scene 2');
			handleContent(state, 'Text 2', true);

			finalizeScene(state);
			finalizeChapter(state);

			expect(state.chapters.length).toBe(2);
			expect(state.chapters[0].title).toBe('Chapter 1');
			expect(state.chapters[1].title).toBe('Chapter 2');
		});

		it('should create default chapter for scene without chapter', () => {
			const state: ImportParseState = {
				chapters: [],
				currentChapter: null,
				currentScene: null,
				currentText: '',
			};

			handleSceneHeading(state, 'Orphan Scene');
			handleContent(state, 'Some text', true);
			finalizeScene(state);
			finalizeChapter(state);

			expect(state.chapters.length).toBe(1);
			expect(state.chapters[0].title).toBe('Chapter 1');
		});

		it('should use default titles for empty headings', () => {
			const state: ImportParseState = {
				chapters: [],
				currentChapter: null,
				currentScene: null,
				currentText: '',
			};

			handleChapterHeading(state, '');
			handleSceneHeading(state, '');
			handleContent(state, 'Content', true);
			finalizeScene(state);
			finalizeChapter(state);

			expect(state.chapters[0].title).toBe('Untitled Chapter');
			expect(state.chapters[0].scenes[0].title).toBe('Untitled Scene');
		});

		it('should handle paragraph vs non-paragraph content', () => {
			const state: ImportParseState = {
				chapters: [],
				currentChapter: null,
				currentScene: null,
				currentText: '',
			};

			handleChapterHeading(state, 'Chapter 1');
			handleSceneHeading(state, 'Scene 1');
			handleContent(state, 'Paragraph 1', true);
			handleContent(state, 'Paragraph 2', true);
			finalizeScene(state);
			finalizeChapter(state);

			expect(state.chapters[0].scenes[0].text).toBe('Paragraph 1\n\nParagraph 2');
		});

		it('should skip scenes with empty text', () => {
			const state: ImportParseState = {
				chapters: [],
				currentChapter: null,
				currentScene: null,
				currentText: '',
			};

			handleChapterHeading(state, 'Chapter 1');
			handleSceneHeading(state, 'Empty Scene');
			// No content added
			handleSceneHeading(state, 'Real Scene');
			handleContent(state, 'Has content', true);
			finalizeScene(state);
			finalizeChapter(state);

			expect(state.chapters[0].scenes.length).toBe(1);
			expect(state.chapters[0].scenes[0].title).toBe('Real Scene');
		});
	});
});
