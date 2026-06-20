# PrismNote Docker Container Code Execution

**Status:** v1.0+ Feature (Infrastructure Ready)  
**Date:** 2026-06-20  
**Requires:** Docker Desktop running in parallel

---

## Overview

Execute code directly in Docker containers from PrismNote notebooks. Access container files, monitor resource usage, and manage container lifecycle—all from within your notebook.

### Key Features

1. **Code Execution in Containers** - Execute Python, bash, Node.js, Ruby in running containers
2. **File Management** - Read, write, and list files in container filesystems
3. **Container Management** - Create, start, stop, and remove containers
4. **Resource Monitoring** - Monitor CPU, memory, network, and disk usage
5. **Logging** - Stream container logs directly to notebooks
6. **Image Management** - Pull Docker images from registries

---

## Quick Start

### Prerequisites

1. Docker Desktop installed and running
2. PrismNote running
3. Docker containers already created or create them through PrismNote

### List Available Containers

**In PrismNote notebook:**
```python
import requests

# Get list of running containers
response = requests.get('http://localhost:8000/api/docker/containers')
containers = response.json()['containers']

for container in containers:
    print(f"{container['name']}: {container['status']}")
```

**Response:**
```json
{
  "containers": [
    {
      "id": "abc123def456",
      "name": "prismnote-dev",
      "image": "python:3.11",
      "status": "running",
      "port_bindings": {"8000/tcp": "8000"}
    }
  ],
  "note": "Requires Docker Desktop running in parallel"
}
```

### Execute Python Code in Container

```python
import requests

code = """
import pandas as pd
df = pd.DataFrame({'a': [1, 2, 3], 'b': [4, 5, 6]})
print(df)
"""

response = requests.post(
    'http://localhost:8000/api/docker/containers/execute',
    json={
        "container_id": "abc123def456",
        "code": code,
        "language": "python",
        "working_dir": "/workspace"
    }
)

result = response.json()
print("STDOUT:", result['stdout'])
print("STDERR:", result['stderr'])
print("Exit Code:", result['exit_code'])
print("Time:", result['execution_time_ms'], "ms")
```

### Execute Bash Commands in Container

```python
import requests

response = requests.post(
    'http://localhost:8000/api/docker/containers/execute',
    json={
        "container_id": "abc123def456",
        "code": "ls -la /workspace && pwd",
        "language": "bash"
    }
)

print(response.json()['stdout'])
```

---

## API Reference

### Container Management

#### List Containers
```
GET /api/docker/containers

Response:
{
  "containers": [
    {
      "id": "string",
      "name": "string",
      "image": "string",
      "status": "running|stopped|created",
      "port_bindings": {"internal": "external"},
      "environment": ["KEY=VALUE"]
    }
  ]
}
```

#### Create Container
```
POST /api/docker/containers/create

Body:
{
  "image": "python:3.11",
  "name": "my-container",
  "environment": {
    "PYTHONUNBUFFERED": "1",
    "MY_VAR": "value"
  },
  "ports": {
    "8000": 8000,
    "5432": 5432
  }
}

Response:
{
  "container": {...},
  "status": "created",
  "note": "Container created but not started"
}
```

#### Start Container
```
POST /api/docker/containers/:container_id/start

Response:
{
  "container_id": "string",
  "status": "started"
}
```

#### Stop Container
```
POST /api/docker/containers/:container_id/stop

Response:
{
  "container_id": "string",
  "status": "stopped"
}
```

#### Remove Container
```
DELETE /api/docker/containers/:container_id

Response:
{
  "container_id": "string",
  "status": "removed"
}
```

### Code Execution

#### Execute Code in Container
```
POST /api/docker/containers/execute

Body:
{
  "container_id": "string",
  "code": "print('hello')",
  "language": "python|bash|node|ruby",
  "working_dir": "/workspace",
  "timeout_seconds": 30
}

Response:
{
  "container_id": "string",
  "exit_code": 0,
  "stdout": "hello\n",
  "stderr": "",
  "execution_time_ms": 125,
  "status": "success"
}
```

Supported Languages:
- `python` - Execute Python code
- `bash` - Execute shell commands
- `node` - Execute Node.js code
- `ruby` - Execute Ruby code

### File Management

#### List Container Files
```
GET /api/docker/containers/:container_id/files-list/:path

Response:
{
  "container_id": "string",
  "path": "/workspace",
  "files": [
    {
      "name": "script.py",
      "path": "/workspace/script.py",
      "size": 1024,
      "is_dir": false,
      "modified_time": "2026-06-20T10:30:00Z"
    }
  ]
}
```

#### Read File from Container
```
GET /api/docker/containers/:container_id/files/:path

Response:
{
  "container_id": "string",
  "path": "/workspace/script.py",
  "content": "print('hello world')",
  "status": "success"
}
```

#### Write File to Container
```
POST /api/docker/containers/:container_id/files/:path

Body:
{
  "content": "print('new content')"
}

Response:
{
  "container_id": "string",
  "path": "/workspace/script.py",
  "status": "written"
}
```

### Monitoring

#### Get Container Logs
```
GET /api/docker/containers/:container_id/logs

Response:
{
  "container_id": "string",
  "logs": "[output from container]",
  "status": "success"
}
```

#### Get Container Statistics
```
GET /api/docker/containers/:container_id/stats

Response:
{
  "container_id": "string",
  "cpu_percent": 25.5,
  "memory_usage": 536870912,
  "memory_limit": 2147483648,
  "network_rx": 1048576,
  "network_tx": 524288,
  "block_read": 10485760,
  "block_write": 5242880,
  "status": "success"
}
```

### Image Management

#### Pull Docker Image
```
POST /api/docker/images/pull

Body:
{
  "image": "python:3.11"
}

Response:
{
  "image": "python:3.11",
  "message": "Pulled image: python:3.11",
  "status": "success"
}
```

---

## Use Cases

### Use Case 1: Multi-Environment Testing

Test code in different Python versions simultaneously:

```python
import requests
import json

# Test in Python 3.9, 3.10, 3.11
python_versions = ["python:3.9", "python:3.10", "python:3.11"]
test_code = "import sys; print(sys.version)"

for version in python_versions:
    # Create container
    create_resp = requests.post(
        'http://localhost:8000/api/docker/containers/create',
        json={
            "image": version,
            "name": f"test-{version.replace(':', '-')}"
        }
    )
    container_id = create_resp.json()['container']['id']
    
    # Start container
    requests.post(
        f'http://localhost:8000/api/docker/containers/{container_id}/start'
    )
    
    # Execute test
    exec_resp = requests.post(
        'http://localhost:8000/api/docker/containers/execute',
        json={
            "container_id": container_id,
            "code": test_code,
            "language": "python"
        }
    )
    
    print(f"{version}: {exec_resp.json()['stdout']}")
    
    # Cleanup
    requests.delete(
        f'http://localhost:8000/api/docker/containers/{container_id}'
    )
```

### Use Case 2: Isolated Data Processing

Process sensitive data in isolated containers:

```python
import requests
import json

# Process sensitive file in isolated container
response = requests.post(
    'http://localhost:8000/api/docker/containers/execute',
    json={
        "container_id": "isolated-worker",
        "code": """
import pandas as pd
import hashlib

# Load sensitive data
df = pd.read_csv('/secure/data.csv')

# Process
df['hashed_id'] = df['id'].apply(lambda x: hashlib.sha256(str(x).encode()).hexdigest())

# Save results
df.to_csv('/results/processed.csv', index=False)
print('Processing complete')
""",
        "language": "python",
        "working_dir": "/workspace"
    }
)

print(response.json()['stdout'])
```

### Use Case 3: System Administration Tasks

Run admin tasks in containers:

```python
import requests

# Update packages in container
response = requests.post(
    'http://localhost:8000/api/docker/containers/execute',
    json={
        "container_id": "admin-container",
        "code": "apt-get update && apt-get install -y curl wget",
        "language": "bash",
        "timeout_seconds": 300
    }
)

if response.json()['exit_code'] == 0:
    print("Packages installed successfully")
else:
    print("Error:", response.json()['stderr'])
```

### Use Case 4: Development Workflow

Develop and test code in containers without local setup:

```python
import requests

# Create development container
container = requests.post(
    'http://localhost:8000/api/docker/containers/create',
    json={
        "image": "python:3.11",
        "name": "dev-env",
        "environment": {
            "PYTHONUNBUFFERED": "1"
        }
    }
).json()['container']

container_id = container['id']

# Start it
requests.post(f'http://localhost:8000/api/docker/containers/{container_id}/start')

# Write code to container
requests.post(
    f'http://localhost:8000/api/docker/containers/{container_id}/files/app.py',
    json={"content": """
import requests
from datetime import datetime

response = requests.get('https://api.example.com/data')
print(f"Data retrieved at {datetime.now()}")
print(response.json())
"""}
)

# Execute code
result = requests.post(
    'http://localhost:8000/api/docker/containers/execute',
    json={
        "container_id": container_id,
        "code": "python /app.py",
        "language": "bash"
    }
).json()

print(result['stdout'])
```

### Use Case 5: Container Resource Monitoring

Monitor container health from notebook:

```python
import requests
import time

container_id = "my-app"

# Monitor for 1 minute
for i in range(12):
    stats = requests.get(
        f'http://localhost:8000/api/docker/containers/{container_id}/stats'
    ).json()
    
    print(f"CPU: {stats['cpu_percent']}%")
    print(f"Memory: {stats['memory_usage'] / 1024 / 1024:.1f}MB / {stats['memory_limit'] / 1024 / 1024 / 1024:.1f}GB")
    print(f"Network RX: {stats['network_rx'] / 1024 / 1024:.1f}MB")
    print("---")
    
    time.sleep(5)
```

---

## Best Practices

### 1. Container Cleanup

Always remove containers when done:

```python
import requests

try:
    # Execute code
    result = requests.post(
        'http://localhost:8000/api/docker/containers/execute',
        json={...}
    )
finally:
    # Always cleanup
    requests.delete(
        f'http://localhost:8000/api/docker/containers/{container_id}'
    )
```

### 2. Error Handling

Always check exit codes:

```python
response = requests.post(
    'http://localhost:8000/api/docker/containers/execute',
    json={...}
).json()

if response['exit_code'] != 0:
    print(f"Error: {response['stderr']}")
else:
    print(f"Success: {response['stdout']}")
```

### 3. Resource Limits

Set timeouts for long-running operations:

```python
response = requests.post(
    'http://localhost:8000/api/docker/containers/execute',
    json={
        "container_id": "id",
        "code": "long_running_task()",
        "language": "python",
        "timeout_seconds": 300  # 5 minutes
    }
)
```

### 4. Isolated Execution

Use separate containers for different tasks:

```python
# Data processing container
data_container = "data-processor"

# ML training container
ml_container = "ml-trainer"

# API server container
api_container = "api-server"

# Each isolated, no interference
```

### 5. Volume Mounting

Use volumes for persistent data:

```python
# When creating container
requests.post(
    'http://localhost:8000/api/docker/containers/create',
    json={
        "image": "python:3.11",
        "name": "data-worker",
        "volumes": {
            "/host/data": "/container/data",
            "/host/results": "/container/results"
        }
    }
)

# Now access mounted data in container code
requests.post(
    'http://localhost:8000/api/docker/containers/execute',
    json={
        "container_id": "data-worker",
        "code": "import pandas as pd\ndf = pd.read_csv('/container/data/input.csv')",
        "language": "python"
    }
)
```

---

## Troubleshooting

### "Docker not available"

**Problem:** Error: "Docker Desktop not running"

**Solution:**
1. Start Docker Desktop
2. Ensure Docker socket is accessible: `docker ps`
3. Check Docker daemon: `docker info`

### "Container not found"

**Problem:** Container ID doesn't exist

**Solution:**
1. List containers: `GET /api/docker/containers`
2. Use correct container ID
3. Check if container was removed

### "Permission denied"

**Problem:** "Cannot access Docker socket"

**Solution:**
```bash
# On Linux, add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Or run with sudo
sudo systemctl restart docker
```

### "Timeout"

**Problem:** Code execution times out

**Solution:**
1. Increase `timeout_seconds`
2. Optimize code
3. Check container resource limits
4. Monitor container CPU/memory usage

### "File not found"

**Problem:** Can't read/write files

**Solution:**
1. Verify path: `GET /api/docker/containers/:id/files-list/:path`
2. Check file permissions in container
3. Ensure working directory exists
4. Use absolute paths

---

## Architecture Diagram

```
PrismNote Notebook
       |
       | REST API
       |
Docker Executor Module
       |
       ├── Container Manager
       │   └── Create/Start/Stop/Remove
       |
       ├── Code Executor
       │   └── Python/Bash/Node/Ruby
       |
       ├── File Manager
       │   └── Read/Write/List files
       |
       └── Monitor
           └── Logs/Stats
       |
Docker Desktop (running in parallel)
       |
       ├── Container 1
       ├── Container 2
       └── Container 3
```

---

## Roadmap

### v1.0 (Current)
- [x] Code execution in containers
- [x] File management
- [x] Container lifecycle management
- [x] Resource monitoring

### v1.1 (Planned)
- [ ] Docker Swarm support
- [ ] Multi-host container management
- [ ] Container networking
- [ ] Volume management
- [ ] Custom network policies

### v1.2+ (Future)
- [ ] Kubernetes Pod execution
- [ ] Container registry integration
- [ ] Advanced scheduling
- [ ] Container templating
- [ ] CI/CD pipeline integration

---

## Security Considerations

### 1. Container Isolation

Each container is isolated:
```
Container 1 (Python) -- Isolated filesystem
Container 2 (Node.js) -- Isolated filesystem
Container 3 (Ruby) -- Isolated filesystem
```

### 2. Code Injection Protection

Sanitize user input:
```python
# Don't do this
code = f"print({user_input})"

# Do this
import shlex
code = f"python -c {shlex.quote(user_input)}"
```

### 3. Resource Limits

Set memory and CPU limits:
```python
requests.post(
    'http://localhost:8000/api/docker/containers/create',
    json={
        "image": "python:3.11",
        "name": "limited-container",
        "memory_limit": "512m",
        "cpu_limit": "0.5"
    }
)
```

### 4. Network Isolation

Containers have restricted network access:
```
Internal Container Network
├── Container-to-Container: Allowed (if on same network)
├── Container-to-Host: Restricted
└── Container-to-External: Allowed (if configured)
```

---

## Conclusion

Docker container code execution in PrismNote enables:
- Multi-environment testing
- Isolated processing
- System administration
- Development workflows
- Resource monitoring

With proper error handling and best practices, containers provide a safe, scalable way to execute code from notebooks.

