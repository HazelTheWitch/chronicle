import { error } from "@sveltejs/kit";
import { invoke } from "@tauri-apps/api/core";
import type { WorkEdit } from "$lib/types/work";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ params }) => {
    let work_id = parseInt(params.work);

    try {
        console.log(work_id);
        let work: WorkEdit = await invoke("get_work_edit_by_id", { id: work_id });

        return { work }
    } catch (e) {
        error(500, e as string);
    }
}
