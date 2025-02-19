# Installation Guide
The following guide will help with installing all dependensies to run the code in this repository. The guide has been made to be as simple as possible. 

After following this guide, you will have a virtual python environment which only contains the required dependensies. If you run into any issues first ensure you're IDE is configured to use this virual environment.

## Initial installation 
During initial installation run the following command the terminal / cmd:
```bash
./manage_env.sh install
```

This will automatically create a virtual environment, updated it and install all dependensies. 

## Update 
If during the course of this project additional dependensies are required add them to the `requirement.txt` file with the correct version of the dependensie. After update the virtual environment by running:
```bash
./manage_env.sh update
```

## Common Issues

permission denied
```bash
chmod +x manage_env.sh
```

activating environment
```bash
source venv/bin/activate
```