#!/usr/bin/env bash                    
set -e                                  

# Unpack all the files
# tar xzf project.tar.gz
   
# activate the venv
source env/bin/activate                   

# run the Python analysis script
for cfg in configurations/*.json; do
    python VeloPix_HyperParamHandler.py --config "$cfg"
    mv result.json output/result_$(basename "$cfg")
done
# deactivate venv
deactivate                                
