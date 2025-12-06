"""Azure AI client with Veridion Nexus compliance integration"""
from typing import Optional, List, Dict, Any
import time
import os
from pathlib import Path

# Import from common module
try:
    from ..common.veridion_client import VeridionClient
except ImportError:
    # Fallback for direct execution
    import sys
    sys.path.insert(0, str(Path(__file__).parent.parent.parent))
    from sdks.common.veridion_client import VeridionClient

try:
    from azure.ai.inference import ChatCompletionsClient
    from azure.core.credentials import AzureKeyCredential
    AZURE_AI_AVAILABLE = True
except ImportError:
    AZURE_AI_AVAILABLE = False
    # Create dummy classes for type hints
    class ChatCompletionsClient:
        pass
    class AzureKeyCredential:
        pass


class VeridionAzureAI:
    """Azure AI client with Veridion Nexus compliance integration"""
    
    def __init__(
        self,
        endpoint: str,
        credential: Any,  # AzureKeyCredential
        veridion_api_url: Optional[str] = None,
        veridion_api_key: Optional[str] = None,
        agent_id: Optional[str] = None,
        **kwargs
    ):
        if not AZURE_AI_AVAILABLE:
            raise ImportError(
                "azure-ai-inference package is required. "
                "Install it with: pip install azure-ai-inference"
            )
        
        # Initialize Azure AI client
        self.azure_client = ChatCompletionsClient(
            endpoint=endpoint,
            credential=credential,
            **kwargs
        )
        
        self.veridion = VeridionClient(
            api_url=veridion_api_url,
            api_key=veridion_api_key,
            agent_id=agent_id or "azure-ai-agent"
        )
        self.endpoint = endpoint
    
    async def complete(
        self,
        messages: List[Dict[str, str]],
        model: str = "gpt-4",
        **kwargs
    ) -> Dict[str, Any]:
        """Complete chat with compliance logging"""
        
        start_time = time.time()
        payload = str(messages)
        
        try:
            # Call Azure AI
            response = await self.azure_client.complete(
                messages=messages,
                model=model,
                **kwargs
            )
            
            # Calculate inference time
            inference_time_ms = int((time.time() - start_time) * 1000)
            
            # Log to Veridion Nexus
            await self.veridion.log_action(
                action="azure_ai_completion",
                payload=payload,
                target_region="EU",  # Azure EU regions
                inference_time_ms=inference_time_ms,
                system_id="azure-ai",
                model_name=model,
                hardware_type="CLOUD"
            )
            
            return response
            
        except Exception as e:
            # Log error
            await self.veridion.log_action(
                action="azure_ai_error",
                payload=f"Error: {str(e)}",
                target_region="EU"
            )
            raise
    
    async def stream(
        self,
        messages: List[Dict[str, str]],
        model: str = "gpt-4",
        **kwargs
    ):
        """Stream chat with compliance logging"""
        
        start_time = time.time()
        payload = str(messages)
        
        try:
            # Stream from Azure AI
            async for chunk in self.azure_client.stream(
                messages=messages,
                model=model,
                **kwargs
            ):
                yield chunk
            
            # Log after streaming completes
            inference_time_ms = int((time.time() - start_time) * 1000)
            await self.veridion.log_action(
                action="azure_ai_stream",
                payload=payload,
                target_region="EU",
                inference_time_ms=inference_time_ms,
                system_id="azure-ai",
                model_name=model,
                hardware_type="CLOUD"
            )
            
        except Exception as e:
            await self.veridion.log_action(
                action="azure_ai_stream_error",
                payload=f"Error: {str(e)}",
                target_region="EU"
            )
            raise
    
    async def close(self):
        """Close clients"""
        await self.veridion.close()
        if hasattr(self.azure_client, 'close'):
            await self.azure_client.close()

