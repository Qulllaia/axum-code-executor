use deadpool_postgres::{Config, ManagerConfig, Object, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

pub async fn connector() ->  Result<Object, Box<dyn std::error::Error>>  {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.port = Some(5432);
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("qulllaia".to_string());
    cfg.dbname = Some("code_executor".to_string());
    
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    Ok(pool.get().await?)
}