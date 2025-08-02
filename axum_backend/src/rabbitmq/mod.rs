use lapin::{options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions, QueueDeclareOptions}, types::FieldTable};
use tokio::{sync::MutexGuard, task::JoinHandle, time::Timeout};

use crate::{types::VerifyToken, Connections};
use futures_lite::stream::StreamExt;

pub struct EmailConsumer;

impl EmailConsumer {
    pub async fn get_verification_ping(connection: &MutexGuard<'_, Connections>, q_name: &String, verify_token: String) -> Timeout<JoinHandle<bool>> {
        connection.rabbitmq_channel_consumer.queue_declare(
            q_name.as_str(), 
            QueueDeclareOptions {
                        durable: true,
                        auto_delete: false,
                        ..Default::default()
                    }, 
            FieldTable::default())
            .await
            .expect("Queue crate failed");

        let mut consumer = connection.rabbitmq_channel_consumer.basic_consume(
            q_name.as_str(), 
            "verification_consumer", 
            BasicConsumeOptions::default(), 
            FieldTable::default())
            .await
            .expect("Consumer creation failed");

        
        let timeout_task = tokio::time::timeout(
            std::time::Duration::from_secs(20),
            tokio::spawn(async move {
                let mut result = false;
                
                while let Some(delivery) = consumer.next().await {
                    match delivery {
                        Ok(delivery) => {
                            let data: VerifyToken = serde_json::from_slice(&delivery.data)
                                .expect("Failed to parse message");
                            
                            if &data.verify_token == &verify_token {
                                delivery.ack(BasicAckOptions::default())
                                    .await
                                    .expect("Failed to ack message");
                                result = true;
                                // println!("Finded");
                                break;
                            } else {
                                delivery.nack(BasicNackOptions {
                                    multiple: false,
                                    requeue: true
                                })
                                .await
                                .expect("Failed to nack message");
                            }
                        },
                        Err(e) => {
                            eprintln!("Error receiving message: {}", e);
                            break;
                        }
                    }
                }
                return result;
            })
        );
        return timeout_task;
    }

}