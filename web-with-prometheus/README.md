# web-with-prometheus

Web server with Prometheus metrics integration.

Features:
- Nickel web framework
- Prometheus metrics using `prometheus-client`
- HTTP request counting with method and path labels
- Metrics endpoint at `/metrics`

Server runs on `http://0.0.0.0:8989`.

## Run

```bash
cargo run -p web-with-prometheus
```
