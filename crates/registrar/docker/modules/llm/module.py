import os
import time
import psutil
import requests
from typing import Dict, Any

class Module:
    def __init__(self):
        self.model_name = "llama2"
        self.requests_processed = 0
        self.total_latency = 0
        self.error_count = 0
        self.start_time = time.time()

    async def initialize(self) -> None:
        """Initialize the Ollama module"""
        # Ensure Ollama service is running
        os.system("ollama serve &")
        time.sleep(5)  # Wait for Ollama to start

    async def health_check(self) -> Dict[str, str]:
        """Check if Ollama is responding"""
        try:
            response = requests.post("http://localhost:11434/api/generate",
                                  json={"model": self.model_name, "prompt": "test"})
            if response.status_code == 200:
                return {"status": "Healthy", "message": "Ollama is responding"}
            return {"status": "Unhealthy", "message": "Ollama returned error"}
        except Exception as e:
            return {"status": "Unhealthy", "message": str(e)}

    def get_capabilities(self) -> Dict[str, Any]:
        """Get module capabilities"""
        return {
            "name": "ollama-llm",
            "version": "1.0",
            "model_type": "llm",
            "max_batch_size": 1,
            "max_sequence_length": 4096,
            "resource_requirements": {
                "min_memory_mb": 8192,
                "min_cpu_cores": 2.0,
                "gpu_required": False,
                "min_gpu_memory_mb": None
            }
        }

    async def run_inference(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Run inference using Ollama"""
        start_time = time.time()
        try:
            response = requests.post(
                "http://localhost:11434/api/generate",
                json={
                    "model": self.model_name,
                    "prompt": input_data["text"],
                    "options": {
                        "temperature": input_data["parameters"]["temperature"],
                        "top_p": input_data["parameters"]["top_p"],
                        "stop": input_data["parameters"]["stop_sequences"]
                    }
                }
            )
            response.raise_for_status()
            result = response.json()

            self.requests_processed += 1
            self.total_latency += (time.time() - start_time) * 1000

            return {
                "text": result["response"],
                "usage": {
                    "prompt_tokens": result.get("prompt_eval_count", 0),
                    "completion_tokens": result.get("eval_count", 0),
                    "total_tokens": result.get("prompt_eval_count", 0) + result.get("eval_count", 0)
                }
            }
        except Exception as e:
            self.error_count += 1
            raise Exception(f"Inference failed: {str(e)}")

    def get_metrics(self) -> Dict[str, Any]:
        """Get module metrics"""
        process = psutil.Process()
        return {
            "requests_processed": self.requests_processed,
            "average_latency_ms": self.total_latency / max(1, self.requests_processed),
            "error_count": self.error_count,
            "memory_usage_mb": process.memory_info().rss / 1024 / 1024
        }
