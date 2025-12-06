"""GCP Vertex AI client with Veridion Nexus compliance integration"""
from typing import Optional, Dict, Any, List
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
    from google.cloud import aiplatform
    from vertexai.preview.language_models import ChatModel, TextGenerationModel
    VERTEX_AI_AVAILABLE = True
except ImportError:
    VERTEX_AI_AVAILABLE = False
    # Create dummy classes for type hints
    class ChatModel:
        pass
    class TextGenerationModel:
        pass
    class aiplatform:
        @staticmethod
        def init(*args, **kwargs):
            pass


class VeridionVertexAI:
    """GCP Vertex AI client with Veridion Nexus compliance integration"""
    
    def __init__(
        self,
        project: str,
        location: str = "europe-west1",  # Default to EU region
        veridion_api_url: Optional[str] = None,
        veridion_api_key: Optional[str] = None,
        agent_id: Optional[str] = None,
        **kwargs
    ):
        if not VERTEX_AI_AVAILABLE:
            raise ImportError(
                "google-cloud-aiplatform package is required. "
                "Install it with: pip install google-cloud-aiplatform"
            )
        
        # Check location - block non-EU regions
        if not location.startswith("europe-"):
            raise ValueError(
                "SOVEREIGN_LOCK_VIOLATION: "
                "Vertex AI must use EU regions (europe-west1, europe-west4, etc.)"
            )
        
        # Initialize Vertex AI
        aiplatform.init(project=project, location=location, **kwargs)
        
        self.location = location
        self.project = project
        self.veridion = VeridionClient(
            api_url=veridion_api_url,
            api_key=veridion_api_key,
            agent_id=agent_id or "gcp-vertex-agent"
        )
    
    def get_chat_model(
        self,
        model_name: str = "chat-bison",
        **kwargs
    ) -> 'VeridionChatModel':
        """Get chat model with compliance wrapper"""
        return VeridionChatModel(
            model_name=model_name,
            veridion=self.veridion,
            **kwargs
        )
    
    def get_text_model(
        self,
        model_name: str = "text-bison",
        **kwargs
    ) -> 'VeridionTextModel':
        """Get text generation model with compliance wrapper"""
        return VeridionTextModel(
            model_name=model_name,
            veridion=self.veridion,
            **kwargs
        )
    
    async def close(self):
        """Close Veridion client"""
        await self.veridion.close()


class VeridionChatModel:
    """Wrapped ChatModel with compliance logging"""
    
    def __init__(
        self,
        model_name: str,
        veridion: VeridionClient,
        **kwargs
    ):
        if not VERTEX_AI_AVAILABLE:
            raise ImportError("google-cloud-aiplatform package is required")
        
        self.model = ChatModel.from_pretrained(model_name, **kwargs)
        self.model_name = model_name
        self.veridion = veridion
    
    def start_chat(self, **kwargs):
        """Start chat with compliance"""
        chat = self.model.start_chat(**kwargs)
        return VeridionChatSession(chat, self.model_name, self.veridion)
    
    def send_message(
        self,
        message: str,
        **kwargs
    ) -> str:
        """Send message with compliance logging"""
        start_time = time.time()
        
        try:
            response = self.model.send_message(message, **kwargs)
            inference_time_ms = int((time.time() - start_time) * 1000)
            
            # Log asynchronously
            import asyncio
            asyncio.create_task(self.veridion.log_action(
                action="vertex_ai_chat",
                payload=message,
                target_region="EU",
                inference_time_ms=inference_time_ms,
                system_id="gcp-vertex",
                model_name=self.model_name,
                hardware_type="CLOUD"
            ))
            
            return response.text
            
        except Exception as e:
            import asyncio
            asyncio.create_task(self.veridion.log_action(
                action="vertex_ai_chat_error",
                payload=f"Error: {str(e)}",
                target_region="EU"
            ))
            raise


class VeridionChatSession:
    """Chat session with compliance logging"""
    
    def __init__(self, chat, model_name: str, veridion: VeridionClient):
        self.chat = chat
        self.model_name = model_name
        self.veridion = veridion
    
    def send_message(self, message: str, **kwargs):
        """Send message in chat session"""
        start_time = time.time()
        
        try:
            response = self.chat.send_message(message, **kwargs)
            inference_time_ms = int((time.time() - start_time) * 1000)
            
            # Log asynchronously (fire and forget)
            import asyncio
            asyncio.create_task(self.veridion.log_action(
                action="vertex_ai_chat_session",
                payload=message,
                target_region="EU",
                inference_time_ms=inference_time_ms,
                system_id="gcp-vertex",
                model_name=self.model_name,
                hardware_type="CLOUD"
            ))
            
            return response
            
        except Exception as e:
            import asyncio
            asyncio.create_task(self.veridion.log_action(
                action="vertex_ai_chat_error",
                payload=f"Error: {str(e)}",
                target_region="EU"
            ))
            raise


class VeridionTextModel:
    """Wrapped TextGenerationModel with compliance logging"""
    
    def __init__(
        self,
        model_name: str,
        veridion: VeridionClient,
        **kwargs
    ):
        if not VERTEX_AI_AVAILABLE:
            raise ImportError("google-cloud-aiplatform package is required")
        
        self.model = TextGenerationModel.from_pretrained(model_name, **kwargs)
        self.model_name = model_name
        self.veridion = veridion
    
    def predict(self, prompt: str, **kwargs) -> str:
        """Predict with compliance logging"""
        start_time = time.time()
        
        try:
            response = self.model.predict(prompt, **kwargs)
            inference_time_ms = int((time.time() - start_time) * 1000)
            
            # Log asynchronously
            import asyncio
            asyncio.create_task(self.veridion.log_action(
                action="vertex_ai_text",
                payload=prompt,
                target_region="EU",
                inference_time_ms=inference_time_ms,
                system_id="gcp-vertex",
                model_name=self.model_name,
                hardware_type="CLOUD"
            ))
            
            return response.text
            
        except Exception as e:
            import asyncio
            asyncio.create_task(self.veridion.log_action(
                action="vertex_ai_text_error",
                payload=f"Error: {str(e)}",
                target_region="EU"
            ))
            raise

