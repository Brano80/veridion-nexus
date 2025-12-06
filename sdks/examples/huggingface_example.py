"""Example: Using Veridion Nexus with HuggingFace Pipelines"""
import os
from sdks.huggingface import create_veridion_pipeline

def main():
    # Create a Veridion-wrapped HuggingFace pipeline
    pipeline = create_veridion_pipeline(
        task="text-generation",
        model="gpt2",  # Use a smaller model for example
        device=-1,  # -1 for CPU, 0+ for GPU
        veridion_api_url=os.getenv("VERIDION_API_URL", "http://localhost:8080"),
        veridion_api_key=os.getenv("VERIDION_API_KEY"),
        agent_id="huggingface-example"
    )
    
    # Use it normally - compliance is automatic
    result = pipeline("Hello, how are you?", max_length=50)
    print("Response:", result)

if __name__ == "__main__":
    main()

