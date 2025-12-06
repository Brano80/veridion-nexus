"""Setup script for Veridion Nexus SDKs"""
from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="veridion-nexus-sdks",
    version="0.1.0",
    author="Veridion Nexus",
    author_email="support@veridion.nexus",
    description="Veridion Nexus SDKs for AI platforms - Compliance integration",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/veridion-nexus/sdks",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
    python_requires=">=3.8",
    install_requires=[
        "httpx>=0.24.0",
    ],
    extras_require={
        "azure": [
            "azure-ai-inference>=1.0.0",
            "azure-core>=1.29.0",
        ],
        "aws": [
            "boto3>=1.28.0",
        ],
        "gcp": [
            "google-cloud-aiplatform>=1.38.0",
        ],
        "langchain": [
            "langchain>=0.1.0",
        ],
        "openai": [
            "openai>=1.0.0",
        ],
        "huggingface": [
            "transformers>=4.30.0",
            "torch>=2.0.0",
        ],
        "all": [
            "azure-ai-inference>=1.0.0",
            "azure-core>=1.29.0",
            "boto3>=1.28.0",
            "google-cloud-aiplatform>=1.38.0",
            "langchain>=0.1.0",
            "openai>=1.0.0",
            "transformers>=4.30.0",
            "torch>=2.0.0",
        ],
    },
)

