@echo off
setlocal enabledelayedexpansion

echo Starting execution of all template combinations...
echo Total: 48 combinations (4 templates x 3 algorithms x 4 solvers)
echo.

set algos=TF ST GDFS
set solvers=GridSearch Bayesian ParticleSwarm PolyHoot
set template_count=0

for %%t in (1 2 3 4) do (
    set /a template_count+=1
    echo Processing Template %%t...
    
    for %%a in (%algos%) do (
        for %%s in (%solvers%) do (
            echo Running combination - Template: %%t, Algo: %%a, Solver: %%s
            
            if %%t==1 (
                python make_config.py --algo %%a --minw 0.1 0.3 0.5 -10.0 --maxw 100.0 100.0 100.0 -100.0 --solver %%s --steps 1 --params %%s_spec.json
            ) else if %%t==2 (
                python make_config.py --algo %%a --minw 0.01 0.5 0.5 -7.0 --maxw 100.0 100.0 100.0 -100.0 --solver %%s --steps 1 --params %%s_spec.json
            ) else if %%t==3 (
                python make_config.py --algo %%a --minw 0.4 0.2 0.5 -7.0 --maxw 100.0 100.0 100.0 -100.0 --solver %%s --steps 1 --params %%s_spec.json
            ) else if %%t==4 (
                python make_config.py --algo %%a --minw 0.4 0.2 0.3 -5.0 --maxw 100.0 100.0 100.0 -100.0 --solver %%s --steps 1 --params %%s_spec.json
            )
            
            echo Command completed.
            echo.
        )
    )
    echo Template %%t completed.
    echo.
)

echo All combinations have been executed successfully!