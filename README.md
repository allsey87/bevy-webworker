# Bevy in a Web Worker

This repository is a template that shows how Bevy can be run inside of a Web Worker. I have avoided using a bundler in this template to keep the code and build process easy to understand. The Bevy app in this repository is a simple world with lights, a camera, four walls, and a red ball that can be rolled around and that bounces off walls. For more detailed information about the architecture of this repository, refer to my blog post: [Running Bevy in a Web Worker with Rendering and Physics!](https://allwright.io/#/blog/20241127-bevy-webworker.md)

## Quick start

### GitHub Codespace
To play around with this repository and run the example without downloading or installing anything, click the green button labeled **Use this template** in the top-right corner of the GitHub user interface and select **Open in a codespace**. This will open the repository in Visual Studio Code for the Web and build the included Docker image. After the image is built, you can run the example by typing `python3 run.py` into VS Code's terminal. Note that since the default VM used by Github Codespaces only has two virtual CPUs, building the development container will take about 2 minutes, while building the project will take around 12 minutes. Each free GitHub account includes 120 core-hours for free.

### Local development
If you would like to play around with this repository locally, start by clicking the green button labeled **Use this template** in the top-right corner of the GitHub user interface and select **Create a new repository**. This will copy this repository to your account with a clean git history.

You can then clone either this repository or the new repository that you just created to your local machine. A `Dockerfile` which satisfies all the necessary dependencies is included in the `.devcontainer` directory. This `Dockerfile` can be built manually or using VS Code's [Dev Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) extension (recommended).

Once the container is running, enter `python3 run.py` into VS Code's terminal to build the crates, run wasm-bindgen and wasm-opt, and start the Python web server on `http://localhost:3000`.

## Repository structure

| File          | Description                                                                   |
|---------------|-------------------------------------------------------------------------------|
| README.md     | This file                                                                     |
| Cargo.toml    | Cargo workspace configuration                                                 |
| Cargo.lock    | Cargo's lock file                                                             |
| worker        | Worker crate that includes all the Bevy logic and is executed in a Web Worker |
| main          | Main crate for the user interface code that runs in the main thread           |
| shared        | Shared crate for message types used by both the worker and main crates        |
| run.py        | Development script to build and serve the code                                |
| index.html    | Static HTML document to be loaded                                             |
| reset.css     | Minimal CSS rules to normalize differences between browsers                   |
| .cargo        | Cargo configuration directory                                                 |
| .devcontainer | Contains Dockerfile and devcontainer.json for VSCode                          |
| .gitignore    | Rules for ignoring the output and target directories in git                   |
