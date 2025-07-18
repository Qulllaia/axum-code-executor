use deadpool_postgres::{Config, ManagerConfig, Object, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;
use dotenv::dotenv;
use std::env;

pub async fn connector() ->  Result<Object, Box<dyn std::error::Error>>  {
    dotenv().ok();

    let host = env::var("HOST");
    let port = env::var("PORT");
    let user = env::var("USER"); 
    let password = env::var("PASSWORD");
    let dbname = env::var("DBNAME");

    match (host, port, user, password, dbname) {
        (Ok(host), Ok(port), Ok(user), Ok(password), Ok(dbname)) => {
            let mut cfg = Config::new();

            let mut cfg = Config::new();
            cfg.host = Some(host);
            cfg.port = Some(port.parse::<u16>().unwrap());
            cfg.user = Some(user);
            cfg.password = Some(password);
            cfg.dbname = Some(dbname);
            cfg.manager = Some(ManagerConfig {
                recycling_method: RecyclingMethod::Fast,
            });
            
            let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
        
            Ok(pool.get().await?)
    
        },
        _ => panic!(".env ERROR")
    }

}