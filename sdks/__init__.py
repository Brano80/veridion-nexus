"""Veridion Nexus SDKs for AI platforms"""

# Import all SDKs
try:
    from .azure_ai import VeridionAzureAI
except ImportError:
    VeridionAzureAI = None

try:
    from .aws_bedrock import VeridionBedrock
except ImportError:
    VeridionBedrock = None

try:
    from .gcp_vertex import VeridionVertexAI, VeridionChatModel, VeridionTextModel
except ImportError:
    VeridionVertexAI = None
    VeridionChatModel = None
    VeridionTextModel = None

try:
    from .langchain import wrap_langchain_llm, VeridionLangChainWrapper
except ImportError:
    wrap_langchain_llm = None
    VeridionLangChainWrapper = None

try:
    from .openai_mcp import VeridionOpenAIMCP
except ImportError:
    VeridionOpenAIMCP = None

try:
    from .huggingface import create_veridion_pipeline, VeridionHuggingFacePipeline
except ImportError:
    create_veridion_pipeline = None
    VeridionHuggingFacePipeline = None

from .common import VeridionClient

__all__ = [
    "VeridionClient",
    "VeridionAzureAI",
    "VeridionBedrock",
    "VeridionVertexAI",
    "VeridionChatModel",
    "VeridionTextModel",
    "wrap_langchain_llm",
    "VeridionLangChainWrapper",
    "VeridionOpenAIMCP",
    "create_veridion_pipeline",
    "VeridionHuggingFacePipeline",
]

