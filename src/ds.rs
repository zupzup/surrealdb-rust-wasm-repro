use std::sync::Arc;

use arc_swap::ArcSwap;
use surrealdb::{
    dbs::Session,
    kvs::Datastore,
    rpc::{Data, Method, RpcContext, RpcError, RpcProtocolV1, RpcProtocolV2},
    sql::{Array, Value},
};
use tokio::sync::Semaphore;
#[cfg(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown"
))]
use tokio_with_wasm as tokio;
use uuid::Uuid;

pub struct SurrealWasmEngine(SurrealWasmEngineInner);

pub struct SurrealWasmEngineInner {
    pub kvs: Arc<Datastore>,
    pub lock: Arc<Semaphore>,
    pub session: ArcSwap<Session>,
}

impl SurrealWasmEngine {
    pub async fn execute(&self, method: Method, params: Array) -> Result<Data, RpcError> {
        let res = RpcContext::execute(&self.0, Some(2), method, params)
            .await
            .map_err(|e| e.to_string())
            .unwrap();
        println!("{:?}", res);
        Ok(res)
    }

    pub async fn new() -> SurrealWasmEngine {
        let kvs = Datastore::new("indxdb://data").await.unwrap();
        let session = Session::default().with_rt(true);

        let inner = SurrealWasmEngineInner {
            kvs: Arc::new(kvs),
            session: ArcSwap::new(Arc::new(session)),
            lock: Arc::new(Semaphore::new(1)),
        };
        RpcContext::execute(&inner, Some(2), Method::Use, Array::from(vec!["", "data"]))
            .await
            .unwrap();

        SurrealWasmEngine(inner)
    }
}

impl RpcContext for SurrealWasmEngineInner {
    fn kvs(&self) -> &Datastore {
        &self.kvs
    }

    fn lock(&self) -> Arc<Semaphore> {
        self.lock.clone()
    }

    fn session(&self) -> Arc<Session> {
        self.session.load_full()
    }

    fn set_session(&self, session: Arc<Session>) {
        self.session.store(session);
    }

    fn version_data(&self) -> Data {
        Value::Strand(format!("surrealdb-2.2.1").into()).into()
    }

    const LQ_SUPPORT: bool = true;
    fn handle_live(&self, _lqid: &Uuid) -> impl std::future::Future<Output = ()> + Send {
        async { () }
    }
    fn handle_kill(&self, _lqid: &Uuid) -> impl std::future::Future<Output = ()> + Send {
        async { () }
    }
}

impl RpcProtocolV1 for SurrealWasmEngineInner {}
impl RpcProtocolV2 for SurrealWasmEngineInner {}
