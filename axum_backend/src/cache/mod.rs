use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use tokio::sync::MutexGuard;

use crate::Connections;
pub struct Cache {}

impl Cache {
    pub async fn set_data_by_field(connection: &mut MutexGuard<'_, Connections>, field: &String, data: &String) -> () {
        let _ : () = connection.redis.set(field,data).await.unwrap();
    }

    pub async fn get_data_by_field(connection: &mut MutexGuard<'_, Connections>, field: &String) -> String {
        return connection.redis.get(field).await.unwrap();
    }

    pub async fn check_filed_existance(connection: &mut MutexGuard<'_, Connections>, field: &String) -> bool {
        return connection.redis.exists(field).await.unwrap();
    }
}