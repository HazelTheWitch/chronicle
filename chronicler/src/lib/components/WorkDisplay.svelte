<script lang="ts">
	import type { Work } from '$lib/types/work';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { appDataDir, join } from '@tauri-apps/api/path';

	let { work }: { work: Work } = $props();

	async function getWorkUrl(path: string): Promise<string | null> {
		let dataDirPath = await appDataDir();
		let filePath = await join(dataDirPath, `works/${path}`);

		return convertFileSrc(filePath);
	}

	let url = $derived(getWorkUrl(work.path));
</script>

{#await url}
	<div class="h-auto max-w-full rounded-lg bg-gray-200 dark:bg-gray-600"></div>
{:then url}
	<a class="h-auto max-w-full rounded-lg" href={`/work/${work.work_id}`}>
		<img class="rounded-lg" src={url} alt={work.caption} />
	</a>
{/await}
