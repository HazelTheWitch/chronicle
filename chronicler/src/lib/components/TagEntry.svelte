<script lang="ts">
	import type { Tag } from '$lib/types/tag';
	import { Helper, Input } from 'flowbite-svelte';
	import TagDisplay from './TagDisplay.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { X } from 'lucide-svelte';
	let currentTag: string = $state('');
	let { disabled = false, tags } = $props();
	let error: string | null = $state(null);

	async function submitTag() {
		if (currentTag == '') {
			error = null;
			return;
		}

		try {
			let newTag: Tag = await invoke('parse_tag', { tag: currentTag.trim() });

			for (let other of tags) {
				if (
					newTag.name == other.name &&
					(newTag.discriminator === null || other.discriminator === null)
				) {
					throw `two tags are conflicting: ${newTag.name}`;
				}
			}

			tags.push(newTag);
			error = null;
			currentTag = '';
		} catch (e) {
			error = e as string;
		}
	}

	export function getTags(): Tag[] {
		return tags;
	}

	function remove(i: number) {
		tags.splice(i, 1);
	}
</script>

<div>
	<form class="w-full" onsubmit={submitTag}>
		<Input
			bind:value={currentTag}
			{disabled}
			placeholder="Tags"
			on:change={() => (error = null)}
			color={error === null ? 'base' : 'red'}><span slot="left">#</span></Input
		>
		<Helper class="min-h-4" color="red">{error === null ? '' : error}</Helper>
	</form>
	<div
		class="mt-1 flex h-72 flex-col gap-2 overflow-scroll rounded-lg border border-gray-300 dark:border-gray-600"
	>
		{#each tags as tag, i}
			<div class="h-md flex w-full justify-between p-2 hover:bg-gray-200 hover:dark:bg-gray-600">
				<TagDisplay {tag} />
				<button onclick={() => remove(i)}><X class="text-gray-800 dark:text-white" /></button>
			</div>
		{/each}
	</div>
</div>
