"""OpenAI MCP client with Veridion Nexus compliance integration"""
from typing import Optional, Dict, Any, List
import time
import json
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
    from openai import OpenAI
    OPENAI_AVAILABLE = True
except ImportError:
    OPENAI_AVAILABLE = False
    # Create dummy class for type hints
    class OpenAI:
        pass


class VeridionOpenAIMCP:
    """OpenAI MCP client with Veridion Nexus compliance integration"""
    
    def __init__(
        self,
        api_key: str,
        veridion_api_url: Optional[str] = None,
        veridion_api_key: Optional[str] = None,
        agent_id: Optional[str] = None,
        base_url: Optional[str] = None,
        **kwargs
    ):
        if not OPENAI_AVAILABLE:
            raise ImportError(
                "openai package is required. "
                "Install it with: pip install openai"
            )
        
        # Initialize OpenAI client
        self.openai_client = OpenAI(
            api_key=api_key,
            base_url=base_url,
            **kwargs
        )
        
        self.veridion = VeridionClient(
            api_url=veridion_api_url,
            api_key=veridion_api_key,
            agent_id=agent_id or "openai-mcp-agent"
        )
    
    async def chat_completions_create(
        self,
        model: str,
        messages: List[Dict[str, str]],
        **kwargs
    ) -> Dict[str, Any]:
        """Create chat completion with compliance logging"""
        start_time = time.time()
        payload = json.dumps(messages)
        
        try:
            # Call OpenAI
            response = self.openai_client.chat.completions.create(
                model=model,
                messages=messages,
                **kwargs
            )
            
            inference_time_ms = int((time.time() - start_time) * 1000)
            
            # Log to Veridion Nexus
            await self.veridion.log_action(
                action="openai_chat_completion",
                payload=payload,
                target_region="EU",
                inference_time_ms=inference_time_ms,
                system_id="openai",
                model_name=model,
                hardware_type="CLOUD"
            )
            
            return response
            
        except Exception as e:
            await self.veridion.log_action(
                action="openai_chat_error",
                payload=f"Error: {str(e)}",
                target_region="EU"
            )
            raise
    
    async def chat_completions_create_stream(
        self,
        model: str,
        messages: List[Dict[str, str]],
        **kwargs
    ):
        """Create streaming chat completion with compliance logging"""
        start_time = time.time()
        payload = json.dumps(messages)
        
        try:
            # Stream from OpenAI
            stream = self.openai_client.chat.completions.create(
                model=model,
                messages=messages,
                stream=True,
                **kwargs
            )
            
            chunks = []
            for chunk in stream:
                chunks.append(chunk)
                yield chunk
            
            # Log after streaming completes
            inference_time_ms = int((time.time() - start_time) * 1000)
            await self.veridion.log_action(
                action="openai_chat_stream",
                payload=payload,
                target_region="EU",
                inference_time_ms=inference_time_ms,
                system_id="openai",
                model_name=model,
                hardware_type="CLOUD"
            )
            
        except Exception as e:
            await self.veridion.log_action(
                action="openai_stream_error",
                payload=f"Error: {str(e)}",
                target_region="EU"
            )
            raise
    
    async def close(self):
        """Close Veridion client"""
        await self.veridion.close()

