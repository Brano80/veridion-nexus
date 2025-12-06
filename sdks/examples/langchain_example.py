"""Example: Using Veridion Nexus with LangChain"""
import os
from sdks.langchain import wrap_langchain_llm

# Try to import LangChain LLM (adjust based on your LangChain version)
try:
    from langchain.llms import OpenAI
except ImportError:
    try:
        from langchain_openai import OpenAI
    except ImportError:
        print("Please install langchain: pip install langchain")
        exit(1)

def main():
    # Create your LangChain LLM
    llm = OpenAI(
        temperature=0.7,
        openai_api_key=os.getenv("OPENAI_API_KEY")
    )
    
    # Wrap it with Veridion compliance
    veridion_llm = wrap_langchain_llm(
        llm=llm,
        veridion_api_url=os.getenv("VERIDION_API_URL", "http://localhost:8080"),
        veridion_api_key=os.getenv("VERIDION_API_KEY"),
        agent_id="langchain-example"
    )
    
    # Use it normally - compliance is automatic
    response = veridion_llm("What is GDPR?")
    print("Response:", response)

if __name__ == "__main__":
    main()

