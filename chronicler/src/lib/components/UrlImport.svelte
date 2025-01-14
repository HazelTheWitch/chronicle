<script lang="ts">
	import type { WorkCreate } from '$lib/types/work';
	import { Button, ButtonGroup, Helper, Input, InputAddon } from 'flowbite-svelte';
	import { Download } from 'lucide-svelte';
	import { invoke } from '@tauri-apps/api/core';

	let { url = '' } = $props();

	export const results: { works: WorkCreate[]; error: string | null } = $state({
		works: [],
		error: null
	});

	async function updateWorks() {
		try {
			results.works = await invoke('import_work_create', { url });
			results.error = null;
		} catch (e) {
			results.error = e as string;
			results.works = [];
		}
	}

	export function popWork(): WorkCreate {
		return results.works.splice(0, 1)[0];
	}

	export function clear() {
		url = '';
	}
</script>

<div class="m-4 w-3/4">
	<ButtonGroup class="w-full">
		<InputAddon>
			<Download />
		</InputAddon>
		<Input size="lg" bind:value={url} />
		<Button color="primary" on:click={updateWorks}>Import</Button>
	</ButtonGroup>
	{#if results.error !== null}
		<Helper class="mt-2 text-sm">{results.error}</Helper>
	{/if}
</div>
