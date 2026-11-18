use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

pub struct HttpServer {
    address: SocketAddr,
    requests: Arc<Mutex<Vec<String>>>,
    stopping: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl HttpServer {
    pub fn start(
        response: impl Fn(&str, SocketAddr) -> (String, Vec<u8>) + Send + 'static,
    ) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stopping = Arc::new(AtomicBool::new(false));
        let log = requests.clone();
        let stop = stopping.clone();
        let worker = std::thread::spawn(move || {
            for connection in listener.incoming() {
                if stop.load(Ordering::SeqCst) {
                    break;
                }
                let mut connection = connection.unwrap();
                connection
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                let mut bytes = Vec::new();
                let mut buffer = [0; 4096];
                while !bytes.windows(4).any(|v| v == b"\r\n\r\n") {
                    let count = connection.read(&mut buffer).unwrap_or(0);
                    if count == 0 {
                        break;
                    }
                    bytes.extend_from_slice(&buffer[..count]);
                }
                let request = String::from_utf8_lossy(&bytes);
                let Some(path) = request
                    .lines()
                    .next()
                    .and_then(|l| l.split_whitespace().nth(1))
                else {
                    continue;
                };
                log.lock().unwrap().push(path.to_string());
                let (kind, body) = response(path, address);
                let _=write!(connection,"HTTP/1.1 200 OK\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len());
                let _ = connection.write_all(&body);
            }
        });
        Self {
            address,
            requests,
            stopping,
            worker: Some(worker),
        }
    }
    pub fn url(&self) -> String {
        format!("http://{}", self.address)
    }
    pub fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}

impl Drop for HttpServer {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.address);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}
