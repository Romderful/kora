# Kora Helm Chart

Deploy [Kora](https://github.com/Romderful/Kora), a Confluent-compatible Schema Registry backed by PostgreSQL, on Kubernetes.

## Prerequisites

- Kubernetes 1.24+
- Helm 3.x
- An existing PostgreSQL database

## Quick Start

```bash
helm install kora oci://ghcr.io/romderful/kora/charts/kora \
  --set database.host=my-postgres.example.com \
  --set database.password=my-secret
```

## Database Configuration

Three configuration modes:

### Host and password

```bash
helm install kora oci://ghcr.io/romderful/kora/charts/kora \
  --set database.host=postgres.example.com \
  --set database.password=secret
```

### Full URL (for SSL, custom params)

```bash
helm install kora oci://ghcr.io/romderful/kora/charts/kora \
  --set database.url="postgres://kora:secret@postgres:5432/kora?sslmode=require"
```

### Existing secret (recommended for production)

```bash
kubectl create secret generic kora-db --from-literal=DATABASE_URL="postgres://..."
helm install kora oci://ghcr.io/romderful/kora/charts/kora \
  --set database.existingSecret=kora-db
```

## Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `image.registry` | Image registry | `ghcr.io` |
| `image.repository` | Image repository | `romderful/kora` |
| `image.tag` | Image tag | `v{appVersion}` |
| `kora.port` | Listen port | `8080` |
| `kora.logLevel` | Log level (RUST_LOG) | `info` |
| `kora.dbPoolMax` | Max DB connections | `20` |
| `kora.maxBodySize` | Max request body (bytes) | `16777216` |
| `database.host` | PostgreSQL host | `""` |
| `database.port` | PostgreSQL port | `5432` |
| `database.user` | PostgreSQL user | `kora` |
| `database.password` | PostgreSQL password | `""` |
| `database.name` | PostgreSQL database | `kora` |
| `database.url` | Full DATABASE_URL override | `""` |
| `database.existingSecret` | Existing secret name | `""` |
| `replicaCount` | Number of replicas | `1` |
| `service.type` | Service type | `ClusterIP` |
| `service.port` | Service port | `8080` |

## Security

The chart enforces [Pod Security Standards "restricted"](https://kubernetes.io/docs/concepts/security/pod-security-standards/) by default:

- Non-root user (UID 65534)
- Read-only root filesystem
- No privilege escalation
- All capabilities dropped
- Seccomp RuntimeDefault profile

## Monitoring

Kora exposes Prometheus metrics at `/metrics`. Pods include `prometheus.io/scrape` annotations by default — any standard Prometheus setup will scrape them automatically.

## Uninstalling

```bash
helm uninstall kora
```
