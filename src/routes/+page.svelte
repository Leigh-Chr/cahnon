<script lang="ts">
	import '../app.css';
	import { appState } from '$lib/stores';
	import Welcome from '$lib/components/Welcome.svelte';
	import Layout from '$lib/components/Layout.svelte';
</script>

<svelte:head>
	<title>{appState.project ? `${appState.project.title} - Cahnon` : 'Cahnon'}</title>
</svelte:head>

{#if appState.project}
	<Layout />
{:else}
	<Welcome />
{/if}
