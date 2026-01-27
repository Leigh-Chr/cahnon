/**
 * TipTap Mark extension for rendering annotations inline in the editor.
 *
 * Annotations are highlighted with colors based on their type:
 * - comment (yellow)
 * - question (blue)
 * - todo (orange)
 * - research (violet)
 * - revision (green)
 */
import { Mark, mergeAttributes } from '@tiptap/core';

export interface AnnotationMarkOptions {
	HTMLAttributes: Record<string, unknown>;
}

declare module '@tiptap/core' {
	interface Commands<ReturnType> {
		annotationMark: {
			setAnnotationMark: (attributes: {
				annotationId: string;
				annotationType: string;
			}) => ReturnType;
			unsetAnnotationMark: () => ReturnType;
		};
	}
}

export const AnnotationMark = Mark.create<AnnotationMarkOptions>({
	name: 'annotationMark',

	addOptions() {
		return {
			HTMLAttributes: {},
		};
	},

	addAttributes() {
		return {
			annotationId: {
				default: null,
				parseHTML: (element) => element.getAttribute('data-annotation-id'),
				renderHTML: (attributes) => ({
					'data-annotation-id': attributes.annotationId as string,
				}),
			},
			annotationType: {
				default: 'comment',
				parseHTML: (element) => element.getAttribute('data-annotation-type'),
				renderHTML: (attributes) => ({
					'data-annotation-type': attributes.annotationType as string,
				}),
			},
		};
	},

	parseHTML() {
		return [
			{
				tag: 'mark[data-annotation-id]',
			},
		];
	},

	renderHTML({ HTMLAttributes }) {
		return [
			'mark',
			mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, {
				class: `annotation-highlight annotation-${(HTMLAttributes['data-annotation-type'] as string) || 'comment'}`,
			}),
			0,
		];
	},

	addCommands() {
		return {
			setAnnotationMark:
				(attributes) =>
				({ commands }) => {
					return commands.setMark(this.name, attributes);
				},
			unsetAnnotationMark:
				() =>
				({ commands }) => {
					return commands.unsetMark(this.name);
				},
		};
	},
});

export default AnnotationMark;
