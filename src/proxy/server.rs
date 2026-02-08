use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;

#[derive(Clone, Debug)]
pub struct Rule {
    pub id: String,
    pub domain: String,
    pub target: String,
    pub protocol: String,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct LogEvent {
    pub time: String,
    pub method: String,
    pub protocol: String,
    pub url: String,
    pub status: String,
    pub status_code: i32,
}

pub struct ProxyServer {
    shutdown_tx: Arc<Mutex<Option<watch::Sender<bool>>>>,
    rules: Arc<RwLock<Vec<Rule>>>,
    log_sender: mpsc::Sender<LogEvent>,
}

impl ProxyServer {
    pub fn new(log_sender: mpsc::Sender<LogEvent>) -> Self {
        Self {
            shutdown_tx: Arc::new(Mutex::new(None)),
            rules: Arc::new(RwLock::new(Vec::new())),
            log_sender,
        }
    }

    pub fn update_rules(&self, new_rules: Vec<Rule>) {
        if let Ok(mut rules) = self.rules.write() {
            *rules = new_rules;
            println!("Updated proxy rules: {} rules active", rules.len());
        }
    }

    pub async fn start(&self, ports: Vec<u16>) -> Result<(), Box<dyn std::error::Error>> {
        let mut unique_ports = ports;
        unique_ports.sort_unstable();
        unique_ports.dedup();

        if unique_ports.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "No proxy ports provided").into());
        }

        let mut listeners = Vec::new();
        for port in unique_ports {
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            let listener = TcpListener::bind(addr).await?;
            listeners.push((port, addr, listener));
        }

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        if let Ok(mut shutdown_guard) = self.shutdown_tx.lock() {
            if let Some(previous_tx) = shutdown_guard.take() {
                let _ = previous_tx.send(true);
            }
            *shutdown_guard = Some(shutdown_tx);
        }

        let mut listener_tasks = Vec::new();
        for (port, addr, listener) in listeners {
            println!("Proxy server listening on http://{}", addr);

            let _ = self.log_sender.send(LogEvent {
                time: time::OffsetDateTime::now_utc().format(&time::format_description::parse("[hour]:[minute]:[second]")?).unwrap(),
                method: "SYSTEM".to_string(),
                protocol: "SYSTEM".to_string(),
                url: format!("Proxy server started on port {}", port),
                status: "OK".to_string(),
                status_code: 200,
            }).await;

            let mut stop_signal = shutdown_rx.clone();
            let rules = self.rules.clone();
            let log_sender = self.log_sender.clone();

            let task = tokio::spawn(async move {
                loop {
                    tokio::select! {
                        changed = stop_signal.changed() => {
                            match changed {
                                Ok(()) => {
                                    if *stop_signal.borrow() {
                                        println!("Proxy listener on port {} stopping...", port);
                                        break;
                                    }
                                }
                                Err(_) => {
                                    println!("Proxy listener on port {} stopping...", port);
                                    break;
                                }
                            }
                        }
                        result = listener.accept() => {
                            match result {
                                Ok((stream, _)) => {
                                    let io = TokioIo::new(stream);
                                    let rules = rules.clone();
                                    let log_sender = log_sender.clone();
                                    tokio::task::spawn(async move {
                                        if let Err(err) = http1::Builder::new()
                                            .preserve_header_case(true)
                                            .title_case_headers(true)
                                            .serve_connection(io, service_fn(move |req| proxy(req, rules.clone(), log_sender.clone())))
                                            .with_upgrades()
                                            .await
                                        {
                                            println!("Failed to serve connection: {:?}", err);
                                        }
                                    });
                                }
                                Err(e) => println!("Error accepting connection: {}", e),
                            }
                        }
                    }
                }
            });

            listener_tasks.push(task);
        }

        let mut shutdown_main = shutdown_rx.clone();
        let _ = shutdown_main.changed().await;
        println!("Proxy server stopping...");
        let _ = self.log_sender.send(LogEvent {
            time: time::OffsetDateTime::now_utc().format(&time::format_description::parse("[hour]:[minute]:[second]")?).unwrap(),
            method: "SYSTEM".to_string(),
            protocol: "SYSTEM".to_string(),
            url: "Proxy server stopped".to_string(),
            status: "OK".to_string(),
            status_code: 200,
        }).await;

        for task in listener_tasks {
            let _ = task.await;
        }

        Ok(())
    }

    pub fn stop(&self) {
        if let Ok(mut shutdown_guard) = self.shutdown_tx.lock()
            && let Some(tx) = shutdown_guard.take()
        {
            let _ = tx.send(true);
        }
    }
}

async fn proxy(
    req: Request<hyper::body::Incoming>,
    rules: Arc<RwLock<Vec<Rule>>>,
    log_sender: mpsc::Sender<LogEvent>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let method = req.method().to_string();
    let url = req.uri().to_string();
    let protocol = format!("{:?}", req.version());

    // Extract host for rule matching
    let host_str = req.uri().host().map(|h| h.to_string()).or_else(|| {
        req.headers().get(hyper::header::HOST)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.split(':').next().unwrap_or(h).to_string())
    });

    // Check for rules match
    let mut target_addr = None;
    if let Some(host) = &host_str {
        if let Ok(rules) = rules.read() {
            for rule in rules.iter() {
                if rule.enabled && rule.domain == *host {
                    println!("Matched rule: {} -> {}", rule.domain, rule.target);
                    let target = rule.target.clone();
                    target_addr = Some(if !target.is_empty() && target.chars().all(|c| c.is_numeric()) {
                        format!("127.0.0.1:{}", target)
                    } else if !target.is_empty() && !target.contains(':') {
                        format!("{}:80", target)
                    } else {
                        target
                    });
                    break;
                }
            }
        }
    }

    if Method::CONNECT == req.method() {
        // HTTPS Tunneling
        let addr = if let Some(target) = target_addr {
            Some(target)
        } else {
            host_addr(req.uri())
        };

        if let Some(addr) = addr {
            let log_sender_clone = log_sender.clone();
            let method_clone = method.clone();
            let url_clone = url.clone();
            
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        if let Err(e) = tunnel(upgraded, addr).await {
                            println!("server io error: {}", e);
                        };
                        // Log successful tunnel
                        let _ = log_sender_clone.send(LogEvent {
                            time: time::OffsetDateTime::now_utc().format(&time::format_description::parse("[hour]:[minute]:[second]").unwrap()).unwrap(),
                            method: method_clone,
                            protocol: "HTTPS".to_string(),
                            url: url_clone,
                            status: "Tunnel Established".to_string(),
                            status_code: 200,
                        }).await;
                    }
                    Err(e) => println!("upgrade error: {}", e),
                }
            });

            Ok(Response::new(empty()))
        } else {
            println!("CONNECT host is not socket addr: {:?}", req.uri());
            let mut resp = Response::new(full("CONNECT must be to a socket address"));
            *resp.status_mut() = StatusCode::BAD_REQUEST;
            Ok(resp)
        }
    } else {
        // Standard HTTP Proxy
        let addr = if let Some(target) = target_addr {
            target
        } else {
            // Determine upstream address
            if let Some(host) = req.uri().host() {
                let port = req.uri().port_u16().unwrap_or(80);
                format!("{}:{}", host, port)
            } else {
                // Fallback to Host header
                req.headers().get(hyper::header::HOST)
                    .and_then(|h| h.to_str().ok())
                    .map(|h| {
                        if h.contains(':') {
                            h.to_string()
                        } else {
                            format!("{}:80", h)
                        }
                    })
                    .unwrap_or_else(|| String::new())
            }
        };

        if addr.is_empty() {
             return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full("Missing Host"))
                    .unwrap());
        }

        let stream = match TcpStream::connect(&addr).await {
            Ok(stream) => stream,
            Err(e) => {
                println!("connect error to {}: {}", addr, e);
                return Ok(Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .body(full(format!("Error connecting to upstream: {}", e)))
                    .unwrap());
            }
        };

        let io = TokioIo::new(stream);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });

        let resp = sender.send_request(req).await?;
        
        // Log response
        let _ = log_sender.send(LogEvent {
            time: time::OffsetDateTime::now_utc().format(&time::format_description::parse("[hour]:[minute]:[second]").unwrap()).unwrap(),
            method,
            protocol,
            url,
            status: resp.status().to_string(),
            status_code: resp.status().as_u16() as i32,
        }).await;

        Ok(resp.map(|b| b.boxed()))
    }
}

fn host_addr(uri: &hyper::Uri) -> Option<String> {
    uri.authority().map(|auth| auth.to_string())
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
    .map_err(|never| match never {})
    .boxed()
}

// Create a TCP connection to host:port, build a tunnel between the connection and
// the upgraded connection
async fn tunnel(
    upgraded: hyper::upgrade::Upgraded,
    addr: String,
) -> std::io::Result<()> {
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    println!(
        "client wrote {} bytes and received {} bytes",
        from_client, from_server
    );

    Ok(())
}
