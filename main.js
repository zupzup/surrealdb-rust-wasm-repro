import init, {
    get_msg_new_db,
    get_msg_db_ref,
    initialize
} from '../pkg/index.js';

async function run() {
    await init();
    await initialize();
}

await run();

document.getElementById("getmsgnew").addEventListener("click", async () => {
    await get_msgs(true);
});

document.getElementById("getmsgref").addEventListener("click", async () => {
    await get_msgs(false);
});

async function get_msgs(use_new_db) {
    let task_1 = get_msg("first", use_new_db);
    let task_2 = get_msg("second", use_new_db);
    let task_3 = get_msg("third", use_new_db);
    let task_4 = get_msg("fourth", use_new_db);
    let task_5 = get_msg("fifth", use_new_db);
    await Promise.all([task_1, task_2, task_3, task_4, task_5]);
}

async function get_msg(t, use_new_db) {
    for (let i = 0; i <= 50; i++) {
        console.log(t, i);
        if (use_new_db) {
            await get_msg_new_db();
        } else {
            await get_msg_db_ref();
        }
    }
}
