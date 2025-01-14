<script lang="ts">
	import WorkEdit from '$lib/components/WorkEdit.svelte';
	import UrlImport from '$lib/components/UrlImport.svelte';
	import type { WorkCreate } from '$lib/types/work';
	import { invoke } from '@tauri-apps/api/core';
	let urlImport: UrlImport | undefined = $state(undefined);

	let work = $derived.by(() => {
		if (urlImport === undefined) {
			return null;
		}

		let works = urlImport.results.works;

		if (works.length == 0) {
			return null;
		}

		return works[0];
	});

	async function submitted(work: WorkCreate) {
		await invoke('create_work', { workCreate: work });
		urlImport?.popWork();
	}

	function canceled() {
		let works = urlImport?.results.works;

		works?.splice(0, 1);
	}
</script>

<div class="flex flex-col items-center">
	<UrlImport bind:this={urlImport} />
	<WorkEdit {work} {submitted} {canceled} />
</div>
