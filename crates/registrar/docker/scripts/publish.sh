#!/bin/bash

# Ensure GitHub token is set
if [ -z "$GITHUB_TOKEN" ]; then
    echo "Error: GITHUB_TOKEN environment variable not set"
    exit 1
fi

# Login to GitHub Container Registry
echo $GITHUB_TOKEN | docker login ghcr.io -u bakobiibizo --password-stdin

# Push all module images
for module in docker/modules/*; do
    if [ -d "$module" ]; then
        module_name=$(basename "$module")
        echo "Publishing module: $module_name"
        docker push "ghcr.io/bakobiibizo/synapse-$module_name:latest"
    fi
done
