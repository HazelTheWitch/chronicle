<script lang="ts">
	import type { Tag } from '$lib/types/tag';

	let { tag = $bindable() }: { tag: Tag } = $props();
</script>

<span class="text-gray-900 dark:text-white">
	{#if tag.discriminator !== null}
		<span>{tag.name}</span><span class="text-gray-600 dark:text-gray-400">#{tag.discriminator}</span
		>
	{:else}
		{tag.name}
	{/if}
</span>
