#!/bin/bash

# Check if an argument was provided
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 [install|update]"
    exit 1
fi

action=$1
venv_dir="venv"

if [ "$action" == "install" ]; then
    # If a venv already exists, remove it
    if [ -d "$venv_dir" ]; then
        echo "Removing existing virtual environment..."
        rm -rf "$venv_dir"
    fi

    echo "Creating a new virtual environment..."
    python3 -m venv "$venv_dir"

    echo "Activating virtual environment..."
    # shellcheck disable=SC1091
    source "$venv_dir/bin/activate"

    echo "Upgrading pip..."
    pip install --upgrade pip

    echo "Installing dependencies from requirements.txt..."
    pip install -r requirements.txt

    cd rust_algorithms
    maturin develop
    cd ..

elif [ "$action" == "update" ]; then
    # Ensure the virtual environment exists
    if [ ! -d "$venv_dir" ]; then
        echo "Virtual environment not found. Please run '$0 install' first."
        exit 1
    fi

    echo "Activating virtual environment..."
    # shellcheck disable=SC1091
    source "$venv_dir/bin/activate"

    echo "Upgrading pip..."
    pip install --upgrade pip

    echo "Updating dependencies from requirements.txt..."
    pip install --upgrade -r requirements.txt

    cd rust_algorithms
    maturin develop
    cd ..

else
    echo "Invalid argument: $action"
    echo "Usage: $0 [install|update]"
    exit 1
fi