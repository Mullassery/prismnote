# PrismNote AI Training & Fine-Tuning

**Status:** Complete - v0.4 Preview Feature  
**Date:** 2026-06-20  
**Supported Compute Providers:** RunPod, Lambda Labs, Vast, Local

---

## Overview

PrismNote provides integrated AI model fine-tuning capabilities directly from your notebooks. Fine-tune LLMs (Large Language Models) on custom data, deploy inference endpoints, and run production AI workloads without leaving your notebook environment.

### Key Capabilities

1. **Model Fine-Tuning** - Fine-tune any open-source LLM (LLaMA 2, Mistral, etc.)
2. **Cost Optimization** - Automatic cost estimation and budget tracking
3. **Distributed Training** - RunPod integration for scalable GPU training
4. **Production Deployment** - Deploy fine-tuned models as API endpoints
5. **Multi-Provider Support** - Run on RunPod, Lambda Labs, Vast, or local GPUs

---

## Supported Models

### Open-Source Models

| Model | Size | Provider | LoRA Support |
|-------|------|----------|---|
| **LLaMA 2** | 7B, 13B, 70B | Meta |  |
| **Mistral** | 7B | Mistral AI |  |
| **Falcon** | 7B, 40B, 180B | TII |  |
| **Grok-1** | 314B | xAI |  |
| **Code Llama** | 7B, 13B, 34B | Meta |  |
| **MPT** | 3B, 7B, 30B | MosaicML |  |
| **Phi** | 3B | Microsoft |  |

### Commercial Models

| Model | Provider | Fine-tuning |
|-------|----------|---|
| **GPT-3.5 Turbo** | OpenAI | Via API |
| **Claude** | Anthropic | Via API |
| **PaLM 2** | Google | Via MakerSuite |

---

## Quick Start

### 1. Prepare Training Data

```python
# Create JSONL training file
import json

training_data = [
    {
        "instruction": "Summarize this article",
        "input": "Article text here...",
        "output": "Summary text here..."
    },
    ...
]

with open("training.jsonl", "w") as f:
    for item in training_data:
        f.write(json.dumps(item) + "\n")
```

### 2. Create Fine-Tuning Job

```python
import requests

job_config = {
    "model_name": "meta-llama/Llama-2-7b",
    "ai_provider": "llama2",
    "compute_provider": "runpod",
    "training_data_path": "/data/training.jsonl",
    "batch_size": 32,
    "num_epochs": 3,
    "learning_rate": 2e-5
}

response = requests.post(
    "http://localhost:8000/api/ai/fine-tuning/jobs",
    json=job_config
)

job_id = response.json()["job_id"]
print(f"Job created: {job_id}")
print(f"Estimated cost: ${response.json()['estimated_cost_usd']:.2f}")
```

### 3. Start Training

```python
requests.post(
    f"http://localhost:8000/api/ai/fine-tuning/jobs/{job_id}/start"
)

# Monitor progress
while True:
    response = requests.get(
        f"http://localhost:8000/api/ai/fine-tuning/jobs/{job_id}"
    )
    job = response.json()
    print(f"Status: {job['status']}")
    print(f"Loss: {job['metrics']['loss']:.4f}")
    
    if job['status'] in ['Completed', 'Failed']:
        break
```

### 4. Deploy Model

```python
# Get best checkpoint
checkpoints = requests.get(
    f"http://localhost:8000/api/ai/fine-tuning/jobs/{job_id}/checkpoints"
).json()["checkpoints"]

best_checkpoint = min(checkpoints, key=lambda c: c["loss"])

# Deploy as API endpoint
endpoint = requests.post(
    f"http://localhost:8000/api/ai/inference/endpoints",
    json={"checkpoint_id": best_checkpoint["checkpoint_id"]}
).json()

print(f"Endpoint ready at: {endpoint['base_url']}")
print(f"API Key: {endpoint['api_key']}")
```

### 5. Use in Notebooks

```python
import requests

response = requests.post(
    f"{endpoint['base_url']}/completions",
    headers={"Authorization": f"Bearer {endpoint['api_key']}"},
    json={
        "prompt": "Summarize this: [article text]",
        "max_tokens": 100
    }
)

print(response.json()["choices"][0]["text"])
```

---

## Compute Providers

### RunPod

**Best for:** Cost-effective GPU training on 4090s and A100s

```python
config = {
    "compute_provider": "runpod",
    "model_name": "meta-llama/Llama-2-7b",
    ...
}

# Set RunPod API key
import os
os.environ["RUNPOD_API_KEY"] = "your-key-here"
```

**Instance Types:**
- **RTX 4090** - $0.44/hr, 24GB VRAM
- **A100 (40GB)** - $1.29/hr
- **A100 (80GB)** - $1.89/hr
- **H100** - $3.59/hr, 80GB VRAM

**Cost Examples:**
- Fine-tune Llama 7B: 8 hours on 4090 = **$3.52**
- Fine-tune Llama 13B: 24 hours on A100 = **$30.96**

### Lambda Labs

**Best for:** Fast training with guaranteed availability

```python
config = {
    "compute_provider": "lambda",
    "model_name": "meta-llama/Llama-2-7b",
    ...
}
```

**Instance Types:**
- **1x GPU A100** - $1.50/hr
- **2x GPU A100** - $2.98/hr
- **8x GPU H100** - $40/hr

### Vast.ai

**Best for:** Cheapest training (variable pricing)

```python
config = {
    "compute_provider": "vast",
    "model_name": "meta-llama/Llama-2-7b",
    ...
}
```

**Typical Pricing:**
- RTX 4090: $0.25-0.40/hr
- A100: $0.60-1.00/hr

### Local Training

**Best for:** Testing and small models

```python
config = {
    "compute_provider": "local",
    "model_name": "meta-llama/Llama-2-7b",
    ...
}

# Requires GPU locally (RTX 4090, A100, etc.)
# No cost, but slower than cloud GPUs
```

---

## Fine-Tuning Techniques

### LoRA (Low-Rank Adaptation)

**Default technique** - Reduces memory usage by 10x

```python
config = {
    "model_name": "meta-llama/Llama-2-7b",
    "lora_rank": 16,  # Lower = smaller, faster
    "lora_alpha": 32,
    ...
}

# Memory usage: ~11GB instead of 39GB
# Speed: 2x faster than full fine-tuning
```

**LoRA Ranks:**
- **4-8** - Minimal accuracy impact, very fast
- **16** (default) - Good balance
- **32** - Better quality, slower
- **64+** - Near full fine-tuning quality

### QLoRA (Quantized LoRA)

**Ultra-efficient** - 4-bit quantized base model + LoRA

```python
config = {
    "model_name": "meta-llama/Llama-2-13b",
    "quantization": "4bit",
    "lora_rank": 16,
    ...
}

# Memory: 6GB for 13B model (vs 26GB normally)
```

### Full Fine-Tuning

**Maximum quality** - Train entire model

```python
config = {
    "model_name": "meta-llama/Llama-2-7b",
    "lora_rank": None,  # No LoRA = full fine-tuning
    "batch_size": 16,   # Reduce for memory
    ...
}
```

---

## Training Configuration

### Hyperparameters

**Learning Rate:**
- Too high (>1e-3): Unstable training, loss diverges
- Too low (<1e-6): Slow convergence
- Sweet spot: **2e-5 to 5e-5** for most models

**Batch Size:**
- Larger (64, 128): Better gradient estimates, more VRAM
- Smaller (8, 16): Less memory, noisier gradients
- Optimal: **32** for 7B model on RTX 4090

**Epochs:**
- 1-2: Quick fine-tuning (hours)
- 3-5: Standard fine-tuning (1-2 days)
- 10+: Deep specialization (may overfit)

**Warmup Steps:**
- 5-10% of total steps
- Example: 10,000 steps → 500-1000 warmup steps

### Data Preparation

```python
# Data quality matters more than quantity
training_data = [
    # Good: Clear instruction → output pairs
    {
        "instruction": "Summarize in one sentence",
        "input": "This article discusses...",
        "output": "Key finding..."
    },
    # Bad: Vague or incomplete
    {
        "instruction": "Do something",
        "input": "...",
        "output": "..."
    }
]

# Recommendations:
# - 1,000-10,000 examples: Good results
# - 100-1,000 examples: Noisy results
# - 10,000+ examples: Professional quality
```

---

## Cost Optimization

### Budget Tracking

```python
# Set monthly budget alert
requests.post(
    "/api/ai/fine-tuning/budget",
    json={
        "monthly_budget_usd": 500,
        "alert_threshold_percent": 80
    }
)
```

### Cost Reduction Tips

1. **Use LoRA** - 10x cheaper than full fine-tuning
2. **Smaller Models** - 7B is 3x cheaper than 13B
3. **Batch Size** - Larger batches are more efficient
4. **RunPod RTX 4090** - Cheapest GPU option
5. **Short Training** - Stop when validation loss plateaus

### Example Costs

```
LLaMA 7B fine-tuning:
- Full + RTX 4090: $3.52 (8 hours)
- LoRA + RTX 4090: $1.76 (4 hours)
- LoRA + CPU: $0 (takes 2-3 days)

LLaMA 13B fine-tuning:
- Full + A100: $30.96 (24 hours)
- LoRA + A100: $15.48 (12 hours)
- QLoRA + RTX 4090: $4.40 (10 hours)
```

---

## API Reference

### Create Fine-Tuning Job

```
POST /api/ai/fine-tuning/jobs
Body:
{
  "model_name": "meta-llama/Llama-2-7b",
  "ai_provider": "llama2",
  "compute_provider": "runpod",
  "training_data_path": "/data/training.jsonl",
  "batch_size": 32,
  "num_epochs": 3,
  "learning_rate": 2e-5
}

Response:
{
  "job_id": "job-123",
  "status": "Pending",
  "estimated_cost_usd": 3.52,
  ...
}
```

### List Jobs

```
GET /api/ai/fine-tuning/jobs

Response:
{
  "jobs": [
    {"job_id": "job-123", "status": "Running", ...},
    {"job_id": "job-124", "status": "Completed", ...}
  ]
}
```

### Get Job Details

```
GET /api/ai/fine-tuning/jobs/:id

Response:
{
  "job_id": "job-123",
  "status": "Running",
  "metrics": {
    "loss": 2.1,
    "val_loss": 2.3,
    "current_epoch": 2,
    "current_step": 1000
  },
  "training_logs": [...]
}
```

### Start Job

```
POST /api/ai/fine-tuning/jobs/:id/start
```

### Cancel Job

```
POST /api/ai/fine-tuning/jobs/:id/cancel
```

### List Checkpoints

```
GET /api/ai/fine-tuning/jobs/:id/checkpoints
```

### Deploy Endpoint

```
POST /api/ai/inference/endpoints
Body:
{
  "checkpoint_id": "ckpt-123"
}

Response:
{
  "endpoint_id": "ep-123",
  "base_url": "https://api.prismnote.dev/v1/models/ckpt-123",
  "api_key": "sk-...",
  "status": "deploying"
}
```

### List Endpoints

```
GET /api/ai/inference/endpoints
```

### Delete Endpoint

```
DELETE /api/ai/inference/endpoints/:id
```

### Get RunPod Instances

```
GET /api/ai/compute/runpod-instances

Response:
{
  "instances": [
    {
      "instance_id": "rtx4090-1",
      "gpu_model": "RTX 4090",
      "hourly_cost_usd": 0.44,
      "pod_status": "available"
    }
  ]
}
```

---

## Examples

### Example 1: Fine-tune on Customer Data

```python
# Prepare customer support data
training_data = []
for ticket in customer_tickets:
    training_data.append({
        "instruction": "Answer support ticket",
        "input": ticket["question"],
        "output": ticket["answer"]
    })

# Save to JSONL
import json
with open("/data/support.jsonl", "w") as f:
    for item in training_data:
        f.write(json.dumps(item) + "\n")

# Create job
job = requests.post("/api/ai/fine-tuning/jobs", json={
    "model_name": "meta-llama/Llama-2-7b",
    "ai_provider": "llama2",
    "compute_provider": "runpod",
    "training_data_path": "/data/support.jsonl",
    "batch_size": 32,
    "num_epochs": 3,
    "learning_rate": 2e-5
}).json()

# Start training
requests.post(f"/api/ai/fine-tuning/jobs/{job['job_id']}/start")

# Deploy when complete
# ... check job status ...
requests.post(
    "/api/ai/inference/endpoints",
    json={"checkpoint_id": best_checkpoint["checkpoint_id"]}
)
```

### Example 2: Domain-Specific Code Model

```python
# Fine-tune Code Llama for your domain
training_data = []
for code_example in repository_code:
    training_data.append({
        "instruction": "Complete this code",
        "input": code_example["prefix"],
        "output": code_example["suffix"]
    })

job = requests.post("/api/ai/fine-tuning/jobs", json={
    "model_name": "meta-llama/CodeLlama-7b",
    "ai_provider": "llama2",
    "compute_provider": "runpod",
    "training_data_path": "/data/code.jsonl",
    "batch_size": 16,
    "num_epochs": 5,
    "learning_rate": 1e-5
}).json()
```

---

## Best Practices

1. **Quality Over Quantity** - 1,000 good examples > 10,000 poor examples
2. **Monitor Validation Loss** - Stop when it plateaus (overfitting)
3. **Use Early Checkpoints** - Save every 500 steps for comparison
4. **Test on Small Data** - Verify setup with 100 examples first
5. **Track Costs** - Monitor hourly spend, set budget alerts
6. **Use LoRA by Default** - 10x cheaper, usually sufficient

---

## Roadmap

**v0.4 (Current):**
-  Job creation and management
-  RunPod integration
-  Checkpoint management
-  Cost estimation

**v0.5 (Planned):**
- Live training monitoring dashboard
- Multi-GPU distributed training
- Evaluation on test set
- Hyperparameter optimization
- Model comparison tools

**v1.0+ (Future):**
- Real-time inference streaming
- Model versioning and rollback
- A/B testing inference endpoints
- MLOps integration (MLflow, Weights & Biases)
- Custom callback training support

---

*AI training and fine-tuning complete*  
*Enterprise-grade model optimization ready*  
*Cost-effective distributed training via RunPod*

