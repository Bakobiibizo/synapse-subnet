#!/usr/bin/env python3

import sys
import requests

def check_health():
    try:
        response = requests.get("http://localhost:8000/health")
        if response.status_code == 200:
            health_data = response.json()
            if health_data["status"] == "Healthy":
                return True
        return False
    except:
        return False

if __name__ == "__main__":
    sys.exit(0 if check_health() else 1)
