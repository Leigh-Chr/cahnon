<!--
  Editor formatting toolbar.
  Provides buttons for text formatting via TipTap.
-->
<script lang="ts">
	import type { Editor } from '@tiptap/core';

	interface Props {
		editor: Editor | null;
	}

	let { editor }: Props = $props();

	function isActive(name: string, attrs?: Record<string, unknown>): boolean {
		return editor?.isActive(name, attrs) ?? false;
	}
</script>

{#if editor}
	<div class="editor-toolbar">
		<button
			class="fmt-btn"
			class:active={isActive('bold')}
			onclick={() => editor?.chain().focus().toggleBold().run()}
			title="Bold"
		>
			<strong>B</strong>
		</button>

		<button
			class="fmt-btn"
			class:active={isActive('italic')}
			onclick={() => editor?.chain().focus().toggleItalic().run()}
			title="Italic"
		>
			<em>I</em>
		</button>

		<div class="separator"></div>

		<button
			class="fmt-btn"
			class:active={isActive('heading', { level: 1 })}
			onclick={() => editor?.chain().focus().toggleHeading({ level: 1 }).run()}
			title="Heading 1"
		>
			H1
		</button>

		<button
			class="fmt-btn"
			class:active={isActive('heading', { level: 2 })}
			onclick={() => editor?.chain().focus().toggleHeading({ level: 2 }).run()}
			title="Heading 2"
		>
			H2
		</button>

		<button
			class="fmt-btn"
			class:active={isActive('heading', { level: 3 })}
			onclick={() => editor?.chain().focus().toggleHeading({ level: 3 }).run()}
			title="Heading 3"
		>
			H3
		</button>

		<div class="separator"></div>

		<button
			class="fmt-btn"
			class:active={isActive('bulletList')}
			onclick={() => editor?.chain().focus().toggleBulletList().run()}
			title="Bullet List"
		>
			UL
		</button>

		<button
			class="fmt-btn"
			class:active={isActive('orderedList')}
			onclick={() => editor?.chain().focus().toggleOrderedList().run()}
			title="Ordered List"
		>
			OL
		</button>

		<div class="separator"></div>

		<button
			class="fmt-btn"
			class:active={isActive('blockquote')}
			onclick={() => editor?.chain().focus().toggleBlockquote().run()}
			title="Blockquote"
		>
			BQ
		</button>

		<button
			class="fmt-btn"
			onclick={() => editor?.chain().focus().setHorizontalRule().run()}
			title="Horizontal Rule"
		>
			HR
		</button>

		<button
			class="fmt-btn"
			class:active={isActive('code')}
			onclick={() => editor?.chain().focus().toggleCode().run()}
			title="Inline Code"
		>
			Code
		</button>
	</div>
{/if}

<style>
	.editor-toolbar {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-bottom: 1px solid var(--color-border);
	}

	.fmt-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 28px;
		height: 28px;
		padding: 0 var(--spacing-xs);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		border-radius: var(--border-radius-sm);
		transition: background-color var(--transition-fast);
	}

	.fmt-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.fmt-btn.active {
		background-color: var(--color-bg-tertiary);
		color: var(--color-accent);
	}

	.separator {
		width: 1px;
		height: 16px;
		background-color: var(--color-border);
		margin: 0 var(--spacing-xs);
	}
</style>
