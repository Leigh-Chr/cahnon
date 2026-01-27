import { AlignmentType, Document, HeadingLevel, Packer, PageBreak, Paragraph, TextRun } from 'docx';
import { saveAs } from 'file-saver';
import mammoth from 'mammoth';
import type { PDFFont, PDFPage } from 'pdf-lib';
import { PDFDocument, rgb, StandardFonts } from 'pdf-lib';

import type { Chapter, Project, Scene } from '$lib/api';

interface ExportOptions {
	includeChapterHeaders: boolean;
	includeSceneHeaders: boolean;
	pageBreakBetweenChapters: boolean;
}

const defaultOptions: ExportOptions = {
	includeChapterHeaders: true,
	includeSceneHeaders: true,
	pageBreakBetweenChapters: true,
};

// ─── HTML Utility Functions ──────────────────────────────────────────────────

/**
 * Converts HTML content to plain text, preserving paragraph breaks.
 * Handles common HTML entities and removes all tags.
 */
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

function escapeHtml(text: string): string {
	return text
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&#39;');
}

// ─── HTML Sanitization ──────────────────────────────────────────────────────

const SANITIZE_ALLOWED_TAGS = new Set([
	'p',
	'br',
	'b',
	'strong',
	'i',
	'em',
	'u',
	's',
	'del',
	'h1',
	'h2',
	'h3',
	'h4',
	'h5',
	'h6',
	'ul',
	'ol',
	'li',
	'blockquote',
	'pre',
	'code',
	'span',
	'div',
	'hr',
	'sub',
	'sup',
	'mark',
	'table',
	'thead',
	'tbody',
	'tr',
	'th',
	'td',
	'caption',
	'a',
]);

const SANITIZE_ALLOWED_ATTRS = new Set(['class', 'style', 'href', 'title', 'colspan', 'rowspan']);

/** Recursively sanitize a single DOM node against the allowlists. */
function sanitizeNode(node: Node): Node | null {
	if (node.nodeType === Node.TEXT_NODE) {
		return node.cloneNode();
	}
	if (node.nodeType !== Node.ELEMENT_NODE) {
		return null;
	}

	const el = node as Element;
	const tagName = el.tagName.toLowerCase();

	if (!SANITIZE_ALLOWED_TAGS.has(tagName)) {
		return sanitizeChildrenAsFragment(el);
	}

	return sanitizeElement(el, tagName);
}

/** For disallowed tags, keep text children but remove the tag itself. */
function sanitizeChildrenAsFragment(el: Element): DocumentFragment {
	const fragment = document.createDocumentFragment();
	for (const child of Array.from(el.childNodes)) {
		const sanitized = sanitizeNode(child);
		if (sanitized) fragment.appendChild(sanitized);
	}
	return fragment;
}

/** Clone an allowed element, copying only safe attributes and recursing into children. */
function sanitizeElement(el: Element, tagName: string): Element {
	const newEl = document.createElement(tagName);
	for (const attr of Array.from(el.attributes)) {
		const name = attr.name.toLowerCase();
		if (!SANITIZE_ALLOWED_ATTRS.has(name)) continue;
		if (name === 'href' && /^\s*javascript:/i.test(attr.value)) continue;
		newEl.setAttribute(attr.name, attr.value);
	}
	for (const child of Array.from(el.childNodes)) {
		const sanitized = sanitizeNode(child);
		if (sanitized) newEl.appendChild(sanitized);
	}
	return newEl;
}

/**
 * Sanitize HTML from TipTap editor content for safe export.
 * Uses a DOM-based allowlist approach rather than fragile regex denylists.
 */
function sanitizeHtml(html: string): string {
	const doc = new DOMParser().parseFromString(html, 'text/html');
	const container = document.createElement('div');
	for (const child of Array.from(doc.body.childNodes)) {
		const sanitized = sanitizeNode(child);
		if (sanitized) container.appendChild(sanitized);
	}
	return container.innerHTML;
}

// ─── DOCX Export ─────────────────────────────────────────────────────────────

function createParagraphsFromHtml(html: string): Paragraph[] {
	const paragraphs: Paragraph[] = [];
	const text = htmlToPlainText(html);
	const parts = text.split(/\n\n+/);

	for (const part of parts) {
		if (part.trim()) {
			paragraphs.push(
				new Paragraph({
					children: [new TextRun(part.trim())],
					spacing: { after: 200, line: 360 },
				})
			);
		}
	}

	return paragraphs;
}

// Helper: Create title page paragraphs
function createTitlePage(project: Project): Paragraph[] {
	const paragraphs: Paragraph[] = [
		new Paragraph({
			children: [new TextRun({ text: project.title, bold: true, size: 56 })],
			heading: HeadingLevel.TITLE,
			alignment: AlignmentType.CENTER,
			spacing: { after: 400 },
		}),
	];

	if (project.author) {
		paragraphs.push(
			new Paragraph({
				children: [new TextRun({ text: `by ${project.author}`, size: 28 })],
				alignment: AlignmentType.CENTER,
				spacing: { after: 800 },
			})
		);
	}

	paragraphs.push(new Paragraph({ children: [new PageBreak()] }));
	return paragraphs;
}

// Helper: Create scene paragraphs
function createSceneParagraphs(scene: Scene, includeHeader: boolean): Paragraph[] {
	const paragraphs: Paragraph[] = [];

	if (includeHeader) {
		paragraphs.push(
			new Paragraph({
				children: [new TextRun({ text: scene.title, bold: true, italics: true })],
				heading: HeadingLevel.HEADING_2,
				spacing: { before: 300, after: 100 },
			})
		);
	}

	paragraphs.push(...createParagraphsFromHtml(scene.text));
	paragraphs.push(new Paragraph({ children: [new TextRun('')], spacing: { after: 200 } }));

	return paragraphs;
}

// Helper: Create chapter paragraphs
function createChapterParagraphs(
	chapter: Chapter,
	scenes: Scene[],
	opts: ExportOptions,
	isFirst: boolean
): Paragraph[] {
	const paragraphs: Paragraph[] = [];

	if (!isFirst && opts.pageBreakBetweenChapters) {
		paragraphs.push(new Paragraph({ children: [new PageBreak()] }));
	}

	if (opts.includeChapterHeaders) {
		paragraphs.push(
			new Paragraph({
				children: [new TextRun({ text: chapter.title, bold: true })],
				heading: HeadingLevel.HEADING_1,
				spacing: { before: 400, after: 200 },
			})
		);
	}

	for (const scene of scenes) {
		paragraphs.push(...createSceneParagraphs(scene, opts.includeSceneHeaders));
	}

	return paragraphs;
}

export async function exportToDocx(
	project: Project,
	chapters: Chapter[],
	scenesByChapter: Map<string, Scene[]>,
	options: Partial<ExportOptions> = {}
): Promise<void> {
	const opts = { ...defaultOptions, ...options };
	const children: Paragraph[] = [...createTitlePage(project)];

	chapters.forEach((chapter, i) => {
		const scenes = scenesByChapter.get(chapter.id) || [];
		children.push(...createChapterParagraphs(chapter, scenes, opts, i === 0));
	});

	const doc = new Document({ sections: [{ properties: {}, children }] });
	const blob = await Packer.toBlob(doc);
	const filename = `${project.title.replace(/[^a-z0-9]/gi, '_')}.docx`;
	saveAs(blob, filename);
}

// ─── HTML Export ─────────────────────────────────────────────────────────────

const HTML_EXPORT_STYLES = `
    body {
      font-family: Georgia, 'Times New Roman', serif;
      max-width: 800px;
      margin: 0 auto;
      padding: 40px 20px;
      line-height: 1.6;
    }
    h1 { text-align: center; margin-bottom: 0.5em; }
    .author { text-align: center; color: #666; margin-bottom: 2em; }
    h2 { margin-top: 2em; border-bottom: 1px solid #ccc; padding-bottom: 0.3em; }
    h3 { font-style: italic; color: #444; }
    p { margin: 1em 0; text-indent: 1.5em; }
    p:first-of-type { text-indent: 0; }`;

// Simple HTML export (for systems without DOCX support)
function exportToHtml(
	project: Project,
	chapters: Chapter[],
	scenesByChapter: Map<string, Scene[]>
): string {
	let html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${escapeHtml(project.title)}</title>
  <style>${HTML_EXPORT_STYLES}
  </style>
</head>
<body>
  <h1>${escapeHtml(project.title)}</h1>
`;

	if (project.author) {
		html += `  <p class="author">by ${escapeHtml(project.author)}</p>\n`;
	}

	for (const chapter of chapters) {
		const scenes = scenesByChapter.get(chapter.id) || [];
		html += `  <h2>${escapeHtml(chapter.title)}</h2>\n`;

		for (const scene of scenes) {
			html += `  <h3>${escapeHtml(scene.title)}</h3>\n`;
			html += `  <div class="scene-content">${sanitizeHtml(scene.text)}</div>\n`;
		}
	}

	html += `</body>\n</html>`;
	return html;
}

export function downloadHtml(
	project: Project,
	chapters: Chapter[],
	scenesByChapter: Map<string, Scene[]>
): void {
	const html = exportToHtml(project, chapters, scenesByChapter);
	const blob = new Blob([html], { type: 'text/html;charset=utf-8' });
	const filename = `${project.title.replace(/[^a-z0-9]/gi, '_')}.html`;
	saveAs(blob, filename);
}

// ─── PDF Export ──────────────────────────────────────────────────────────────

/** Layout constants for PDF generation (US Letter, 1-inch margins). */
const PDF_LAYOUT = {
	pageWidth: 612,
	pageHeight: 792,
	margin: 72,
	get contentWidth() {
		return this.pageWidth - 2 * this.margin;
	},
	lineHeight: 14,
	fontSize: 12,
	titleFontSize: 24,
	h1FontSize: 18,
	h2FontSize: 14,
} as const;

/** Mutable rendering context that tracks the current page and vertical position. */
interface PdfRenderContext {
	pdfDoc: PDFDocument;
	currentPage: PDFPage;
	y: number;
	fonts: {
		regular: PDFFont;
		bold: PDFFont;
		italic: PDFFont;
	};
}

/** Wrap text into lines that fit within maxWidth at the given font and size. */
function pdfWrapText(text: string, font: PDFFont, size: number, maxWidth: number): string[] {
	const words = text.split(' ');
	const lines: string[] = [];
	let currentLine = '';

	for (const word of words) {
		const testLine = currentLine ? `${currentLine} ${word}` : word;
		const width = font.widthOfTextAtSize(testLine, size);

		if (width > maxWidth && currentLine) {
			lines.push(currentLine);
			currentLine = word;
		} else {
			currentLine = testLine;
		}
	}

	if (currentLine) {
		lines.push(currentLine);
	}

	return lines;
}

/** Add a new page and reset the vertical cursor. */
function pdfAddNewPage(ctx: PdfRenderContext): void {
	ctx.currentPage = ctx.pdfDoc.addPage([PDF_LAYOUT.pageWidth, PDF_LAYOUT.pageHeight]);
	ctx.y = PDF_LAYOUT.pageHeight - PDF_LAYOUT.margin;
}

/** Draw wrapped text onto the current page, creating new pages as needed. */
function pdfDrawText(
	ctx: PdfRenderContext,
	text: string,
	font: PDFFont,
	size: number,
	indent: number = 0
): void {
	const lines = pdfWrapText(text, font, size, PDF_LAYOUT.contentWidth - indent);
	for (const line of lines) {
		if (ctx.y < PDF_LAYOUT.margin + PDF_LAYOUT.lineHeight) {
			pdfAddNewPage(ctx);
		}
		ctx.currentPage.drawText(line, {
			x: PDF_LAYOUT.margin + indent,
			y: ctx.y,
			size,
			font,
			color: rgb(0, 0, 0),
		});
		ctx.y -= PDF_LAYOUT.lineHeight * (size / PDF_LAYOUT.fontSize);
	}
}

/** Render the centered title page with project title and optional author. */
function pdfDrawTitlePage(ctx: PdfRenderContext, project: Project): void {
	ctx.y = PDF_LAYOUT.pageHeight / 2;
	const titleWidth = ctx.fonts.bold.widthOfTextAtSize(project.title, PDF_LAYOUT.titleFontSize);
	ctx.currentPage.drawText(project.title, {
		x: (PDF_LAYOUT.pageWidth - titleWidth) / 2,
		y: ctx.y,
		size: PDF_LAYOUT.titleFontSize,
		font: ctx.fonts.bold,
		color: rgb(0, 0, 0),
	});

	if (project.author) {
		ctx.y -= 40;
		const authorText = `by ${project.author}`;
		const authorWidth = ctx.fonts.regular.widthOfTextAtSize(authorText, PDF_LAYOUT.fontSize);
		ctx.currentPage.drawText(authorText, {
			x: (PDF_LAYOUT.pageWidth - authorWidth) / 2,
			y: ctx.y,
			size: PDF_LAYOUT.fontSize,
			font: ctx.fonts.regular,
			color: rgb(0.3, 0.3, 0.3),
		});
	}
}

/** Render a single scene (optional header + content paragraphs). */
function pdfDrawScene(ctx: PdfRenderContext, scene: Scene, includeHeader: boolean): void {
	if (includeHeader) {
		if (ctx.y < PDF_LAYOUT.margin + PDF_LAYOUT.lineHeight * 4) {
			pdfAddNewPage(ctx);
		}
		ctx.y -= PDF_LAYOUT.lineHeight / 2;
		pdfDrawText(ctx, scene.title, ctx.fonts.italic, PDF_LAYOUT.h2FontSize);
		ctx.y -= PDF_LAYOUT.lineHeight / 2;
	}

	const plainText = htmlToPlainText(scene.text);
	const paragraphs = plainText.split(/\n\n+/);

	for (const paragraph of paragraphs) {
		if (paragraph.trim()) {
			pdfDrawText(ctx, paragraph.trim(), ctx.fonts.regular, PDF_LAYOUT.fontSize, 20);
			ctx.y -= PDF_LAYOUT.lineHeight / 2;
		}
	}
	ctx.y -= PDF_LAYOUT.lineHeight;
}

/** Render a chapter (page break + optional header + scenes). */
function pdfDrawChapter(
	ctx: PdfRenderContext,
	chapter: Chapter,
	scenes: Scene[],
	opts: ExportOptions,
	isFirst: boolean
): void {
	if (opts.pageBreakBetweenChapters || isFirst) {
		pdfAddNewPage(ctx);
	}

	if (opts.includeChapterHeaders) {
		pdfDrawText(ctx, chapter.title, ctx.fonts.bold, PDF_LAYOUT.h1FontSize);
		ctx.y -= PDF_LAYOUT.lineHeight;
	}

	for (const scene of scenes) {
		pdfDrawScene(ctx, scene, opts.includeSceneHeaders);
	}
}

export async function exportToPdf(
	project: Project,
	chapters: Chapter[],
	scenesByChapter: Map<string, Scene[]>,
	options: Partial<ExportOptions> = {}
): Promise<void> {
	const opts = { ...defaultOptions, ...options };
	const pdfDoc = await PDFDocument.create();

	const ctx: PdfRenderContext = {
		pdfDoc,
		currentPage: pdfDoc.addPage([PDF_LAYOUT.pageWidth, PDF_LAYOUT.pageHeight]),
		y: PDF_LAYOUT.pageHeight - PDF_LAYOUT.margin,
		fonts: {
			regular: await pdfDoc.embedFont(StandardFonts.TimesRoman),
			bold: await pdfDoc.embedFont(StandardFonts.TimesRomanBold),
			italic: await pdfDoc.embedFont(StandardFonts.TimesRomanItalic),
		},
	};

	pdfDrawTitlePage(ctx, project);

	for (let i = 0; i < chapters.length; i++) {
		const chapter = chapters[i];
		const scenes = scenesByChapter.get(chapter.id) || [];
		pdfDrawChapter(ctx, chapter, scenes, opts, i === 0);
	}

	const pdfBytes = await pdfDoc.save();
	const blob = new Blob([pdfBytes as BlobPart], { type: 'application/pdf' });
	const filename = `${project.title.replace(/[^a-z0-9]/gi, '_')}.pdf`;
	saveAs(blob, filename);
}

// ─── DOCX Import ─────────────────────────────────────────────────────────────

export interface ImportedDocument {
	content: string; // HTML content
	chapters: Array<{
		title: string;
		scenes: Array<{
			title: string;
			text: string;
		}>;
	}>;
}

// Types for import parsing state
type ImportChapter = { title: string; scenes: Array<{ title: string; text: string }> };
type ImportScene = { title: string; text: string };

interface ImportParseState {
	chapters: ImportChapter[];
	currentChapter: ImportChapter | null;
	currentScene: ImportScene | null;
	currentText: string;
}

// Helper: Finalize and save current scene to chapter
function finalizeScene(state: ImportParseState): void {
	if (state.currentScene && state.currentText.trim()) {
		state.currentScene.text = state.currentText.trim();
		state.currentChapter?.scenes.push(state.currentScene);
	}
}

// Helper: Finalize and save current chapter
function finalizeChapter(state: ImportParseState): void {
	if (state.currentChapter) {
		state.chapters.push(state.currentChapter);
	}
}

// Helper: Handle H1 heading (chapter marker)
function handleChapterHeading(state: ImportParseState, title: string): void {
	finalizeScene(state);
	finalizeChapter(state);
	state.currentChapter = { title: title || 'Untitled Chapter', scenes: [] };
	state.currentScene = null;
	state.currentText = '';
}

// Helper: Handle H2/H3 heading (scene marker)
function handleSceneHeading(state: ImportParseState, title: string): void {
	finalizeScene(state);
	if (!state.currentChapter) {
		state.currentChapter = { title: 'Chapter 1', scenes: [] };
	}
	state.currentScene = { title: title || 'Untitled Scene', text: '' };
	state.currentText = '';
}

// Helper: Handle content elements
function handleContent(state: ImportParseState, html: string, isParagraph: boolean): void {
	const separator = isParagraph ? '\n\n' : '\n';
	state.currentText += (state.currentText ? separator : '') + html;
}

// Helper: Create default chapter when no structure found
function createDefaultChapter(html: string): ImportChapter {
	return {
		title: 'Imported Content',
		scenes: [{ title: 'Imported Scene', text: html }],
	};
}

export async function importFromDocx(file: File): Promise<ImportedDocument> {
	const arrayBuffer = await file.arrayBuffer();
	const result = await mammoth.convertToHtml({ arrayBuffer });
	const html = result.value;

	const doc = new DOMParser().parseFromString(html, 'text/html');
	const state: ImportParseState = {
		chapters: [],
		currentChapter: null,
		currentScene: null,
		currentText: '',
	};

	// Process each element in the document
	for (const element of Array.from(doc.body.children)) {
		const tagName = element.tagName.toLowerCase();

		switch (tagName) {
			case 'h1':
				handleChapterHeading(state, element.textContent || '');
				break;
			case 'h2':
			case 'h3':
				handleSceneHeading(state, element.textContent || '');
				break;
			case 'p':
				handleContent(state, element.innerHTML, true);
				break;
			default:
				handleContent(state, element.innerHTML, false);
		}
	}

	// Finalize remaining content
	finalizeScene(state);
	finalizeChapter(state);

	// Use default structure if no chapters found
	if (state.chapters.length === 0) {
		state.chapters.push(createDefaultChapter(html));
	}

	return { content: html, chapters: state.chapters };
}
