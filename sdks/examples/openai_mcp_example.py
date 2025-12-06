"""Example: Using Veridion Nexus with OpenAI MCP"""
import asyncio
import os
from sdks.openai_mcp import VeridionOpenAIMCP

async def main():
    # Initialize Veridion-wrapped OpenAI MCP client
    client = VeridionOpenAIMCP(
        api_key=os.getenv("OPENAI_API_KEY", "your-openai-key"),
        veridion_api_url=os.getenv("VERIDION_API_URL", "http://localhost:8080"),
        veridion_api_key=os.getenv("VERIDION_API_KEY"),
        agent_id="openai-mcp-example"
    )
    
    try:
        # Create chat completion - compliance is automatic
        response = await client.chat_completions_create(
            model="gpt-4",
            messages=[
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": "What is GDPR?"}
            ]
        )
        
        print("Response:", response.choices[0].message.content)
        
        # Or use streaming
        print("\nStreaming response:")
        async for chunk in client.chat_completions_create_stream(
            model="gpt-4",
            messages=[{"role": "user", "content": "Explain GDPR briefly."}]
        ):
            if chunk.choices[0].delta.content:
                print(chunk.choices[0].delta.content, end="", flush=True)
        
    finally:
        await client.close()

if __name__ == "__main__":
    asyncio.run(main())

