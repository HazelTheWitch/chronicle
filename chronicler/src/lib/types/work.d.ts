import { Author } from "./author";
import { Tag } from "./tag";

export interface Work {
	path: string,
	work_id: number,
	size: number,
	title: string | null,
	author_id: number | null,
	caption: string | null,
	url: string | null,
	hash: number,
}

export interface WorkCreate {
	path: string,
	title: string | null,
	author: Author | null,
	caption: string | null,
	url: string | null,
	tags: Tag[],
}

export interface WorkEdit extends WorkCreate {
	work_id: number,
}
