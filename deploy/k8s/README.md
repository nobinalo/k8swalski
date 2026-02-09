# Kubernetes Manifests

Kubernetes deployment manifests using Kustomize.

## Structure

```
k8s/
├── base/                   # Base resources
│   ├── deployment.yaml
│   ├── service.yaml
│   └── kustomization.yaml
└── overlays/               # Environment-specific overlays
    ├── dev/                # Development environment
    │   ├── kustomization.yaml
    │   └── deployment-patch.yaml
    └── prod/               # Production environment
        ├── kustomization.yaml
        └── deployment-patch.yaml
```

## Usage

### Deploy to Development

```bash
# Preview
kubectl kustomize k8s/overlays/dev

# Apply
kubectl apply -k k8s/overlays/dev

# Port forward
kubectl port-forward -n k8swalski-dev svc/k8swalski 8080:80
```

### Deploy to Production

```bash
# Preview
kubectl kustomize k8s/overlays/prod

# Apply
kubectl apply -k k8s/overlays/prod

# Port forward
kubectl port-forward -n k8swalski-prod svc/k8swalski 8080:80
```

### Deploy Base (Default Namespace)

```bash
kubectl apply -k k8s/base
```

## Configuration

Environment variables can be configured via:
- Deployment patches in overlays
- ConfigMaps
- Secrets

See [deployment.yaml](base/deployment.yaml) for available options.

## Resource Limits

### Development
- CPU: 100m request, 500m limit
- Memory: 128Mi request, 256Mi limit
- Replicas: 1

### Production
- CPU: 200m request, 1000m limit
- Memory: 256Mi request, 512Mi limit
- Replicas: 3
