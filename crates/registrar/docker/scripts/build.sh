#!/bin/bash

# Build base image
docker build -t synapse-base:latest docker/base

# Build all module images
for module in docker/modules/*; do
    if [ -d "$module" ]; then
        module_name=$(basename "$module")
        echo "Building module: $module_name"
        docker build -t "synapse-$module_name:latest" "$module"
        
        # Tag for GitHub packages
        docker tag "synapse-$module_name:latest" "ghcr.io/bakobiibizo/synapse-$module_name:latest"
    fi
done
