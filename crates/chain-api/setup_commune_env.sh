#!/bin/bash

# Create virtual environment if it doesn't exist
if [ ! -d ".commune-env" ]; then
    python3 -m venv .commune-env
fi

# Activate virtual environment
source .commune-env/bin/activate

# Install required packages
pip install communex substrate-interface

# Deactivate virtual environment
deactivate
