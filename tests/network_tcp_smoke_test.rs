use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

#[test]
fn test_tcp_listener_bind_and_accept() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get local addr");

    let handle = std::thread::spawn(move || {
        let _ = listener.accept();
    });

    let stream =
        TcpStream::connect_timeout(&addr, Duration::from_secs(5)).expect("Failed to connect");
    assert!(stream.peer_addr().is_ok());
    let _ = handle.join();
}

#[test]
fn test_tcp_stream_peer_addr() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get local addr");

    let handle = std::thread::spawn(move || {
        let _ = listener.accept();
    });

    let stream =
        TcpStream::connect_timeout(&addr, Duration::from_secs(5)).expect("Failed to connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();

    assert!(stream.peer_addr().is_ok());
    assert!(stream.local_addr().is_ok());
    let _ = handle.join();
}

#[test]
fn test_tcp_multiple_connections() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get local addr");

    let handle = std::thread::spawn(move || {
        let mut count = 0;
        for conn in listener.incoming().take(3) {
            if conn.is_ok() {
                count += 1;
            }
        }
        count
    });

    for _ in 0..3 {
        let stream =
            TcpStream::connect_timeout(&addr, Duration::from_secs(5)).expect("Failed to connect");
        assert!(stream.peer_addr().is_ok());
    }

    let count = handle.join().expect("Thread panicked");
    assert_eq!(count, 3);
}

#[test]
fn test_tcp_connection_refused_on_closed_port() {
    use std::net::SocketAddr;
    use std::str::FromStr;
    let addr = SocketAddr::from_str("127.0.0.1:1").unwrap();
    let result = TcpStream::connect_timeout(&addr, Duration::from_millis(100));
    assert!(result.is_err());
}

#[test]
fn test_tcp_set_options() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get local addr");

    let handle = std::thread::spawn(move || {
        let _ = listener.accept();
    });

    let stream =
        TcpStream::connect_timeout(&addr, Duration::from_secs(5)).expect("Failed to connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    stream.set_nodelay(true).unwrap();

    let _ = handle.join();
}

#[test]
fn test_tcp_stream_write_read() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get local addr");

    let handle = std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 8];
            let _ = stream.read(&mut buf);
            let _ = stream.write_all(&buf);
        }
    });

    let mut stream =
        TcpStream::connect_timeout(&addr, Duration::from_secs(5)).expect("Failed to connect");

    let data = b"hello!\n";
    stream.write_all(data).expect("Failed to write");

    let mut buf = vec![0u8; data.len()];
    let n = stream.read(&mut buf).expect("Failed to read");
    assert_eq!(n, data.len());
    assert_eq!(&buf[..], data);

    let _ = handle.join();
}
