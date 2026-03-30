# tcp-http-server

A tiny, from-scratch HTTP/1.1 server built on `TcpListener` / `TcpStream` (no external dependencies).

## What it does (today)

- **Listens on**: `0.0.0.0:42069`
- **Parses**: HTTP/1.1 request line + headers (+ optional body via `Content-Length`)
- **Routes**:
  - **`GET /`** → `200 OK` (default response)
  - **anything else** → `404 Not Found`

## Run

```bash
cargo run
```

You should see:

```text
Server started on port 42069
```

## Try it

```bash
curl -i http://127.0.0.1:42069/
```

```bash
curl -i http://127.0.0.1:42069/does-not-exist
```

```txt
Not Found
```

## Project layout

- **`src/main.rs`**: binary entrypoint
- **`src/cmd/http_server.rs`**: starts the server (port is currently hard-coded)
- **`src/internal/`**: request parsing + response building

## Work in progress

1. Benchmarks

2. Concurrency using Tokio

3. HTTP/2.0 support

4. Chunk/Streaming

5. Web-socket support
