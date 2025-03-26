use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use surrealdb::{engine::any::Any, Surreal};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub msg: String,
}

thread_local! {
    static SURREAL_DB: RefCell<Option<&'static Surreal<Any>>> = const { RefCell::new(None) };
}

async fn get_db_ref() -> &'static Surreal<Any> {
    return SURREAL_DB.with(|db| db.borrow().expect("DB is not initialized"));
}

async fn get_new_db() -> Surreal<Any> {
    let db = surrealdb::engine::any::connect("indxdb://data")
        .await
        .unwrap();
    db.use_ns("").use_db("data").await.unwrap();
    return db;
}

#[wasm_bindgen]
pub async fn get_msg_new_db() {
    let db = get_new_db().await;
    get_msg(&db).await;
}

async fn get_msg(db: &Surreal<Any>) {
    let _: Option<Message> = db
        .create("msg")
        .content(Message {
            msg: "this is a msg".to_owned(),
        })
        .await
        .unwrap();
    let _msgs: Vec<Message> = db.select("msg").await.unwrap();
}

#[wasm_bindgen]
pub async fn get_msg_db_ref() {
    let db = get_db_ref().await;
    get_msg(&db).await;
}

#[wasm_bindgen]
pub async fn initialize() {
    console_log::init_with_level(log::Level::Info).unwrap();

    let db = get_new_db().await;
    SURREAL_DB.with(|surreal_db| {
        let mut db_ref = surreal_db.borrow_mut();
        if db_ref.is_none() {
            let leaked: &'static Surreal<Any> = Box::leak(Box::new(db));
            *db_ref = Some(leaked);
        }
    });
}
