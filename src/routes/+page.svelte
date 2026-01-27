<script lang="ts">
	import '../app.css';

	import { listen } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { onMount } from 'svelte';

	import Layout from '$lib/components/Layout.svelte';
	import Welcome from '$lib/components/Welcome.svelte';
	import { appState } from '$lib/stores';

	// Show window after first render and listen for file open events
	onMount(() => {
		getCurrentWindow().show();

		// Listen for .cahnon file opened via file association or CLI argument
		const unlistenPromise = listen<string>('open-file', async (event) => {
			try {
				await appState.loadProject(event.payload);
			} catch (e) {
				console.error('Failed to open file from event:', e);
			}
		});

		return () => {
			unlistenPromise.then((fn) => fn());
		};
	});

	// Dynamic window title with unsaved indicator
	let windowTitle = $derived.by(() => {
		if (!appState.project) return 'Cahnon';
		const prefix = appState.hasUnsavedChanges ? '● ' : '';
		return `${prefix}${appState.project.title} — Cahnon`;
	});

	$effect(() => {
		getCurrentWindow().setTitle(windowTitle);
	});
</script>

<svelte:head>
	<title>{windowTitle}</title>
</svelte:head>

{#if appState.project}
	<Layout />
{:else}
	<Welcome />
{/if}
