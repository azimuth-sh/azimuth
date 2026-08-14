use azimuth_annotations::realizes;
use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;

#[realizes("polyglot/identity", "rust-identifies")]
fn identity() -> &'static str {
    "rust"
}

fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8086".into());
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut request = [0_u8; 1024];
        let length = stream.read(&mut request)?;
        let path_is_identity = request[..length].starts_with(b"GET /identity ");
        let (status, body) = if path_is_identity {
            ("200 OK", format!("{}\n", identity()))
        } else {
            ("404 Not Found", String::new())
        };
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use azimuth_annotations::covers;

    #[test]
    #[covers("polyglot/identity", "rust-identifies", "unit", "example", "direct")]
    fn identity_is_rust() {
        assert_eq!(identity(), "rust");
    }
}
