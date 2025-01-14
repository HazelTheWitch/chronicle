<script lang="ts">
	import type { AuthorCreate } from '$lib/types/author';
	import { Avatar } from 'flowbite-svelte';
	import { open } from '@tauri-apps/plugin-shell';
	import { ExternalLink } from 'lucide-svelte';

	let { author = $bindable() }: { author: AuthorCreate } = $props();
</script>

<div
	class="flex gap-2 rounded-lg border border-gray-300 px-2 py-3 text-gray-900 dark:border-gray-600 dark:text-white"
>
	<div class="flex-1 overflow-scroll pb-3">
		<span class="text-sm text-gray-600 dark:text-gray-400"
			>{author.id !== null ? author.id : 'Author will be created'}</span
		>
		<div class="flex w-full flex-col">
			<span class="font-bold">Aliases</span>
			{#each author.names as name}
				<div>{name}</div>
			{/each}
		</div>
		<div class="flex w-full flex-col">
			<span class="font-bold">URLs</span>
			{#each author.urls as url}
				<div class="flex gap-2">
					<button onclick={() => open(url)}><ExternalLink size={18} /></button><span
						class="underline">{url}</span
					>
				</div>
			{/each}
		</div>
	</div>
</div>
