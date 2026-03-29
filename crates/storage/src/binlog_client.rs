//! Binlog Client - Slave side network client for replication
//!
//! Connects to master node and receives binlog events.

use crate::binlog_protocol::{
    BinlogEventData, BinlogProtocol, PacketReader, PacketWriter, ReplicationMessage,
};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct BinlogClient {
    server_addr: SocketAddr,
    slave_id: u32,
    stream: Option<TcpStream>,
    current_file: String,
    current_pos: u64,
    heartbeat_interval: Duration,
}

impl BinlogClient {
    pub fn new(master_host: &str, master_port: u16, slave_id: u32) -> std::io::Result<Self> {
        let server_addr: SocketAddr = format!("{}:{}", master_host, master_port).parse().unwrap();

        Ok(Self {
            server_addr,
            slave_id,
            stream: None,
            current_file: String::new(),
            current_pos: 0,
            heartbeat_interval: Duration::from_millis(BinlogProtocol::HEARTBEAT_INTERVAL_MS),
        })
    }

    pub fn connect(&mut self) -> std::io::Result<()> {
        let stream = TcpStream::connect_timeout(
            &self.server_addr,
            Duration::from_millis(BinlogProtocol::CONNECTION_TIMEOUT_MS),
        )?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn start_replication(&mut self) -> std::io::Result<Receiver<BinlogEventData>> {
        let stream = self.stream.as_ref().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "Not connected to master")
        })?;

        let mut stream = stream.try_clone()?;

        let handshake = ReplicationMessage::HandshakeRequest {
            slave_id: self.slave_id,
            host: "127.0.0.1".to_string(),
            port: 0,
            replication_protocol_version: BinlogProtocol::VERSION,
        };

        PacketWriter::write_packet(&mut stream, &handshake.serialize())?;

        let response_data = match PacketReader::read_packet(&mut stream) {
            Ok(Some(d)) => d,
            Ok(None) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Connection closed",
                ))
            }
            Err(e) => return Err(e),
        };

        let response = match ReplicationMessage::deserialize(&response_data) {
            Some(ReplicationMessage::HandshakeResponse {
                binlog_file,
                binlog_pos,
                ..
            }) => {
                self.current_file = binlog_file.clone();
                self.current_pos = binlog_pos;
                ReplicationMessage::HandshakeResponse {
                    binlog_file,
                    binlog_pos,
                    server_id: 0,
                    server_version: String::new(),
                }
            }
            Some(ReplicationMessage::Error { code, message }) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Handshake error {}: {}", code, message),
                ));
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid handshake response",
                ));
            }
        };

        match response {
            ReplicationMessage::HandshakeResponse {
                binlog_file,
                binlog_pos,
                ..
            } => {
                self.current_file = binlog_file;
                self.current_pos = binlog_pos;
            }
            _ => {}
        }

        let (tx, rx) = mpsc::channel();

        let slave_id = self.slave_id;
        let heartbeat_interval = self.heartbeat_interval;

        thread::spawn(move || {
            let _ = run_replication_loop(stream, tx, slave_id, heartbeat_interval);
        });

        Ok(Receiver { inner: rx })
    }

    pub fn request_binlog_position(&mut self) -> std::io::Result<u64> {
        let stream = self.stream.as_ref().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "Not connected to master")
        })?;

        let mut stream = stream.try_clone()?;

        let request = ReplicationMessage::BinlogPosRequest {
            file: self.current_file.clone(),
            pos: self.current_pos,
        };

        PacketWriter::write_packet(&mut stream, &request.serialize())?;

        let response_data = match PacketReader::read_packet(&mut stream) {
            Ok(Some(d)) => d,
            Ok(None) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Connection closed",
                ))
            }
            Err(e) => return Err(e),
        };

        let response = match ReplicationMessage::deserialize(&response_data) {
            Some(m) => m,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid response",
                ))
            }
        };

        match response {
            ReplicationMessage::BinlogPosResponse { file, pos } => {
                self.current_file = file;
                self.current_pos = pos;
                Ok(pos)
            }
            ReplicationMessage::Error { code, message } => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error {}: {}", code, message),
            )),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unexpected response",
            )),
        }
    }

    pub fn send_ack(&mut self, lsn: u64) -> std::io::Result<()> {
        let stream = self.stream.as_ref().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "Not connected to master")
        })?;

        let mut stream = stream.try_clone()?;

        let ack = ReplicationMessage::HeartbeatAck { lsn };
        PacketWriter::write_packet(&mut stream, &ack.serialize())?;

        self.current_pos = lsn;
        Ok(())
    }

    pub fn close(&mut self) {
        if let Some(mut stream) = self.stream.take() {
            let _ = PacketWriter::write_packet(&mut stream, &ReplicationMessage::EOF.serialize());
        }
    }

    pub fn current_position(&self) -> (String, u64) {
        (self.current_file.clone(), self.current_pos)
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
}

impl Drop for BinlogClient {
    fn drop(&mut self) {
        self.close();
    }
}

fn run_replication_loop(
    mut stream: TcpStream,
    tx: mpsc::Sender<BinlogEventData>,
    slave_id: u32,
    heartbeat_interval: Duration,
) -> std::io::Result<()> {
    stream.set_nonblocking(false)?;

    loop {
        match PacketReader::read_packet(&mut stream) {
            Ok(Some(data)) => {
                let msg = match ReplicationMessage::deserialize(&data) {
                    Some(m) => m,
                    None => continue,
                };

                match msg {
                    ReplicationMessage::BinlogData { events, .. } => {
                        for event in events {
                            if tx.send(event).is_err() {
                                return Ok(());
                            }
                        }
                    }
                    ReplicationMessage::Heartbeat { lsn, timestamp } => {
                        let _ = timestamp;
                        let ack = ReplicationMessage::HeartbeatAck { lsn };
                        let _ = PacketWriter::write_packet(&mut stream, &ack.serialize());
                    }
                    ReplicationMessage::EOF => {
                        break;
                    }
                    ReplicationMessage::Error { code, message } => {
                        eprintln!(
                            "Error from master (slave {}): {} - {}",
                            slave_id, code, message
                        );
                        break;
                    }
                    _ => {}
                }
            }
            Ok(None) => {
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(heartbeat_interval);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(())
}

pub struct Receiver<T> {
    inner: mpsc::Receiver<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, mpsc::RecvError> {
        self.inner.recv()
    }

    pub fn try_recv(&self) -> Result<T, mpsc::TryRecvError> {
        self.inner.try_recv()
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, mpsc::RecvTimeoutError> {
        self.inner.recv_timeout(timeout)
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.recv().ok()
    }
}

pub struct BinlogClientBuilder {
    master_host: String,
    master_port: u16,
    slave_id: u32,
}

impl BinlogClientBuilder {
    pub fn new(master_host: &str, master_port: u16, slave_id: u32) -> Self {
        Self {
            master_host: master_host.to_string(),
            master_port,
            slave_id,
        }
    }

    pub fn build(&self) -> std::io::Result<BinlogClient> {
        BinlogClient::new(&self.master_host, self.master_port, self.slave_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_builder() {
        let client = BinlogClientBuilder::new("127.0.0.1", 3333, 2)
            .build()
            .unwrap();

        assert_eq!(client.slave_id, 2);
        assert!(!client.is_connected());
    }
}
