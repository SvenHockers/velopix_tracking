# 📦 Installation Guide

This guide will walk you through setting up the environment and installing all required dependencies to run the code in this repository.  
For simplicity, we use a **virtual Python environment**, ensuring all dependencies remain isolated.

🚀 **After completing this guide, you'll have:**
- A fully functional virtual environment.
- All required dependencies installed.
- A clean workspace for running and developing the project.

If you encounter issues, make sure your **IDE is configured** to use the correct virtual environment.

---

## 🔧 Initial Installation

To set up the environment, open a terminal (or command prompt) and run:

```bash
./manage_env.sh install
```

This command will: </br>
✅ Create a **virtual environment**  
✅ Install all required dependencies  
✅ Update dependencies to the latest compatible versions  

---

## 🔄 Updating Dependencies

If new dependencies are required during development:
1. **Add the new package** (with the correct version) to `requirements.txt`.
2. Run the following command to update the environment:

```bash
./manage_env.sh update
```

This ensures your environment stays up to date with the latest project requirements.

---

## ⚠️ Common Issues & Fixes

### 🛑 Permission Denied for `manage_env.sh`
If you encounter a **permission denied** error when running the script:
```bash
chmod +x manage_env.sh
```
This grants execution permission to the script.

### 🚀 Activating the Virtual Environment
If the environment is not active, manually activate it using:
```bash
source venv/bin/activate
```
For Windows users (Command Prompt):
```cmd
venv\Scripts\activate
```
For Windows users (PowerShell):
```powershell
venv\Scripts\Activate.ps1
```

---

## ✅ You're All Set!
You should now have everything installed and ready to go. Run the project with:
```bash
python3 run_track_reconstruction.py
```

For further troubleshooting or contributions, check the [README](README.md). 🚀
