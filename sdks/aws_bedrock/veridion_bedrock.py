"""AWS Bedrock client with Veridion Nexus compliance integration"""
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
    import boto3
    from botocore.exceptions import ClientError
    BOTO3_AVAILABLE = True
except ImportError:
    BOTO3_AVAILABLE = False
    ClientError = Exception


class VeridionBedrock:
    """AWS Bedrock client with Veridion Nexus compliance integration"""
    
    def __init__(
        self,
        region_name: str = "eu-west-1",  # Default to EU region
        veridion_api_url: Optional[str] = None,
        veridion_api_key: Optional[str] = None,
        agent_id: Optional[str] = None,
        **kwargs
    ):
        if not BOTO3_AVAILABLE:
            raise ImportError(
                "boto3 package is required. "
                "Install it with: pip install boto3"
            )
        
        # Check region - block non-EU regions
        if not region_name.startswith("eu-"):
            raise ValueError(
                "SOVEREIGN_LOCK_VIOLATION: "
                "AWS Bedrock must use EU regions (eu-west-1, eu-central-1, etc.)"
            )
        
        self.bedrock_runtime = boto3.client(
            'bedrock-runtime',
            region_name=region_name,
            **kwargs
        )
        self.veridion = VeridionClient(
            api_url=veridion_api_url,
            api_key=veridion_api_key,
            agent_id=agent_id or "aws-bedrock-agent"
        )
        self.region = region_name
    
    def invoke_model(
        self,
        model_id: str,
        body: Dict[str, Any],
        **kwargs
    ) -> Dict[str, Any]:
        """Invoke Bedrock model with compliance logging"""
        
        # Check region - block non-EU regions
        if not self.region.startswith("eu-"):
            raise ValueError(
                "SOVEREIGN_LOCK_VIOLATION: "
                "AWS Bedrock must use EU regions (eu-west-1, eu-central-1, etc.)"
            )
        
        start_time = time.time()
        payload = json.dumps(body)
        
        try:
            # Invoke Bedrock
            response = self.bedrock_runtime.invoke_model(
                modelId=model_id,
                body=json.dumps(body),
                **kwargs
            )
            
            response_body = json.loads(response['body'].read())
            inference_time_ms = int((time.time() - start_time) * 1000)
            
            # Log to Veridion Nexus (async, fire and forget)
            import asyncio
            asyncio.create_task(self.veridion.log_action(
                action="aws_bedrock_invoke",
                payload=payload,
                target_region="EU",
                inference_time_ms=inference_time_ms,
                system_id="aws-bedrock",
                model_name=model_id,
                hardware_type="CLOUD"
            ))
            
            return response_body
            
        except ClientError as e:
            import asyncio
            asyncio.create_task(self.veridion.log_action(
                action="aws_bedrock_error",
                payload=f"Error: {str(e)}",
                target_region="EU"
            ))
            raise
    
    def invoke_model_with_response_stream(
        self,
        model_id: str,
        body: Dict[str, Any],
        **kwargs
    ):
        """Invoke Bedrock with streaming and compliance logging"""
        
        if not self.region.startswith("eu-"):
            raise ValueError("SOVEREIGN_LOCK_VIOLATION: Must use EU regions")
        
        start_time = time.time()
        payload = json.dumps(body)
        
        try:
            # Stream from Bedrock
            response = self.bedrock_runtime.invoke_model_with_response_stream(
                modelId=model_id,
                body=json.dumps(body),
                **kwargs
            )
            
            chunks = []
            for event in response['body']:
                chunks.append(event)
                yield event
            
            # Log after streaming
            inference_time_ms = int((time.time() - start_time) * 1000)
            import asyncio
            asyncio.create_task(self.veridion.log_action(
                action="aws_bedrock_stream",
                payload=payload,
                target_region="EU",
                inference_time_ms=inference_time_ms,
                system_id="aws-bedrock",
                model_name=model_id,
                hardware_type="CLOUD"
            ))
            
        except ClientError as e:
            import asyncio
            asyncio.create_task(self.veridion.log_action(
                action="aws_bedrock_stream_error",
                payload=f"Error: {str(e)}",
                target_region="EU"
            ))
            raise
    
    async def close(self):
        """Close Veridion client"""
        await self.veridion.close()

