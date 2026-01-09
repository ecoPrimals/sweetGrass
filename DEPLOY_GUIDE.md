# 🚀 SweetGrass Deployment Guide

**Version**: v0.5.1  
**Status**: ✅ **PRODUCTION READY - A+ TIER**  
**Date**: January 3, 2026

---

## ⚡ Quick Start

### Prerequisites

- Rust 1.70+ (stable)
- PostgreSQL 14+ (optional, for postgres backend)
- Docker (optional, for integration tests)

### Build Release Binary

```bash
cd /path/to/sweetGrass
cargo build --release
```

**Binary Location**: `target/release/sweet-grass-service`  
**Size**: 4.1 MB (optimized)

### Run Service

```bash
# Memory backend (dev/testing)
./target/release/sweet-grass-service \
    --port 8091 \
    --storage-backend memory

# PostgreSQL backend (production)
export DATABASE_URL="postgresql://user:pass@host:5432/sweetgrass"
./target/release/sweet-grass-service \
    --port 8091 \
    --storage-backend postgres

# Sled backend (embedded, pure Rust)
./target/release/sweet-grass-service \
    --port 8091 \
    --storage-backend sled \
    --storage-path ./data/sweetgrass
```

---

## 🔧 Configuration

### Environment Variables

```bash
# Required
export PRIMAL_NAME=sweetgrass
export PRIMAL_DID=did:key:z6MkSweetGrass...

# Storage (choose one)
export STORAGE_BACKEND=memory|postgres|sled
export DATABASE_URL=postgresql://...    # for postgres
export STORAGE_PATH=./data/sweetgrass   # for sled

# Discovery
export DISCOVERY_SERVICE=http://songbird:8080
export DISCOVERY_TIMEOUT=5s

# Optional
export LOG_LEVEL=info
export RUST_LOG=sweet_grass=debug
```

### Configuration File

Create `config.toml`:

```toml
[primal]
name = "sweetgrass"
did = "did:key:z6MkSweetGrass..."

[server]
port = 8091
host = "0.0.0.0"

[storage]
backend = "postgres"
database_url = "postgresql://user:pass@host:5432/sweetgrass"

[discovery]
service_url = "http://songbird:8080"
timeout_secs = 5
```

Run with config:
```bash
./target/release/sweet-grass-service --config config.toml
```

---

## 🔍 Health Checks

### HTTP Endpoints

```bash
# Basic health check
curl http://localhost:8091/health

# Detailed health check
curl http://localhost:8091/health/detailed

# Liveness probe (Kubernetes)
curl http://localhost:8091/health/live

# Readiness probe (Kubernetes)
curl http://localhost:8091/health/ready
```

### Expected Response

```json
{
  "status": "healthy",
  "version": "0.5.1",
  "uptime_secs": 3600,
  "storage_backend": "postgres",
  "capabilities": ["attribution", "provenance", "query"]
}
```

---

## 🐳 Docker Deployment

### Build Image

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/sweet-grass-service /usr/local/bin/
EXPOSE 8091
CMD ["sweet-grass-service", "--port", "8091"]
```

Build and run:
```bash
docker build -t sweetgrass:latest .
docker run -p 8091:8091 \
    -e STORAGE_BACKEND=memory \
    -e PRIMAL_NAME=sweetgrass \
    sweetgrass:latest
```

---

## ☸️ Kubernetes Deployment

### Deployment Manifest

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sweetgrass
  namespace: ecoprimals
spec:
  replicas: 3
  selector:
    matchLabels:
      app: sweetgrass
  template:
    metadata:
      labels:
        app: sweetgrass
    spec:
      containers:
      - name: sweetgrass
        image: sweetgrass:latest
        ports:
        - containerPort: 8091
          name: http
          protocol: TCP
        env:
        - name: STORAGE_BACKEND
          value: "postgres"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: sweetgrass-db
              key: connection-string
        - name: PRIMAL_NAME
          value: "sweetgrass"
        - name: DISCOVERY_SERVICE
          value: "http://songbird.ecoprimals.svc.cluster.local:8080"
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8091
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8091
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "256Mi"
            cpu: "200m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: sweetgrass
  namespace: ecoprimals
spec:
  type: ClusterIP
  ports:
  - port: 8091
    targetPort: 8091
    protocol: TCP
    name: http
  selector:
    app: sweetgrass
```

Deploy:
```bash
kubectl apply -f sweetgrass-deployment.yaml
```

---

## 🔐 Security

### TLS/SSL

SweetGrass uses tarpc with optional TLS for RPC:

```rust
// Enable TLS in production
let config = ServerConfig::with_tls(
    cert_path: "/etc/sweetgrass/tls/cert.pem",
    key_path: "/etc/sweetgrass/tls/key.pem",
);
```

### Authentication

Integration with BearDog for DID-based authentication:

```bash
export AUTH_PRIMAL=http://beardog:8080
export AUTH_REQUIRED=true
```

---

## 📊 Monitoring

### Prometheus Metrics

Endpoint: `http://localhost:8091/metrics`

Key metrics:
- `sweetgrass_braids_total` - Total braids created
- `sweetgrass_queries_total` - Total queries executed
- `sweetgrass_attribution_calcs_total` - Attribution calculations
- `sweetgrass_request_duration_seconds` - Request latency

### Logging

```bash
# Set log level
export RUST_LOG=sweet_grass=info,sweet_grass_service=debug

# JSON logging for production
export LOG_FORMAT=json
```

---

## 🧪 Testing Deployment

### Smoke Test

```bash
# Create a test braid
curl -X POST http://localhost:8091/api/v1/braids \
  -H "Content-Type: application/json" \
  -d '{
    "data_hash": "sha256:test123",
    "mime_type": "text/plain",
    "size": 100,
    "attributed_to": "did:key:z6MkTest"
  }'

# Query braids
curl http://localhost:8091/api/v1/braids

# Check health
curl http://localhost:8091/health
```

---

## 📚 API Documentation

### REST API

Base URL: `http://localhost:8091/api/v1`

**Endpoints**:
- `GET /braids` - List braids
- `GET /braids/{id}` - Get braid by ID
- `GET /braids/hash/{hash}` - Get braid by content hash
- `POST /braids` - Create new braid
- `GET /provenance/{hash}` - Get provenance graph
- `GET /attribution/{hash}` - Get attribution chain

### tarpc RPC

Connect via tarpc for high-performance RPC:

```rust
use tarpc::serde_transport::tcp;
use sweet_grass_service::SweetGrassRpcClient;

let transport = tcp::connect("localhost:8091", Bincode::default).await?;
let client = SweetGrassRpcClient::new(Default::default(), transport).spawn();

let braid = client.get_braid(context::current(), braid_id).await??;
```

---

## 🛠️ Troubleshooting

### Common Issues

**Service won't start**:
- Check port 8091 is not in use: `lsof -i :8091`
- Verify DATABASE_URL is correct
- Check logs: `RUST_LOG=debug ./sweet-grass-service`

**Tests failing**:
- Ensure Docker is running (for integration tests)
- Run with: `cargo test -- --ignored` for Docker tests

**High memory usage**:
- Use Sled or PostgreSQL backend (not memory)
- Adjust connection pool: `export PG_MAX_CONNECTIONS=10`

---

## 🎯 Production Checklist

- [ ] Build release binary (`cargo build --release`)
- [ ] Configure PostgreSQL or Sled backend
- [ ] Set up environment variables
- [ ] Configure TLS/SSL certificates
- [ ] Enable monitoring (Prometheus)
- [ ] Set up log aggregation
- [ ] Configure health checks
- [ ] Test deployment with smoke tests
- [ ] Enable auto-scaling (Kubernetes HPA)
- [ ] Set up backup strategy (database)

---

## 📞 Support

**Documentation**: `/path/to/sweetGrass/docs/`  
**Status**: `STATUS.md`  
**Quick Reference**: `QUICK_REFERENCE.md`

---

**SweetGrass v0.5.1 - Production Ready** ✅

*Fair attribution. Complete transparency. Human dignity preserved.* 🌾

