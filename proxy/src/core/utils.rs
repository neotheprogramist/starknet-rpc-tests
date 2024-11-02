use super::errors::ProxyError;
use openssl::pkey::PKey;
use reqwest::{blocking::Client, Body, Method, Response, Url};
use rustls::{Certificate, ServerConfig, ServerConnection, StreamOwned};
use rustls_pemfile::Item;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
    net::TcpStream,
    sync::Arc,
};

use tracing::info;

use crate::test_cases::state_machines::{GET_BLOCK_NUMBER_URL, Factory};

pub fn load_tls_config() -> Result<Arc<ServerConfig>, ProxyError> {
    let private_key_bytes = include_bytes!("../../alpha-sepolia-certs/server.pem");
    let pkey = PKey::private_key_from_pem(private_key_bytes)?;

    let private_key = rustls::PrivateKey(pkey.private_key_to_der()?);

    let certs = load_certs("proxy/alpha-sepolia-certs/server.crt")?;

    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)?;

    Ok(Arc::new(config))
}

fn load_certs(path: &str) -> Result<Vec<Certificate>, ProxyError> {
    let cert_file = File::open(path)?;
    let mut cert_reader = BufReader::new(cert_file);

    let mut certs = Vec::new();
    while let Some(item) = rustls_pemfile::read_one(&mut cert_reader)? {
        if let Item::X509Certificate(cert) = item {
            certs.push(Certificate(cert.to_vec()));
        }
    }

    Ok(certs)
}

fn write_response_to_stream(
    tls_stream: &mut StreamOwned<ServerConnection, TcpStream>,
    response: reqwest::blocking::Response,
) -> Result<String, ProxyError> {
    let mut body = "".to_string();
    if response.status().is_success() {
        body = response.text()?;
        info!("Response body: {}", body);

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        tls_stream.write_all(response.as_bytes())?;
    } else {
        let error_response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 20\r\n\r\nSepolia Request Failed".to_string();
        tls_stream.write_all(error_response.as_bytes())?;
        info!("Failed to fetch response. Status: {}", response.status());
    }
    Ok(body)
}

fn extract_request_body<R: BufRead>(
    request_header: &[String],
    reader: &mut R,
) -> Result<String, ProxyError> {
    let content_length = request_header
        .iter()
        .find(|line| line.to_lowercase().starts_with("content-length:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|length| length.parse::<usize>().ok());

    let mut body = String::new();
    if let Some(length) = content_length {
        reader.take(length as u64).read_to_string(&mut body)?;
    }
    Ok(body)
}

pub fn handle_connection(
    stream: TcpStream,
    tls_config: Arc<ServerConfig>,
) -> Result<(), ProxyError> {
    let server_conn = ServerConnection::new(tls_config)?;
    let mut tls_stream = StreamOwned::new(server_conn, stream);

    let mut buf_reader: BufReader<&mut StreamOwned<ServerConnection, TcpStream>> =
        BufReader::new(&mut tls_stream);

    let mut request_header = Vec::new();
    loop {
        let mut line = String::new();
        buf_reader.read_line(&mut line)?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
        request_header.push(line);
    }

    info!("Request: {:#?}", request_header);

    if let Some(request_line) = request_header.first() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();

        if parts.len() >= 2 {
            let method = parts[0];
            let path = parts[1];

            info!("Method: {method}, Path: {path}");

            let url = Url::parse(format!("https://alpha-sepolia.starknet.io{}", path).as_str())?;

            let state_factory = Factory::new();

            let client = Client::new();

            let response = match method {
                "GET" => {
                    info!("Handling GET request");
                    client.get(url.clone()).send()?
                }
                "POST" => {
                    info!("Handling POST request");
                    let body = extract_request_body(&request_header, &mut buf_reader)?;

                    info!("Received POST body: {}", body);

                    client.post(url.clone()).body(body).send()?
                }
                _ => {
                    info!("Unsupported HTTP method: {method}");
                    return Err(ProxyError::MethodError {
                        method: method.to_string(),
                    });
                }
            };

            let response_body = write_response_to_stream(&mut tls_stream, response)?;
            state_factory.block_number_machine.step(Some(response_body))?;
        } else {
            info!("Invalid request format");
        }
    }

    Ok(())
}
