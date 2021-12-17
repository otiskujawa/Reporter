use crate::auth_manager::AuthManager;
use crate::data_collector::DataCollector;
use crate::util::arcmutex;
use anyhow::Result;
use parking_lot::Mutex;
use serde_json::json;
use std::net::TcpStream;
use std::sync::Arc;
use websocket::sync::Client;
use websocket::{ClientBuilder, Message};
extern crate machine_uid;

pub struct Reporter {
    pub data_collector: DataCollector,
    pub version: String,
    pub websocket: Client<TcpStream>,
    pub is_connected: Arc<Mutex<bool>>,
    pub hardware_uuid: String,
    pub auth_manager: AuthManager,
}

impl Reporter {
    pub async fn new() -> Result<Self> {
        let auth_manager: AuthManager = AuthManager::new()?;
        let data_collector: DataCollector = DataCollector::new()?;
        let version: String = env!("CARGO_PKG_VERSION").to_string();
        let statics = data_collector.get_statics().await?;
        let is_connected = arcmutex(false);

        // Make this return result ? somehow
        let hardware_uuid: String = machine_uid::get().unwrap();

        let mut websocket = ClientBuilder::new("ws://localhost:8000")?.connect_insecure()?;
        *is_connected.lock() = true;

        if !auth_manager.access_token.is_empty() {
            websocket.send_message(&Message::text(
                &json!({
                    "e": 0x01,
                    "access_token": &auth_manager.access_token,
                })
                .to_string(),
            ))?;
        }

        // websocket.send_message(&Message::text(
        //     &json!({
        //         "e": 0x03,
        //         "version": &version,
        //         "name": "Xornet Reporter",
        //         "statics": statics,
        //     })
        //     .to_string(),
        // ))?;

        return Ok(Self {
            data_collector,
            hardware_uuid,
            version,
            websocket,
            is_connected,
            auth_manager,
        });
    }

    pub fn send_stats(&mut self) -> Result<()> {
        if *self.is_connected.lock() {
            self.websocket.send_message(&Message::text(
                &json!({
                    "e": 0x04,
                    "cpu": self.data_collector.get_cpu()?,
                    "ram": self.data_collector.get_ram()?,
                    "gpu": self.data_collector.get_gpu()?,
                    "processes": self.data_collector.get_total_process_count()?.to_string(),
                    "disks": self.data_collector.get_disks()?,
                })
                .to_string(),
            ))?;
        }

        return Ok(());
    }
}
