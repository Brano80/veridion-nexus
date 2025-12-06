# Veridion Nexus SDKs

Compliance integration SDKs for major AI platforms. Automatically log all AI operations to Veridion Nexus for EU AI Act, GDPR, and eIDAS compliance.

## Supported Platforms

- ✅ **Azure AI** - Azure OpenAI and Azure AI services
- ✅ **AWS Bedrock** - Amazon Bedrock models
- ✅ **GCP Vertex AI** - Google Cloud Vertex AI
- ✅ **LangChain** - LangChain LLM wrappers
- ✅ **OpenAI MCP** - OpenAI API with Model Context Protocol
- ✅ **HuggingFace** - Transformers pipelines

## Installation

### Install all SDKs
```bash
pip install veridion-nexus-sdks[all]
```

### Install specific platform SDKs
```bash
# Azure AI
pip install veridion-nexus-sdks[azure]

# AWS Bedrock
pip install veridion-nexus-sdks[aws]

# GCP Vertex AI
pip install veridion-nexus-sdks[gcp]

# LangChain
pip install veridion-nexus-sdks[langchain]

# OpenAI
pip install veridion-nexus-sdks[openai]

# HuggingFace
pip install veridion-nexus-sdks[huggingface]
```

## Quick Start

### Environment Variables

Set these environment variables (or pass them to SDK constructors):

```bash
export VERIDION_API_URL="http://localhost:8080"
export VERIDION_API_KEY="your-api-key"
export VERIDION_AGENT_ID="my-ai-agent"
```

### Azure AI Example

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

### AWS Bedrock Example

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

### GCP Vertex AI Example

```python
from sdks.gcp_vertex import VeridionVertexAI

vertex = VeridionVertexAI(
    project="your-project",
    location="europe-west1",  # Must be EU region
    veridion_api_url="http://localhost:8080",
    veridion_api_key="your-veridion-key"
)

chat_model = vertex.get_chat_model(model_name="chat-bison")
response = chat_model.send_message("Hello!")
print(response)
```

### LangChain Example

```python
from sdks.langchain import wrap_langchain_llm
from langchain.llms import OpenAI

# Create your LangChain LLM
llm = OpenAI(temperature=0.7)

# Wrap it with Veridion compliance
veridion_llm = wrap_langchain_llm(
    llm=llm,
    veridion_api_url="http://localhost:8080",
    veridion_api_key="your-veridion-key"
)

# Use it normally - compliance is automatic
response = veridion_llm("Hello!")
print(response)
```

### OpenAI MCP Example

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

### HuggingFace Example

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

### Automatic Compliance Logging
- All AI operations are automatically logged to Veridion Nexus
- Includes inference time, energy consumption, and carbon footprint
- Tracks model name, version, and hardware type

### Data Sovereignty Enforcement
- AWS Bedrock: Only EU regions allowed (eu-west-1, eu-central-1, etc.)
- GCP Vertex: Only EU regions allowed (europe-west1, europe-west4, etc.)
- Non-EU regions raise `SOVEREIGN_LOCK_VIOLATION` error

### Error Handling
- All errors are logged to Veridion Nexus
- Original exceptions are preserved and re-raised
- Compliance logging never blocks your application

### Async Support
- All SDKs support async operations
- Non-blocking compliance logging
- Fire-and-forget logging for sync operations

## Configuration

### VeridionClient Options

```python
from sdks.common import VeridionClient

client = VeridionClient(
    api_url="http://localhost:8080",  # Veridion Nexus API URL
    api_key="your-api-key",            # Optional: API key for authentication
    agent_id="my-agent"                 # Optional: Agent identifier
)
```

### Platform-Specific Options

Each SDK accepts platform-specific options plus Veridion options:

- `veridion_api_url`: Veridion Nexus API URL
- `veridion_api_key`: API key for authentication
- `agent_id`: Unique identifier for your AI agent

## Compliance Features

All SDKs automatically:

1. **Log every AI operation** to Veridion Nexus
2. **Track inference time** for performance monitoring
3. **Calculate energy consumption** (GPU/CPU power ratings)
4. **Calculate carbon footprint** (EU average: 475 g CO2/kWh)
5. **Enforce data sovereignty** (block non-EU regions)
6. **Handle errors gracefully** (log errors without breaking your app)

## License

MIT License - See LICENSE file for details

## Support

- Documentation: https://docs.veridion.nexus
- Issues: https://github.com/veridion-nexus/sdks/issues
- Email: support@veridion.nexus

