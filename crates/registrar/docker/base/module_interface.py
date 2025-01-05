#!/usr/bin/env python3

import abc
import json
import os
from dataclasses import dataclass
from typing import Dict, List, Optional
from fastapi import FastAPI, HTTPException
import uvicorn

app = FastAPI()

@dataclass
class Health:
    status: str
    message: str

@dataclass
class ResourceRequirements:
    min_memory_mb: int
    min_cpu_cores: float
    gpu_required: bool
    min_gpu_memory_mb: Optional[int] = None

@dataclass
class ModuleCapabilities:
    name: str
    version: str
    model_type: str
    max_batch_size: int
    max_sequence_length: int
    resource_requirements: ResourceRequirements

@dataclass
class InferenceParameters:
    max_tokens: int
    temperature: float
    top_p: float
    stop_sequences: List[str]

@dataclass
class Input:
    text: str
    parameters: InferenceParameters

@dataclass
class TokenUsage:
    prompt_tokens: int
    completion_tokens: int
    total_tokens: int

@dataclass
class Output:
    text: str
    usage: TokenUsage

@dataclass
class MetricsData:
    requests_processed: int
    average_latency_ms: float
    error_count: int
    memory_usage_mb: int

class ModuleInterface(abc.ABC):
    @abc.abstractmethod
    async def initialize(self) -> None:
        """Initialize the module"""
        pass

    @abc.abstractmethod
    async def health_check(self) -> Health:
        """Check module health"""
        pass

    @abc.abstractmethod
    def get_capabilities(self) -> ModuleCapabilities:
        """Get module capabilities"""
        pass

    @abc.abstractmethod
    async def run_inference(self, input_data: Input) -> Output:
        """Run inference"""
        pass

    @abc.abstractmethod
    def get_metrics(self) -> MetricsData:
        """Get module metrics"""
        pass

# Load the specific module implementation
module_path = os.getenv('MODULE_PATH', '/opt/synapse/module')
module_file = os.path.join(module_path, 'module.py')

if not os.path.exists(module_file):
    raise FileNotFoundError(f"Module implementation not found at {module_file}")

# Import the module implementation
import importlib.util
spec = importlib.util.spec_from_file_location("module", module_file)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

# Initialize module
module_instance = module.Module()

@app.on_event("startup")
async def startup():
    await module_instance.initialize()

@app.get("/health")
async def health():
    return await module_instance.health_check()

@app.get("/capabilities")
def capabilities():
    return module_instance.get_capabilities()

@app.post("/inference")
async def inference(input_data: Input):
    try:
        return await module_instance.run_inference(input_data)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/metrics")
def metrics():
    return module_instance.get_metrics()

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)
