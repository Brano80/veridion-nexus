"""HuggingFace Pipeline wrapper with Veridion Nexus compliance"""
from typing import Optional, Dict, Any, List, Union
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
    from transformers import pipeline, Pipeline
    import torch
    TRANSFORMERS_AVAILABLE = True
except ImportError:
    TRANSFORMERS_AVAILABLE = False
    # Create dummy classes for type hints
    class Pipeline:
        pass
    torch = None


class VeridionHuggingFacePipeline:
    """HuggingFace Pipeline wrapper with Veridion Nexus compliance"""
    
    def __init__(
        self,
        task: str,
        model: Optional[str] = None,
        device: int = -1,  # -1 for CPU, 0+ for GPU
        veridion_api_url: Optional[str] = None,
        veridion_api_key: Optional[str] = None,
        agent_id: Optional[str] = None,
        **kwargs
    ):
        if not TRANSFORMERS_AVAILABLE:
            raise ImportError(
                "transformers package is required. "
                "Install it with: pip install transformers torch"
            )
        
        # Create HuggingFace pipeline
        self.pipeline = pipeline(
            task=task,
            model=model,
            device=device,
            **kwargs
        )
        
        self.veridion = VeridionClient(
            api_url=veridion_api_url,
            api_key=veridion_api_key,
            agent_id=agent_id or "huggingface-agent"
        )
        
        self.task = task
        self.model_name = model or "default"
        self.device = device
        self.hardware_type = "GPU" if device >= 0 else "CPU"
    
    def __call__(self, inputs: Union[str, List[str], Dict], **kwargs) -> Any:
        """Call pipeline with compliance logging"""
        import asyncio
        start_time = time.time()
        
        # Determine power rating based on device
        gpu_power = 250.0 if self.device >= 0 else None
        cpu_power = 100.0 if self.device < 0 else None
        
        try:
            # Call pipeline
            result = self.pipeline(inputs, **kwargs)
            
            inference_time_ms = int((time.time() - start_time) * 1000)
            
            # Log to Veridion Nexus
            payload = str(inputs) if not isinstance(inputs, str) else inputs
            # Limit payload size to avoid issues
            if len(payload) > 1000:
                payload = payload[:1000] + "..."
            
            asyncio.create_task(self.veridion.log_action(
                action=f"huggingface_{self.task}",
                payload=payload,
                target_region="EU",
                inference_time_ms=inference_time_ms,
                gpu_power_rating_watts=gpu_power,
                cpu_power_rating_watts=cpu_power,
                system_id="huggingface",
                model_name=self.model_name,
                hardware_type=self.hardware_type
            ))
            
            return result
            
        except Exception as e:
            import asyncio
            asyncio.create_task(self.veridion.log_action(
                action=f"huggingface_{self.task}_error",
                payload=f"Error: {str(e)}",
                target_region="EU"
            ))
            raise
    
    async def close(self):
        """Close Veridion client"""
        await self.veridion.close()


def create_veridion_pipeline(
    task: str,
    model: Optional[str] = None,
    device: int = -1,
    veridion_api_url: Optional[str] = None,
    veridion_api_key: Optional[str] = None,
    agent_id: Optional[str] = None,
    **kwargs
) -> VeridionHuggingFacePipeline:
    """Convenience function to create a Veridion-wrapped HuggingFace pipeline"""
    return VeridionHuggingFacePipeline(
        task=task,
        model=model,
        device=device,
        veridion_api_url=veridion_api_url,
        veridion_api_key=veridion_api_key,
        agent_id=agent_id,
        **kwargs
    )

