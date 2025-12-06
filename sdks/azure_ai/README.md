# Veridion Nexus - Azure AI SDK

Azure AI SDK with automatic Veridion Nexus compliance integration.

## Installation

```bash
pip install veridion-nexus-sdks[azure]
```

## Usage

```python
from sdks.azure_ai import VeridionAzureAI
from azure.core.credentials import AzureKeyCredential
import asyncio

async def main():
    client = VeridionAzureAI(
        endpoint="https://your-endpoint.openai.azure.com/",
        credential=AzureKeyCredential("your-key"),
        veridion_api_url="http://localhost:8080",
        veridion_api_key="your-veridion-key",
        agent_id="my-azure-agent"
    )
    
    response = await client.complete(
        messages=[{"role": "user", "content": "Hello!"}],
        model="gpt-4"
    )
    
    print(response)
    await client.close()

asyncio.run(main())
```

## Features

- ✅ Automatic compliance logging
- ✅ Inference time tracking
- ✅ Energy consumption calculation
- ✅ Error logging
- ✅ Streaming support

