<script lang="ts">
	import { searchQuery } from '$lib/search.svelte';

	let query = searchQuery();
</script>

<input type="text" bind:value={query.query} />

{query.works.length}

{#each query.works as work}
	<div>
		{work.path}
	</div>
{/each}
