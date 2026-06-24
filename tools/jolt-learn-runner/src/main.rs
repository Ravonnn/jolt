use jolt_learn_runner::{cors_headers, handle_request, repo_root, resolve_jolt_bin, DEFAULT_PORT};
use std::net::TcpListener;
use tiny_http::{Header, Response, Server};

fn main() {
    let port = std::env::var("JOLT_LEARN_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("failed to bind {addr}: {e}");
        std::process::exit(1);
    });
    let server = Server::from_listener(listener, None).expect("server");
    let jolt_bin = resolve_jolt_bin();
    let repo = repo_root();

    eprintln!("jolt-learn-runner listening on http://{addr}");
    eprintln!("jolt binary: {}", jolt_bin.display());
    eprintln!("repo root: {}", repo.display());

    for mut request in server.incoming_requests() {
        let method = request.method().to_string();
        let path = request.url().to_string();
        let mut body = String::new();
        let reader = request.as_reader();
        let _ = std::io::Read::read_to_string(reader, &mut body);

        let (status, json) = handle_request(&method, &path, &body, &jolt_bin, &repo);
        let mut response = Response::from_string(json).with_status_code(status);
        response = response.with_header(
            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
        );
        for header in cors_headers() {
            response = response.with_header(header);
        }
        let _ = request.respond(response);
    }
}
