<script lang="ts">
	import type { WorkCreate } from '$lib/types/work';
	import { appDataDir, join } from '@tauri-apps/api/path';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import {
		Button,
		ButtonGroup,
		Checkbox,
		FloatingLabelInput,
		Spinner,
		Textarea
	} from 'flowbite-svelte';
	import TagEntry from './TagEntry.svelte';
	import AuthorEdit from './AuthorEdit.svelte';
	import type { Author } from '$lib/types/author';

	let {
		work,
		submitted,
		canceled
	}: { work: WorkCreate | null; submitted: (work: WorkCreate) => void; canceled: () => void } =
		$props();

	let disabled = $derived(work === null);

	async function getWorkUrl(path: string | undefined): Promise<string | null> {
		if (path === undefined) {
			return null;
		}

		let dataDirPath = await appDataDir();
		let filePath = await join(dataDirPath, `works/${path}`);

		return convertFileSrc(filePath);
	}

	function propertyGetter(field: 'title' | 'caption' | 'url') {
		return {
			get field() {
				if (work === null) {
					return '';
				}

				if (work[field] === null) {
					return '';
				}

				return work[field];
			},
			set field(value: string) {
				if (work === null) {
					return;
				}

				if (value.length === 0) {
					work[field] = '';
				}

				work[field] = value;
			}
		};
	}

	let title = $state(propertyGetter('title'));
	let url = $state(propertyGetter('url'));
	let caption = $state(propertyGetter('caption'));

	let author = $state({
		get author() {
			if (work === null) {
				return null;
			}

			return work.author;
		},
		set author(author: Author | null) {
			if (work === null) {
				return;
			}

			work.author = author;
		}
	});

	let altAuthor: Author | null = $state(null);

	function toggleAuthor() {
		if (work === null) {
			return;
		}

		if (altAuthor === null && author.author === null) {
			altAuthor = { id: null, urls: [], names: [] };
		}

		let temp = altAuthor;
		altAuthor = author.author;
		author.author = temp;
	}

	let tagEntry: TagEntry;

	const workUrl = $derived(getWorkUrl(work?.path));
</script>

<div class="m-4 flex w-full justify-center gap-6">
	{#if workUrl !== null}
		{#await workUrl}
			<div
				class="flex aspect-square w-[40vw] items-center justify-center rounded-lg bg-gray-200 dark:bg-gray-600"
			>
				<Spinner size={8} />
			</div>
		{:then url}
			{#if url !== null && url.length > 0}
				<img src={url} alt="" class="w-[40vw] rounded-lg object-contain" />
			{:else}
				<div class="aspect-square w-[40vw] rounded-lg bg-gray-200 dark:bg-gray-600"></div>
			{/if}
		{/await}
	{:else}
		<div class="aspect-square w-[40vw] rounded-lg bg-gray-200 dark:bg-gray-600"></div>
	{/if}
	<div class="flex w-96 flex-col items-stretch gap-6">
		<FloatingLabelInput style="standard" type="text" {disabled} bind:value={title.field}
			>Title</FloatingLabelInput
		>
		<FloatingLabelInput style="standard" type="text" {disabled} bind:value={url.field}
			>Url</FloatingLabelInput
		>
		<Textarea placeholder="Caption" rows={4} {disabled} bind:value={caption.field} />
		<TagEntry {disabled} bind:this={tagEntry} tags={[...work?.tags]} />
		{#if disabled}
			<Checkbox checked={author.author !== null} color="primary" disabled
				>This work has an author</Checkbox
			>
		{:else}
			<Checkbox checked={author.author !== null} color="primary" on:click={toggleAuthor}
				>This work has an author</Checkbox
			>
		{/if}
		{#if author.author !== null}
			<AuthorEdit bind:author={author.author} />
		{/if}
		<ButtonGroup>
			<Button color="dark" {disabled} onclick={() => canceled()}>Cancel</Button>
			<Button
				color="primary"
				{disabled}
				onclick={() => {
					if (work !== null) {
						work.tags = tagEntry.getTags();
						submitted(work);
					}
				}}>Submit</Button
			>
		</ButtonGroup>
	</div>
</div>
