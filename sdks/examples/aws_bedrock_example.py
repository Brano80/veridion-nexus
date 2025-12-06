"""Example: Using Veridion Nexus with AWS Bedrock"""
import json
import os
from sdks.aws_bedrock import VeridionBedrock

def main():
    # Initialize Veridion-wrapped Bedrock client
    # Note: Must use EU region (eu-west-1, eu-central-1, etc.)
    bedrock = VeridionBedrock(
        region_name="eu-west-1",  # EU region required
        veridion_api_url=os.getenv("VERIDION_API_URL", "http://localhost:8080"),
        veridion_api_key=os.getenv("VERIDION_API_KEY"),
        agent_id="aws-bedrock-example"
    )
    
    try:
        # Invoke Bedrock model - compliance is automatic
        response = bedrock.invoke_model(
            model_id="anthropic.claude-v2",
            body={
                "prompt": "\n\nHuman: What is GDPR?\n\nAssistant:",
                "max_tokens_to_sample": 100
            }
        )
        
        print("Response:", response.get("completion", ""))
        
    except ValueError as e:
        if "SOVEREIGN_LOCK_VIOLATION" in str(e):
            print("ERROR: Non-EU region detected. Must use EU regions.")
        else:
            raise
    
    finally:
        import asyncio
        asyncio.run(bedrock.close())

if __name__ == "__main__":
    main()

