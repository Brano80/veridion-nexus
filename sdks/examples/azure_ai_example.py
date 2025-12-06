"""Example: Using Veridion Nexus with Azure AI"""
import asyncio
import os
from sdks.azure_ai import VeridionAzureAI
from azure.core.credentials import AzureKeyCredential

async def main():
    # Initialize Veridion-wrapped Azure AI client
    client = VeridionAzureAI(
        endpoint=os.getenv("AZURE_AI_ENDPOINT", "https://your-endpoint.openai.azure.com/"),
        credential=AzureKeyCredential(os.getenv("AZURE_AI_KEY", "your-key")),
        veridion_api_url=os.getenv("VERIDION_API_URL", "http://localhost:8080"),
        veridion_api_key=os.getenv("VERIDION_API_KEY"),
        agent_id="azure-ai-example"
    )
    
    try:
        # Use Azure AI - compliance is automatic
        response = await client.complete(
            messages=[
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": "What is GDPR?"}
            ],
            model="gpt-4"
        )
        
        print("Response:", response)
        
    finally:
        await client.close()

if __name__ == "__main__":
    asyncio.run(main())

