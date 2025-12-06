# Veridion Nexus - HuggingFace Pipelines SDK

HuggingFace Transformers pipelines with automatic Veridion Nexus compliance integration.

## Installation

```bash
pip install veridion-nexus-sdks[huggingface]
```

## Usage

```python
from sdks.huggingface import create_veridion_pipeline

# Create a Veridion-wrapped pipeline
pipeline = create_veridion_pipeline(
    task="text-generation",
    model="gpt2",
    device=-1,  # -1 for CPU, 0+ for GPU
    veridion_api_url="http://localhost:8080",
    veridion_api_key="your-veridion-key"
)

# Use it normally - compliance is automatic
result = pipeline("Hello, how are you?")
print(result)
```

## Features

- ✅ Works with all HuggingFace pipeline tasks
- ✅ Automatic compliance logging
- ✅ GPU/CPU power tracking
- ✅ Energy consumption calculation
- ✅ Carbon footprint tracking

## Supported Tasks

All HuggingFace pipeline tasks are supported:
- `text-generation`
- `text-classification`
- `question-answering`
- `summarization`
- `translation`
- And more...

