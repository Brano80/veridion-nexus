"""LangChain wrapper with Veridion Nexus compliance integration"""
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
    from langchain.llms.base import LLM
    from langchain.callbacks.manager import CallbackManagerForLLMRun
    LANGCHAIN_AVAILABLE = True
except ImportError:
    LANGCHAIN_AVAILABLE = False
    # Create dummy classes for type hints
    class LLM:
        pass
    class CallbackManagerForLLMRun:
        pass


class VeridionLangChainWrapper:
    """Wrapper for LangChain LLMs with Veridion Nexus compliance"""
    
    def __init__(
        self,
        llm: Any,  # LLM
        veridion_api_url: Optional[str] = None,
        veridion_api_key: Optional[str] = None,
        agent_id: Optional[str] = None
    ):
        if not LANGCHAIN_AVAILABLE:
            raise ImportError(
                "langchain package is required. "
                "Install it with: pip install langchain"
            )
        
        self.llm = llm
        self.veridion = VeridionClient(
            api_url=veridion_api_url,
            api_key=veridion_api_key,
            agent_id=agent_id or "langchain-agent"
        )
    
    def __call__(
        self,
        prompt: str,
        stop: Optional[List[str]] = None,
        run_manager: Optional[Any] = None,  # CallbackManagerForLLMRun
        **kwargs
    ) -> str:
        """Call LLM with compliance logging"""
        import asyncio
        start_time = time.time()
        
        try:
            # Call underlying LLM
            response = self.llm(prompt, stop=stop, run_manager=run_manager, **kwargs)
            
            inference_time_ms = int((time.time() - start_time) * 1000)
            
            # Log to Veridion Nexus
            asyncio.create_task(self.veridion.log_action(
                action="langchain_llm_call",
                payload=prompt,
                target_region="EU",
                inference_time_ms=inference_time_ms,
                system_id="langchain",
                model_name=getattr(self.llm, 'model_name', getattr(self.llm, '_llm_type', 'unknown')),
                hardware_type="CLOUD"
            ))
            
            return response
            
        except Exception as e:
            import asyncio
            asyncio.create_task(self.veridion.log_action(
                action="langchain_llm_error",
                payload=f"Error: {str(e)}",
                target_region="EU"
            ))
            raise
    
    async def ainvoke(
        self,
        input: str,
        config: Optional[Dict] = None,
        **kwargs
    ) -> str:
        """Async invoke with compliance logging"""
        start_time = time.time()
        
        try:
            # Call underlying async method
            if hasattr(self.llm, 'ainvoke'):
                response = await self.llm.ainvoke(input, config=config, **kwargs)
            else:
                response = self.llm(input, **kwargs)
            
            inference_time_ms = int((time.time() - start_time) * 1000)
            
            await self.veridion.log_action(
                action="langchain_async_invoke",
                payload=input,
                target_region="EU",
                inference_time_ms=inference_time_ms,
                system_id="langchain",
                model_name=getattr(self.llm, 'model_name', getattr(self.llm, '_llm_type', 'unknown')),
                hardware_type="CLOUD"
            )
            
            return response
            
        except Exception as e:
            await self.veridion.log_action(
                action="langchain_async_error",
                payload=f"Error: {str(e)}",
                target_region="EU"
            )
            raise
    
    async def close(self):
        """Close Veridion client"""
        await self.veridion.close()


def wrap_langchain_llm(
    llm: Any,  # LLM
    veridion_api_url: Optional[str] = None,
    veridion_api_key: Optional[str] = None,
    agent_id: Optional[str] = None
) -> VeridionLangChainWrapper:
    """Convenience function to wrap a LangChain LLM"""
    return VeridionLangChainWrapper(
        llm=llm,
        veridion_api_url=veridion_api_url,
        veridion_api_key=veridion_api_key,
        agent_id=agent_id
    )

