# Veridion Nexus - OpenAI MCP SDK

OpenAI MCP SDK with automatic Veridion Nexus compliance integration.

## Installation

```bash
pip install veridion-nexus-sdks[openai]
```

## Usage

```python
from sdks.openai_mcp import VeridionOpenAIMCP
import asyncio

async def main():
    client = VeridionOpenAIMCP(
        api_key="your-openai-key",
        veridion_api_url="http://localhost:8080",
        veridion_api_key="your-veridion-key"
    )
    
    response = await client.chat_completions_create(
        model="gpt-4",
        messages=[{"role": "user", "content": "Hello!"}]
    )
    
    print(response.choices[0].message.content)
    await client.close()

asyncio.run(main())
```

## Features

- ✅ Automatic compliance logging
- ✅ Streaming support
- ✅ Inference time tracking
- ✅ Error handling
- ✅ Compatible with OpenAI API

