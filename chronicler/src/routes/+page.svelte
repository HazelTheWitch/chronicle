<script lang="ts">
	import WorkDisplay from '$lib/components/WorkDisplay.svelte';
	import type { Work } from '$lib/types/work';
	import { invoke } from '@tauri-apps/api/core';
	import { ButtonGroup, Input, InputAddon } from 'flowbite-svelte';
	import debounce from 'just-debounce-it';
	import { Search } from 'lucide-svelte';

	let results: Work[] = $state([]);
	let query: string = $state('');

	$effect(() => updateQuery(query));

	const updateQuery = debounce((q: string) => submitQuery(q), 500);

	async function submitQuery(query: string) {
		results = await invoke('work_query', { query });
	}
</script>

<div class="flex flex-col items-center">
	<ButtonGroup class="m-4 w-3/4">
		<InputAddon>
			<Search />
		</InputAddon>
		<Input placeholder="Search" size="lg" bind:value={query} />
	</ButtonGroup>
	<div class="grid grid-cols-2 gap-4">
		{#each { length: 2 }, x}
			<div class="flex flex-col flex-wrap gap-4 lg:grid lg:grid-cols-2">
				{#each { length: 2 }, y}
					{@const c = x * 2 + y}
					<div class="flex flex-col gap-4">
						{#each results as work, i}
							{#if (i + 4 - c) % 4 == 0}
								<WorkDisplay {work} />
							{/if}
						{/each}
					</div>
				{/each}
			</div>
		{/each}
	</div>
</div>
