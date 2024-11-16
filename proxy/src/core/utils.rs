use super::errors::ProxyError;
use colored::*;
use openssl::pkey::PKey;
use reqwest::{Client, Url};
use rustls::{Certificate, ServerConfig};
use rustls_pemfile::Item;
use std::io::BufReader as StdBufReader;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;
use tracing::info;

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../target/shared/generated_state_machines.rs"
));

pub fn load_tls_config() -> Result<Arc<ServerConfig>, ProxyError> {
    let private_key_bytes = include_bytes!("../../alpha-sepolia-certs/server.pem");
    let pkey = PKey::private_key_from_pem(private_key_bytes)?;

    let private_key = rustls::PrivateKey(pkey.private_key_to_der()?);

    let certs = load_certs("proxy/alpha-sepolia-certs/server.crt")?;

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)?;

    Ok(Arc::new(config))
}

fn load_certs(path: &str) -> Result<Vec<Certificate>, ProxyError> {
    let cert_file = std::fs::File::open(path)?;
    let mut cert_reader = StdBufReader::new(cert_file);

    let mut certs = Vec::new();
    while let Some(item) = rustls_pemfile::read_one(&mut cert_reader)? {
        if let Item::X509Certificate(cert) = item {
            certs.push(Certificate(cert.to_vec()));
        }
    }

    Ok(certs)
}

async fn write_response_to_stream(
    tls_stream: &mut TlsStream<TcpStream>,
    response: reqwest::Response,
) -> Result<String, ProxyError> {
    let status = response.status();
    let body_bytes = response.bytes().await?;

    let body = String::from_utf8_lossy(&body_bytes).to_string();

    info!("Response Status: {}", status);
    info!("Response Body: {}", body);

    if status.is_success() {
        let response_str = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        tls_stream.write_all(response_str.as_bytes()).await?;
    } else {
        let error_response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 22\r\n\r\nSepolia Request Failed";
        tls_stream.write_all(error_response.as_bytes()).await?;
        info!("Failed to fetch response. Status: {}", status);
    }

    Ok(body)
}

async fn extract_request_body<R: AsyncBufReadExt + Unpin>(
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
        let mut body_reader = reader.take(length as u64);
        body_reader.read_to_string(&mut body).await?;
    }
    Ok(body)
}

pub async fn handle_connection(
    stream: TcpStream,
    tls_config: Arc<ServerConfig>,
) -> Result<(), ProxyError> {
    let acceptor = TlsAcceptor::from(tls_config);
    let mut tls_stream = acceptor.accept(stream).await?;

    let mut buf_reader = BufReader::new(&mut tls_stream);

    let mut request_header = Vec::new();
    loop {
        let mut line = String::new();
        buf_reader.read_line(&mut line).await?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
        request_header.push(line);
    }
    info!("Request: {:#?}", request_header);

    let request_body = extract_request_body(&request_header, &mut buf_reader).await?;

    if let Some(request_line) = request_header.first() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();

        if parts.len() >= 2 {
            let method = parts[0];
            let path = parts[1];

            info!("Method: {method}, Path: {path}");

            let url = Url::parse(format!("https://alpha-sepolia.starknet.io{}", path).as_str())?;

            let client = Client::new();

            let response = match method {
                "GET" => {
                    info!("Handling GET request");
                    client.get(url.clone()).send().await?
                }
                "POST" => {
                    info!("Handling POST request");
                    info!("Post body: {}", request_body);
                    client
                        .post(url.clone())
                        .body(request_body.clone())
                        .send()
                        .await?
                }
                _ => {
                    info!("Unsupported HTTP method: {method}");
                    return Err(ProxyError::MethodError {
                        method: method.to_string(),
                    });
                }
            };
            let response_body = write_response_to_stream(&mut tls_stream, response).await?;
            run_generated_state_machines(request_body, response_body, path.to_string());
        } else {
            info!("Invalid request format");
        }
    }

    Ok(())
}
