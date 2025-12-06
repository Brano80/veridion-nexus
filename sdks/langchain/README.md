# Veridion Nexus - LangChain SDK

LangChain wrapper with automatic Veridion Nexus compliance integration.

## Installation

```bash
pip install veridion-nexus-sdks[langchain]
```

## Usage

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

## Features

- ✅ Works with any LangChain LLM
- ✅ Automatic compliance logging
- ✅ Async support
- ✅ Inference time tracking
- ✅ Error handling

## Supported LLMs

Works with all LangChain-compatible LLMs:
- OpenAI
- Anthropic
- Cohere
- HuggingFace
- Local models
- Custom LLMs

