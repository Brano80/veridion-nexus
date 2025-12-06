# Veridion Nexus - GCP Vertex AI SDK

GCP Vertex AI SDK with automatic Veridion Nexus compliance integration and data sovereignty enforcement.

## Installation

```bash
pip install veridion-nexus-sdks[gcp]
```

## Usage

```python
from sdks.gcp_vertex import VeridionVertexAI

vertex = VeridionVertexAI(
    project="your-project",
    location="europe-west1",  # Must be EU region
    veridion_api_url="http://localhost:8080",
    veridion_api_key="your-veridion-key"
)

# Chat model
chat_model = vertex.get_chat_model(model_name="chat-bison")
response = chat_model.send_message("Hello!")
print(response)

# Text generation model
text_model = vertex.get_text_model(model_name="text-bison")
response = text_model.predict("Explain GDPR.")
print(response)
```

## Features

- ✅ **Data Sovereignty Enforcement**: Only EU regions allowed
- ✅ Automatic compliance logging
- ✅ Chat and text generation support
- ✅ Inference time tracking
- ✅ Error handling

## Supported Regions

Only EU regions are allowed:
- `europe-west1` (Belgium)
- `europe-west4` (Netherlands)
- `europe-west6` (Zurich)
- `europe-central2` (Warsaw)

Non-EU regions will raise `SOVEREIGN_LOCK_VIOLATION` error.

