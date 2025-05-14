use js_sys::Promise;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    sync::{Arc, OnceLock},
};
use surrealdb::{
    engine::any::Any,
    rpc::{Data, Method},
    sql::{Array, Object, Value},
    Surreal,
};
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot::{self},
    Semaphore,
};
#[cfg(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown"
))]
use tokio_with_wasm as tokio;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

mod ds;

const POOL_SIZE: usize = 10;

pub struct SurrealPool {
    // conns: VecDeque<Surreal<Any>>,
    conns: VecDeque<ds::SurrealWasmEngine>,
}

impl SurrealPool {
    pub async fn new() -> Self {
        let mut conns = VecDeque::with_capacity(POOL_SIZE);
        for _ in 0..POOL_SIZE {
            let db = get_new_db().await;
            conns.push_back(db);
        }
        Self { conns }
    }

    // pub fn take(&mut self) -> Surreal<Any> {
    //     let conn = self.conns.pop_front().expect("DB pool exhausted");
    //     conn
    // }

    // pub fn give_back(&mut self, conn: Surreal<Any>) {
    //     self.conns.push_back(conn);
    // }

    pub fn take(&mut self) -> ds::SurrealWasmEngine {
        let conn = self.conns.pop_front().expect("DB pool exhausted");
        conn
    }

    pub fn give_back(&mut self, conn: ds::SurrealWasmEngine) {
        self.conns.push_back(conn);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub msg: String,
}

pub async fn flush_js_microtasks() {
    let _ = JsFuture::from(Promise::resolve(&JsValue::undefined())).await;
}

pub enum DbTask {
    CreateMessage {
        msg: Message,
        respond_to: oneshot::Sender<Result<Option<Data>, surrealdb::Error>>,
    },
    GetMessages {
        respond_to: oneshot::Sender<Result<Vec<Data>, surrealdb::Error>>,
    },
}

// pub fn start_db_worker(db: Surreal<Any>) -> mpsc::Sender<DbTask> {
pub fn start_db_worker(mut pool: SurrealPool) -> mpsc::Sender<DbTask> {
    let (tx, mut rx) = mpsc::channel::<DbTask>(1);

    tokio::task::spawn(async move {
        while let Some(task) = rx.recv().await {
            // flush_js_microtasks().await;
            let db = pool.take();

            // log::info!("flush processing...");
            // match task {
            //     DbTask::CreateMessage { msg, respond_to } => {
            //         let result: Result<Option<Message>, surrealdb::Error> =
            //             db.create("msg").content(msg).await;
            //         // flush_js_microtasks().await;
            //         let _ = respond_to.send(result);
            //     }
            //     DbTask::GetMessages { respond_to } => {
            //         let result = db.select("msg").await;
            //         // flush_js_microtasks().await;
            //         let _ = respond_to.send(result);
            //     }
            // }
            match task {
                DbTask::CreateMessage { msg, respond_to } => {
                    let mut obj = HashMap::new();
                    obj.insert("msg", Value::from("this is a msg"));
                    let result = db
                        .execute(
                            Method::Create,
                            Array::from(vec![Value::from("msg"), Value::Object(Object::from(obj))]),
                        )
                        .await
                        .unwrap();
                    // flush_js_microtasks().await;
                    let _ = respond_to.send(Ok(Some(result)));
                }
                DbTask::GetMessages { respond_to } => {
                    let result = db
                        .execute(Method::Select, Array::from(vec!["msg"]))
                        .await
                        .unwrap();
                    // flush_js_microtasks().await;
                    let _ = respond_to.send(Ok(vec![result]));
                }
            }
            pool.give_back(db);
            // flush_js_microtasks().await;
            // log::info!("processed...");
        }
    });

    tx
}

pub static GLOBAL_DB_TX: OnceLock<Sender<DbTask>> = OnceLock::new();

thread_local! {
    // static SURREAL_DB: RefCell<Option<&'static Surreal<Any>>> = const { RefCell::new(None) };
    static DB: RefCell<Option<&'static ds::SurrealWasmEngine>> = const { RefCell::new(None) };
}

pub static GLOBAL_DB_ACCESS_GUARD: Lazy<Arc<Semaphore>> = Lazy::new(|| Arc::new(Semaphore::new(1)));

// async fn get_db_ref() -> &'static Surreal<Any> {
async fn get_db_ref() -> &'static ds::SurrealWasmEngine {
    return DB.with(|db| db.borrow().expect("DB is not initialized"));
    // return SURREAL_DB.with(|db| db.borrow().expect("DB is not initialized"));
}

async fn get_new_db_old() -> Surreal<Any> {
    let db = surrealdb::engine::any::connect("indxdb://data")
        .await
        .unwrap();
    db.use_ns("").use_db("data").await.unwrap();
    return db;
}

async fn get_new_db() -> ds::SurrealWasmEngine {
    let engine = ds::SurrealWasmEngine::new().await;
    engine
}

#[wasm_bindgen]
pub async fn get_msg_new_db() {
    let engine = ds::SurrealWasmEngine::new().await; // works
                                                     // let engine = get_db_ref().await; // doesn't work at all
    let mut obj = HashMap::new();
    obj.insert("msg", Value::from("this is a msg"));
    engine
        .execute(
            Method::Create,
            Array::from(vec![Value::from("msg"), Value::Object(Object::from(obj))]),
        )
        .await
        .unwrap();
    // let data = engine
    //     .execute(Method::Select, Array::from(vec!["msg"]))
    //     .await
    //     .unwrap();
    if let Ok(v) = engine.select::<Vec<Message>>("msg", None).await {
        // log::info!("returned a value! {v:?}");
        // log::info!("ret");
    }
    // 00fg7fm50o9z1pxkfq1k
    if let Ok(single) = engine
        .select::<Option<Message>>("msg", Some("00fg7fm50o9z1pxkfq1k".into()))
        .await
    {
        if let Some(msg) = single {
            // log::info!("returned a value! {msg:?}");
        }
    }
    // log::info!("{:?}", data);
}

async fn get_msg() {
    // log::info!("getting lock get...");
    // log::info!("get...");
    // let _permit = GLOBAL_DB_ACCESS_GUARD.acquire().await.unwrap();
    // let tx = GLOBAL_DB_TX
    //     .get()
    //     .expect("DB queue not initialized")
    //     .clone();

    // let (res_tx, res_rx) = tokio::sync::oneshot::channel();
    // tx.send(DbTask::GetMessages { respond_to: res_tx })
    //     .await
    //     .unwrap();
    // let _messages = res_rx.await.unwrap().unwrap();
    let _msgs: Vec<Message> = get_new_db_old().await.select("msg").await.unwrap();
    // log::info!("releasing lock get...");
    // log::info!("get done...");
}

async fn add_msg() {
    // log::info!("add...");
    // log::info!("getting lock add...");
    // let _permit = GLOBAL_DB_ACCESS_GUARD.acquire().await.unwrap();
    // let tx = GLOBAL_DB_TX
    //     .get()
    //     .expect("DB queue not initialized")
    //     .clone();

    // let (res_tx, res_rx) = tokio::sync::oneshot::channel();
    // tx.send(DbTask::CreateMessage {
    //     msg: Message {
    //         msg: "this is a msg".to_owned(),
    //     },
    //     respond_to: res_tx,
    // })
    // .await
    // .unwrap();
    // let _msg = res_rx.await.unwrap().unwrap();
    // let _messages = res_rx.await.unwrap().unwrap();
    let _: Option<Message> = get_new_db_old()
        .await
        .create("msg")
        .content(Message {
            msg: "this is a msg".to_owned(),
        })
        .await
        .unwrap();
    // log::info!("releasing lock add...");
    // log::info!("add done...");
}

#[wasm_bindgen]
pub async fn get_msg_db_ref() {
    // works
    // let db = get_db_ref().await;
    get_msg().await;
    add_msg().await;
}

#[wasm_bindgen]
pub async fn initialize() {
    console_log::init_with_level(log::Level::Info).unwrap();

    // let db = get_new_db().await;
    // SURREAL_DB.with(|surreal_db| {
    //     let mut db_ref = surreal_db.borrow_mut();
    //     if db_ref.is_none() {
    //         let leaked: &'static Surreal<Any> = Box::leak(Box::new(db));
    //         *db_ref = Some(leaked);
    //     }
    // });
    let db = ds::SurrealWasmEngine::new().await;
    DB.with(|surreal_db| {
        let mut db_ref = surreal_db.borrow_mut();
        if db_ref.is_none() {
            let leaked: &'static ds::SurrealWasmEngine = Box::leak(Box::new(db));
            *db_ref = Some(leaked);
        }
    });
    let pool = SurrealPool::new().await;
    // let tx = start_db_worker(get_new_db().await);
    let tx = start_db_worker(pool);
    if let Err(_) = GLOBAL_DB_TX.set(tx) {
        panic!("global tx set");
    }
}
