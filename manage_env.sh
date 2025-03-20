#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 [install|update]"
    exit 1
fi

action=$1
venv_dir="venv"

os=$(uname)
if [[ "$os" == "Darwin" || "$os" == "Linux" ]]; then
    activate_script="$venv_dir/bin/activate"
    python_cmd="python3"
else
    # assume Windows (Git Bash)
    activate_script="$venv_dir/Scripts/activate"
    python_cmd="python"
fi

if [ "$action" == "install" ]; then
    # If a venv already exists, remove it
    if [ -d "$venv_dir" ]; then
        echo "Removing existing virtual environment..."
        rm -rf "$venv_dir"
    fi

    # check if Rust is installed by looking for rustc
    if ! command -v rustc &> /dev/null; then
        echo "Rust is not installed. Installing Rust..."
        if [ "$os" == "Darwin" ]; then
            brew install rust
        else
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
            if [ -f "$HOME/.cargo/env" ]; then
                echo "Sourcing Rust environment..."
                source "$HOME/.cargo/env"
            else
                echo "Warning: $HOME/.cargo/env not found. Please check your rustup installation."
            fi
        fi
    else
        echo "Rust is already installed. Skipping Rust installation."
    fi

    echo "Creating a new virtual environment..."
    $python_cmd -m venv "$venv_dir"

    echo "Activating virtual environment..."
    # shellcheck disable=SC1091
    source "$activate_script"

    export PATH="$HOME/.cargo/bin:$PATH"

    echo "Upgrading pip..."
    pip install --upgrade pip

    echo "Installing dependencies from requirements.txt..."
    pip install -r requirements.txt

    cd VeloPix
    maturin develop --release
    cd ..

elif [ "$action" == "update" ]; then
    if [ ! -d "$venv_dir" ]; then
        echo "Virtual environment not found. Please run '$0 install' first."
        exit 1
    fi

    echo "Activating virtual environment..."
    # shellcheck disable=SC1091
    source "$activate_script"

    export PATH="$HOME/.cargo/bin:$PATH"

    echo "Upgrading pip..."
    pip install --upgrade pip

    echo "Updating dependencies from requirements.txt..."
    pip install --upgrade -r requirements.txt

    cd VeloPix
    maturin develop --release
    cd ..

else
    echo "Invalid argument: $action"
    echo "Usage: $0 [install|update]"
    exit 1
fi