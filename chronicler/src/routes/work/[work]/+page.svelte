<script lang="ts">
	import { goto } from '$app/navigation';
	import WorkEdit from '$lib/components/WorkEdit.svelte';
	import type { WorkCreate } from '$lib/types/work';
	import { invoke } from '@tauri-apps/api/core';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	console.log(data);

	function canceled() {
		goto('/');
	}

	async function submitted(work: WorkCreate) {
		let edit = { work_id: data.work.work_id, ...work };

		await invoke('edit_work', { workEdit: edit });

		goto('/');
	}
</script>

<WorkEdit work={data.work} {submitted} {canceled} />
