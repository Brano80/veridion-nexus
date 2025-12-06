"""Example: Using Veridion Nexus with GCP Vertex AI"""
import os
from sdks.gcp_vertex import VeridionVertexAI

def main():
    # Initialize Veridion-wrapped Vertex AI client
    # Note: Must use EU region (europe-west1, europe-west4, etc.)
    vertex = VeridionVertexAI(
        project=os.getenv("GCP_PROJECT", "your-project"),
        location="europe-west1",  # EU region required
        veridion_api_url=os.getenv("VERIDION_API_URL", "http://localhost:8080"),
        veridion_api_key=os.getenv("VERIDION_API_KEY"),
        agent_id="gcp-vertex-example"
    )
    
    try:
        # Get chat model - compliance is automatic
        chat_model = vertex.get_chat_model(model_name="chat-bison")
        response = chat_model.send_message("What is GDPR?")
        print("Response:", response)
        
        # Or use text generation model
        text_model = vertex.get_text_model(model_name="text-bison")
        response = text_model.predict("Explain GDPR in one sentence.")
        print("Response:", response)
        
    except ValueError as e:
        if "SOVEREIGN_LOCK_VIOLATION" in str(e):
            print("ERROR: Non-EU region detected. Must use EU regions.")
        else:
            raise
    
    finally:
        import asyncio
        asyncio.run(vertex.close())

if __name__ == "__main__":
    main()

