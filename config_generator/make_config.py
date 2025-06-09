#!/usr/bin/env python3
import argparse
import json
import itertools
from typing import Any
from uuid import uuid4

def generate_numeric_range(min_val: float, max_val: float, steps: int) -> list[float]:
    """Generate evenly spaced values between min and max (inclusive)."""
    step_size = (max_val - min_val) / (steps - 1) if steps > 1 else 0
    return [round(min_val + j * step_size, 3) for j in range(steps)]

def generate_weight_combinations(
    minw: tuple[float, float, float, float],
    maxw: tuple[float, float, float, float], 
    steps: int,
    weight_count: int
) -> list[list[float]]:
    """Generate combinations of weights within the specified ranges."""
    ranges: list[list[float]] = []
    for i in range(weight_count):
        min_val, max_val = minw[i], maxw[i]
        ranges.append(generate_numeric_range(min_val, max_val, steps))
    
    # Generate all possible combinations
    combinations = list(itertools.product(*ranges))
    return [list(combo) for combo in combinations]

def generate_param_combinations(param_specs: dict[str, dict[str, Any]]) -> list[dict[str, Any]]:
    """Generate all combinations of optimizer parameters based on specifications."""
    # Extract parameter values for each parameter
    param_values: dict[str, Any] = {}
    
    for param_name, spec in param_specs.items():
        if spec["type"] == "num":
            param_values[param_name] = generate_numeric_range(
                spec["min"], spec["max"], spec["steps"])
        elif spec["type"] == "cat":
            param_values[param_name] = spec["values"]
    
    # Generate combinations
    param_names = list(param_values.keys())
    value_combinations = itertools.product(*(param_values[name] for name in param_names))
    
    result: list[Any] = []
    for values in value_combinations:
        result.append({name: value for name, value in zip(param_names, values)})
    
    return result

def main():
    parser = argparse.ArgumentParser(description='Generate configuration files for optimization')
    parser.add_argument('--algo', type=str, choices=['TF', 'GDFS', 'ST'],
                        help='Reconstruction algorithm')
    parser.add_argument('--minw', type=float, nargs=4, 
                        help='Minimum values for weights (four float values)')
    parser.add_argument('--maxw', type=float, nargs=4,
                        help='Maximum values for weights (four float values)')
    parser.add_argument('--solver', type=str, choices=['GridSearch', 'ParticleSwarm', 'Bayesian', 'PolyHoot'],
                        help='Optimization solver')
    parser.add_argument('--steps', type=int, default=3,
                        help='Number of steps for each weight (default: 3)')
    
    # Add optimizer parameter arguments
    parser.add_argument('--params', type=str, default=None,
                        help='JSON file with optimizer parameter specifications')
    
    args = parser.parse_args()
    
    # Load base configuration for the specified solver
    base_config_file = f"{args.solver}_base.json"
    try:
        with open(base_config_file, 'r') as f:
            base_config = json.load(f)
    except FileNotFoundError:
        print(f"Error: Base configuration file '{base_config_file}' not found.")
        return
    
    # Determine the number of weights needed
    weight_count = 4  # Default
    
    # Generate weight combinations
    weight_combinations = generate_weight_combinations(
        tuple(args.minw),
        tuple(args.maxw),
        args.steps,
        weight_count
    )
    
    # Load parameter specifications if provided
    param_combinations: list[Any] = [{}]  # Default: just one combination with no custom params
    if args.params:
        try:
            with open(args.params, 'r') as f:
                param_specs = json.load(f)
            param_combinations = generate_param_combinations(param_specs)
        except FileNotFoundError:
            print(f"Warning: Parameter configuration file '{args.params}' not found.")
            print("Proceeding with weight combinations only.")
    
    # Generate config files for each combination of weights and parameters
    file_counter = 1
    for weights in weight_combinations:
        for params in param_combinations:
            # Create a deep copy of the base config
            config = json.loads(json.dumps(base_config))
            
            # Set required fields
            config["solverName"] = args.solver
            config["reconstruction_algo"] = args.algo
            
            # Ensure optimizer exists 
            if "optimizer" not in config:
                config["optimizer"] = {}
            
            # Set weights
            config["optimizer"]["weights"] = weights
            
            # Set additional optimizer parameters
            for param_name, param_value in params.items():
                config["optimizer"][param_name] = param_value
            
            # Create output file name
            output_file = f"output/{args.solver}_{args.algo}_{file_counter}_{uuid4()}.json"
            
            # Save configuration to file
            with open(output_file, 'w') as f:
                json.dump(config, f, indent=4)
            
            print(f"Generated: {output_file}")
            file_counter += 1
    
    total_configs = len(weight_combinations) * len(param_combinations)
    print(f"Successfully generated {total_configs} configuration files.")

if __name__ == "__main__":
    main()