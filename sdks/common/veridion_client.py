"""Base client for Veridion Nexus API integration"""
import httpx
import os
from typing import Optional, Dict, Any
from datetime import datetime


class VeridionClient:
    """Base client for Veridion Nexus API integration"""
    
    def __init__(
        self,
        api_url: Optional[str] = None,
        api_key: Optional[str] = None,
        agent_id: Optional[str] = None
    ):
        self.api_url = api_url or os.getenv("VERIDION_API_URL", "http://localhost:8080")
        self.api_key = api_key or os.getenv("VERIDION_API_KEY")
        self.agent_id = agent_id or os.getenv("VERIDION_AGENT_ID", "default-agent")
        self.client = httpx.AsyncClient(timeout=30.0)
    
    async def log_action(
        self,
        action: str,
        payload: str,
        target_region: str = "EU",
        user_id: Optional[str] = None,
        requires_human_oversight: bool = False,
        inference_time_ms: Optional[int] = None,
        gpu_power_rating_watts: Optional[float] = None,
        cpu_power_rating_watts: Optional[float] = None,
        system_id: Optional[str] = None,
        model_name: Optional[str] = None,
        model_version: Optional[str] = None,
        hardware_type: Optional[str] = None,
        **kwargs
    ) -> Dict[str, Any]:
        """Log an action to Veridion Nexus"""
        
        headers = {}
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        
        data = {
            "agent_id": self.agent_id,
            "action": action,
            "payload": payload,
            "target_region": target_region,
            "user_id": user_id,
            "requires_human_oversight": requires_human_oversight,
            "inference_time_ms": inference_time_ms,
            "gpu_power_rating_watts": gpu_power_rating_watts,
            "cpu_power_rating_watts": cpu_power_rating_watts,
            "system_id": system_id,
            "model_name": model_name,
            "model_version": model_version,
            "hardware_type": hardware_type,
            **kwargs
        }
        
        response = await self.client.post(
            f"{self.api_url}/api/v1/log_action",
            json=data,
            headers=headers
        )
        
        if response.status_code == 403:
            raise ValueError("SOVEREIGN_LOCK_VIOLATION: Action blocked due to data sovereignty requirements")
        
        response.raise_for_status()
        return response.json()
    
    async def close(self):
        """Close the HTTP client"""
        await self.client.aclose()
    
    async def __aenter__(self):
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.close()

