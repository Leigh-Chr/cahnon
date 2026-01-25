import { Document, Packer, Paragraph, TextRun, HeadingLevel, PageBreak, AlignmentType } from 'docx';
import { PDFDocument, StandardFonts, rgb } from 'pdf-lib';
import mammoth from 'mammoth';
import { saveAs } from 'file-saver';
import type { Chapter, Scene, Project } from '$lib/api';

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

function _htmlToTextRuns(html: string): TextRun[] {
	// Simple HTML to text conversion
	// Remove tags and convert basic formatting

	// Remove HTML tags but preserve line breaks
	const text = html
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

	// Split into paragraphs
	const paragraphs = text.split(/\n\n+/);

	return paragraphs.map((p) => new TextRun({ text: p.trim(), break: 2 }));
}

function createParagraphsFromHtml(html: string): Paragraph[] {
	const paragraphs: Paragraph[] = [];

	// Simple HTML to paragraphs conversion
	const text = html
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

// Simple HTML export (for systems without DOCX support)
export function exportToHtml(
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
  <style>
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
    p:first-of-type { text-indent: 0; }
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
			html += `  <div class="scene-content">${scene.text}</div>\n`;
		}
	}

	html += `</body>\n</html>`;
	return html;
}

function escapeHtml(text: string): string {
	return text
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&#39;');
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

// PDF Export
export async function exportToPdf(
	project: Project,
	chapters: Chapter[],
	scenesByChapter: Map<string, Scene[]>,
	options: Partial<ExportOptions> = {}
): Promise<void> {
	const opts = { ...defaultOptions, ...options };
	const pdfDoc = await PDFDocument.create();
	const timesRomanFont = await pdfDoc.embedFont(StandardFonts.TimesRoman);
	const timesRomanBoldFont = await pdfDoc.embedFont(StandardFonts.TimesRomanBold);
	const timesRomanItalicFont = await pdfDoc.embedFont(StandardFonts.TimesRomanItalic);

	const pageWidth = 612; // Letter size
	const pageHeight = 792;
	const margin = 72; // 1 inch margin
	const contentWidth = pageWidth - 2 * margin;
	const lineHeight = 14;
	const fontSize = 12;
	const titleFontSize = 24;
	const h1FontSize = 18;
	const h2FontSize = 14;

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

	function wrapText(
		text: string,
		font: typeof timesRomanFont,
		size: number,
		maxWidth: number
	): string[] {
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

	let currentPage = pdfDoc.addPage([pageWidth, pageHeight]);
	let y = pageHeight - margin;

	function addNewPage() {
		currentPage = pdfDoc.addPage([pageWidth, pageHeight]);
		y = pageHeight - margin;
	}

	function drawText(text: string, font: typeof timesRomanFont, size: number, indent: number = 0) {
		const lines = wrapText(text, font, size, contentWidth - indent);
		for (const line of lines) {
			if (y < margin + lineHeight) {
				addNewPage();
			}
			currentPage.drawText(line, {
				x: margin + indent,
				y,
				size,
				font,
				color: rgb(0, 0, 0),
			});
			y -= lineHeight * (size / fontSize);
		}
	}

	// Title page
	y = pageHeight / 2;
	const titleWidth = timesRomanBoldFont.widthOfTextAtSize(project.title, titleFontSize);
	currentPage.drawText(project.title, {
		x: (pageWidth - titleWidth) / 2,
		y,
		size: titleFontSize,
		font: timesRomanBoldFont,
		color: rgb(0, 0, 0),
	});

	if (project.author) {
		y -= 40;
		const authorText = `by ${project.author}`;
		const authorWidth = timesRomanFont.widthOfTextAtSize(authorText, fontSize);
		currentPage.drawText(authorText, {
			x: (pageWidth - authorWidth) / 2,
			y,
			size: fontSize,
			font: timesRomanFont,
			color: rgb(0.3, 0.3, 0.3),
		});
	}

	// Content
	for (let i = 0; i < chapters.length; i++) {
		const chapter = chapters[i];
		const scenes = scenesByChapter.get(chapter.id) || [];

		// New page for each chapter
		if (opts.pageBreakBetweenChapters || i === 0) {
			addNewPage();
		}

		// Chapter header
		if (opts.includeChapterHeaders) {
			drawText(chapter.title, timesRomanBoldFont, h1FontSize);
			y -= lineHeight;
		}

		// Scenes
		for (const scene of scenes) {
			if (opts.includeSceneHeaders) {
				if (y < margin + lineHeight * 4) {
					addNewPage();
				}
				y -= lineHeight / 2;
				drawText(scene.title, timesRomanItalicFont, h2FontSize);
				y -= lineHeight / 2;
			}

			// Scene content
			const plainText = htmlToPlainText(scene.text);
			const paragraphs = plainText.split(/\n\n+/);

			for (const paragraph of paragraphs) {
				if (paragraph.trim()) {
					drawText(paragraph.trim(), timesRomanFont, fontSize, 20); // First line indent
					y -= lineHeight / 2;
				}
			}
			y -= lineHeight;
		}
	}

	const pdfBytes = await pdfDoc.save();
	const blob = new Blob([pdfBytes as BlobPart], { type: 'application/pdf' });
	const filename = `${project.title.replace(/[^a-z0-9]/gi, '_')}.pdf`;
	saveAs(blob, filename);
}

// DOCX Import
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
