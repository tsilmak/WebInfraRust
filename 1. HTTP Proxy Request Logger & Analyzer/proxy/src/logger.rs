use std::net::SocketAddr;
use std::collections::HashMap;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct ConnectRequestLog {
    pub client_addr: SocketAddr,
    pub target_host: String,
    pub target_port: u16,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub raw_request: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct HttpRequestLog {
    pub client_addr: SocketAddr,
    pub method: String,
    pub path: String,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub raw_request: String,
    pub timestamp: String,
}

pub struct ProxyLogger;

impl ProxyLogger {
    pub fn log_proxy_start(addr: SocketAddr) {
        println!("Proxy running at http://{}", addr);
    }

    pub fn log_connection_accepted(peer_addr: SocketAddr) {
        println!("Connection accepted from: {}", peer_addr);
    }

    pub fn log_raw_request(request: &str) {
        println!("Request: {}", request.trim());
    }

    pub fn log_connect_request(target_host: &str, target_port: &str) {
        println!("CONNECT to {}:{}", target_host, target_port);
    }

    pub fn log_connection_established(target: &str) {
        println!("Connection established to {}", target);
    }

    pub fn log_connection_failed(target: &str, error: &str) {
        println!("Failed to connect to {}: {}", target, error);
    }

    pub fn log_error(context: &str, error: &dyn std::fmt::Display) {
        eprintln!("Error in {}: {}", context, error);
    }

    // Parse and log HTTP request details
    pub fn parse_and_log_http_request(raw_request: &str, client_addr: SocketAddr) -> Option<HttpRequestLog> {
        let request_log = Self::parse_http_request(raw_request, client_addr)?;
        
        // Log the parsed request details
        println!("HTTP Request Details:");
        println!("  Method: {}", request_log.method);
        println!("  Path: {}", request_log.path);
        println!("  HTTP Version: {}", request_log.http_version);
        println!("  Client: {}", request_log.client_addr);
        
        // Log important headers
        for header_name in &["User-Agent", "Host", "Accept", "Cache-Control", "Connection"] {
            if let Some(value) = request_log.headers.get(*header_name) {
                println!("  {}: {}", header_name, value);
            }
        }
        
        // Log any other headers
        for (key, value) in &request_log.headers {
            if !["User-Agent", "Host", "Accept", "Cache-Control", "Connection"].contains(&key.as_str()) {
                println!("  {}: {}", key, value);
            }
        }
        
        Some(request_log)
    }

    // Parse and log CONNECT request details
    pub fn parse_and_log_connect_request(raw_request: &str, client_addr: SocketAddr) -> Option<ConnectRequestLog> {
        let request_log = Self::parse_connect_request(raw_request, client_addr)?;
        
        println!("CONNECT Request Details:");
        println!("  Target: {}:{}", request_log.target_host, request_log.target_port);
        println!("  HTTP Version: {}", request_log.http_version);
        println!("  Client: {}", request_log.client_addr);
        
        // Log headers
        for (key, value) in &request_log.headers {
            println!("  {}: {}", key, value);
        }
        
        Some(request_log)
    }

    // Helper function to parse HTTP requests (GET, POST, etc.)
    fn parse_http_request(raw_request: &str, client_addr: SocketAddr) -> Option<HttpRequestLog> {
        let mut lines = raw_request.lines();
        let first_line = lines.next()?.trim();
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        
        if parts.len() < 3 {
            return None;
        }
        
        let method = parts[0].to_string();
        let path = parts[1].to_string();
        let http_version = parts[2].to_string();

        // Parse headers
        let headers = lines
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return None;
                }
                if let Some((k, v)) = line.split_once(':') {
                    Some((k.trim().to_string(), v.trim().to_string()))
                } else {
                    None
                }
            })
            .collect();

        Some(HttpRequestLog {
            client_addr,
            method,
            path,
            http_version,
            headers,
            raw_request: raw_request.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        })
    }

    // Helper function to parse CONNECT requests
    fn parse_connect_request(raw_request: &str, client_addr: SocketAddr) -> Option<ConnectRequestLog> {
        let mut lines = raw_request.lines();
        let first_line = lines.next()?.trim();
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        
        if parts.len() < 3 || parts[0] != "CONNECT" {
            return None;
        }
        
        let target = parts[1];
        let http_version = parts[2].to_string();

        let (host, port_str) = target.split_once(':')?;
        let port: u16 = port_str.parse().ok()?;

        // Parse headers
        let headers = lines
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return None;
                }
                if let Some((k, v)) = line.split_once(':') {
                    Some((k.trim().to_string(), v.trim().to_string()))
                } else {
                    None
                }
            })
            .collect();

        Some(ConnectRequestLog {
            client_addr,
            target_host: host.to_string(),
            target_port: port,
            http_version,
            headers,
            raw_request: raw_request.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        })
    }

    // Log detailed request information with better formatting
    pub fn log_detailed_request(request_str: &str, client_addr: SocketAddr) {
        println!("\n=== Incoming Request ===");
        println!("From: {}", client_addr);
        println!("Timestamp: {}", Utc::now().to_rfc3339());
        
        // Try to parse as HTTP first, then as CONNECT
        if let Some(_) = Self::parse_and_log_http_request(request_str, client_addr) {
            // HTTP request logged
        } else if let Some(_) = Self::parse_and_log_connect_request(request_str, client_addr) {
            // CONNECT request logged
        } else {
            // Fallback to raw logging
            println!("Raw Request:");
            for line in request_str.lines() {
                if !line.trim().is_empty() {
                    println!("  {}", line);
                }
            }
        }
        println!("========================\n");
    }
}