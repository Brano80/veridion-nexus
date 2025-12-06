# Veridion Nexus - AWS Bedrock SDK

AWS Bedrock SDK with automatic Veridion Nexus compliance integration and data sovereignty enforcement.

## Installation

```bash
pip install veridion-nexus-sdks[aws]
```

## Usage

```python
from sdks.aws_bedrock import VeridionBedrock
import json

bedrock = VeridionBedrock(
    region_name="eu-west-1",  # Must be EU region
    veridion_api_url="http://localhost:8080",
    veridion_api_key="your-veridion-key"
)

response = bedrock.invoke_model(
    model_id="anthropic.claude-v2",
    body={
        "prompt": "Hello!",
        "max_tokens_to_sample": 100
    }
)

print(response)
```

## Features

- ✅ **Data Sovereignty Enforcement**: Only EU regions allowed
- ✅ Automatic compliance logging
- ✅ Inference time tracking
- ✅ Streaming support
- ✅ Error handling

## Supported Regions

Only EU regions are allowed:
- `eu-west-1` (Ireland)
- `eu-west-2` (London)
- `eu-west-3` (Paris)
- `eu-central-1` (Frankfurt)
- `eu-north-1` (Stockholm)

Non-EU regions will raise `SOVEREIGN_LOCK_VIOLATION` error.

