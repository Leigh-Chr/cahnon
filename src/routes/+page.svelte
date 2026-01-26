<script lang="ts">
	import '../app.css';

	import Layout from '$lib/components/Layout.svelte';
	import Welcome from '$lib/components/Welcome.svelte';
	import { appState } from '$lib/stores';
</script>

<svelte:head>
	<title>{appState.project ? `${appState.project.title} - Cahnon` : 'Cahnon'}</title>
</svelte:head>

{#if appState.project}
	<Layout />
{:else}
	<Welcome />
{/if}
