import { invoke } from "@tauri-apps/api/core";
import type { Work } from "./types/work";

export function searchQuery() {
	let query = $state("");
	let works: Work[] = $state([]);

	$effect(() => {
		invoke("work_query", { query: query }).then((newWorks) => { works = newWorks as Work[]; }).catch((error) => console.error(error));
	})

	return {
		get query() { return query; },
		set query(newQuery: string) { query = newQuery; },
		get works() { return works; },
	}
}
