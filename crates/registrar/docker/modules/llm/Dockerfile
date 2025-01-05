# Use the base Synapse module image
FROM synapse-base:latest

# Install Ollama
RUN curl -fsSL https://ollama.ai/install.sh | sh

# Copy module implementation
COPY module.py /opt/synapse/module/
COPY requirements.txt /opt/synapse/module/

# Install module-specific requirements
RUN pip3 install -r /opt/synapse/module/requirements.txt

# Pull the default model
RUN ollama pull llama2

# Expose the API port
EXPOSE 8000

# Default command inherited from base image
