//! Binlog Server - Master side network service for replication
//!
//! Accepts connections from slave nodes and pushes binlog events.

use crate::binlog_protocol::{
    BinlogEventData, BinlogProtocol, PacketReader, PacketWriter, ReplicationMessage,
};
use crate::replication::{BinlogEvent, BinlogEventType, BinlogWriter};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct SlaveSubscriber {
    slave_id: u32,
    stream: TcpStream,
    binlog_file: String,
    binlog_pos: u64,
}

impl Clone for SlaveSubscriber {
    fn clone(&self) -> Self {
        Self {
            slave_id: self.slave_id,
            stream: self.stream.try_clone().unwrap(),
            binlog_file: self.binlog_file.clone(),
            binlog_pos: self.binlog_pos,
        }
    }
}

pub struct BinlogServer {
    listener: TcpListener,
    server_id: u32,
    server_version: String,
    binlog_path: PathBuf,
    binlog_writer: Arc<Mutex<BinlogWriter>>,
    subscribers: Arc<Mutex<HashMap<u32, SlaveSubscriber>>>,
    is_running: Arc<Mutex<bool>>,
}

impl BinlogServer {
    pub fn new(
        host: &str,
        port: u16,
        server_id: u32,
        binlog_path: PathBuf,
    ) -> std::io::Result<Self> {
        let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;

        let binlog_writer = BinlogWriter::new(binlog_path.clone())?;

        Ok(Self {
            listener,
            server_id,
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            binlog_path,
            binlog_writer: Arc::new(Mutex::new(binlog_writer)),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            is_running: Arc::new(Mutex::new(false)),
        })
    }

    pub fn start(&self) -> std::io::Result<()> {
        *self.is_running.lock().unwrap() = true;
        let listener = &self.listener;
        let subscribers = self.subscribers.clone();
        let is_running = self.is_running.clone();

        loop {
            if !*is_running.lock().unwrap() {
                break;
            }

            match listener.accept() {
                Ok((stream, addr)) => {
                    let subscribers = subscribers.clone();
                    let server_id = self.server_id;
                    let version = self.server_version.clone();
                    let writer = self.binlog_writer.clone();

                    thread::spawn(move || {
                        if let Err(e) = handle_slave_connection(
                            stream,
                            addr,
                            server_id,
                            &version,
                            writer,
                            subscribers,
                        ) {
                            eprintln!("Error handling slave {}: {}", addr, e);
                        }
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                }
            }
        }

        Ok(())
    }

    pub fn stop(&self) {
        *self.is_running.lock().unwrap() = false;
    }

    pub fn write_event(&self, event: &BinlogEvent) -> std::io::Result<u64> {
        let mut writer = self.binlog_writer.lock().unwrap();
        let lsn = writer.write_event(event)?;

        let event_data = convert_to_event_data(event);
        let msg = ReplicationMessage::BinlogData {
            file: format!("binlog.{:06}", 1),
            pos: lsn,
            events: vec![event_data],
        };

        self.broadcast(&msg);

        Ok(lsn)
    }

    fn broadcast(&self, msg: &ReplicationMessage) {
        let data = msg.serialize();
        let subscribers: Vec<_> = self.subscribers.lock().unwrap().values().cloned().collect();

        for mut subscriber in subscribers {
            if let Err(e) = PacketWriter::write_packet(&mut subscriber.stream, &data) {
                eprintln!("Failed to send to slave {}: {}", subscriber.slave_id, e);
            }
        }
    }

    pub fn register_subscriber(&self, slave_id: u32, subscriber: SlaveSubscriber) {
        self.subscribers
            .lock()
            .unwrap()
            .insert(slave_id, subscriber);
    }

    pub fn unregister_subscriber(&self, slave_id: u32) {
        self.subscribers.lock().unwrap().remove(&slave_id);
    }

    pub fn get_binlog_position(&self) -> u64 {
        let writer = self.binlog_writer.lock().unwrap();
        writer.position()
    }
}

fn handle_slave_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    server_id: u32,
    server_version: &str,
    binlog_writer: Arc<Mutex<BinlogWriter>>,
    subscribers: Arc<Mutex<HashMap<u32, SlaveSubscriber>>>,
) -> std::io::Result<()> {
    stream.set_nonblocking(false)?;

    loop {
        let data = match PacketReader::read_packet(&mut stream) {
            Ok(Some(d)) => d,
            Ok(None) => break,
            Err(e) => {
                eprintln!("Read error from {}: {}", addr, e);
                break;
            }
        };

        let msg = match ReplicationMessage::deserialize(&data) {
            Some(m) => m,
            None => {
                let err = ReplicationMessage::Error {
                    code: 1,
                    message: "Failed to parse message".to_string(),
                };
                PacketWriter::write_packet(&mut stream, &err.serialize())?;
                continue;
            }
        };

        match msg {
            ReplicationMessage::HandshakeRequest {
                slave_id,
                host: _,
                port: _,
                replication_protocol_version,
            } => {
                if replication_protocol_version > BinlogProtocol::VERSION {
                    let err = ReplicationMessage::Error {
                        code: 2,
                        message: "Unsupported replication protocol version".to_string(),
                    };
                    PacketWriter::write_packet(&mut stream, &err.serialize())?;
                    break;
                }

                let writer = binlog_writer.lock().unwrap();
                let response = ReplicationMessage::HandshakeResponse {
                    server_id,
                    server_version: server_version.to_string(),
                    binlog_file: format!("binlog.{:06}", 1),
                    binlog_pos: writer.position(),
                };
                drop(writer);

                PacketWriter::write_packet(&mut stream, &response.serialize())?;

                let subscriber = SlaveSubscriber {
                    slave_id,
                    stream: stream.try_clone()?,
                    binlog_file: format!("binlog.{:06}", 1),
                    binlog_pos: 0,
                };
                subscribers.lock().unwrap().insert(slave_id, subscriber);
            }

            ReplicationMessage::BinlogPosRequest { file, pos } => {
                let _ = (file, pos);
                let writer = binlog_writer.lock().unwrap();
                let response = ReplicationMessage::BinlogPosResponse {
                    file: format!("binlog.{:06}", 1),
                    pos: writer.position(),
                };
                PacketWriter::write_packet(&mut stream, &response.serialize())?;
            }

            ReplicationMessage::HeartbeatAck { lsn: _ } => {}

            ReplicationMessage::EOF => {
                break;
            }

            _ => {
                let err = ReplicationMessage::Error {
                    code: 3,
                    message: "Unexpected message type".to_string(),
                };
                PacketWriter::write_packet(&mut stream, &err.serialize())?;
            }
        }
    }

    Ok(())
}

fn convert_to_event_data(event: &BinlogEvent) -> BinlogEventData {
    BinlogEventData {
        event_type: event.event_type as u8,
        tx_id: event.tx_id,
        table_id: event.table_id,
        database: event.database.clone(),
        table: event.table.clone(),
        sql: event.sql.clone(),
        row_data: event.row_data.clone(),
        lsn: event.lsn,
        timestamp: event.timestamp,
    }
}

pub fn start_heartbeat(server: &BinlogServer) {
    let server = server.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(BinlogProtocol::HEARTBEAT_INTERVAL_MS));

        if !*server.is_running.lock().unwrap() {
            break;
        }

        let writer = server.binlog_writer.lock().unwrap();
        let heartbeat = ReplicationMessage::Heartbeat {
            lsn: writer.current_lsn(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        drop(writer);

        server.broadcast(&heartbeat);
    });
}

impl Clone for BinlogServer {
    fn clone(&self) -> Self {
        Self {
            listener: TcpListener::bind("127.0.0.1:0").unwrap(),
            server_id: self.server_id,
            server_version: self.server_version.clone(),
            binlog_path: self.binlog_path.clone(),
            binlog_writer: self.binlog_writer.clone(),
            subscribers: self.subscribers.clone(),
            is_running: self.is_running.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let temp_dir = std::env::temp_dir();
        let binlog_path = temp_dir.join("test_binlog_server");

        let server = BinlogServer::new("127.0.0.1", 0, 1, binlog_path);
        assert!(server.is_ok());
    }
}
