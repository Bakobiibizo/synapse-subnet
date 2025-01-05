# Base image for all Synapse modules
FROM ubuntu:22.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    python3.10 \
    python3-pip \
    curl \
    git \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies
RUN pip3 install --no-cache-dir \
    torch \
    numpy \
    transformers \
    accelerate

# Create module directory structure
RUN mkdir -p /opt/synapse/module
WORKDIR /opt/synapse/module

# Copy module interface requirements
COPY requirements.txt /opt/synapse/
RUN pip3 install -r /opt/synapse/requirements.txt

# Module interface script
COPY module_interface.py /opt/synapse/

# Health check script
COPY healthcheck.py /opt/synapse/
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD python3 /opt/synapse/healthcheck.py

# Environment variables
ENV MODULE_PATH=/opt/synapse/module
ENV PYTHONPATH=/opt/synapse:$PYTHONPATH

# Default command
CMD ["python3", "/opt/synapse/module_interface.py"]
