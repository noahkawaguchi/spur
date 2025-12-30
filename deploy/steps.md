# Deployment Steps

This document documents the steps of the deployment process from an Ubuntu instance on AWS EC2. It's not fully scripted because many of the commands require `sudo` and/or manual interactivity.

## Step 0: System basics

Make sure packages are up to date and (optionally) change the hostname.

```bash
sudo apt update
sudo apt upgrade

sudo hostnamectl set-hostname spur-server
exec bash
```

## Step 1: Rootless Docker

Install the Docker Engine using the Docker `apt` repository method as explained in the Docker docs [here](https://docs.docker.com/engine/install/ubuntu/#install-using-the-repository).

While it's easier to run Docker as root, it should really be run rootless for better security. Rootless Docker setup is explained in the Docker docs [here](https://docs.docker.com/engine/security/rootless/) and includes the following commands in this case:

```bash
sudo apt install uidmap
sudo systemctl disable --now docker.service docker.socket
sudo rm /var/run/docker.sock
dockerd-rootless-setuptool.sh install
```

While this step is not strictly required, rootful Docker can be masked to make sure it never runs unless explicitly unmasked:

```bash
sudo systemctl mask docker.service docker.socket
```

User containers need to run even when the user is not logged in. The default username is `ubuntu`.

```bash
sudo loginctl enable-linger ubuntu
```

Caddy will need access to privileged ports 80 and 443. This part may need to be repeated after updating packages.

```bash
sudo setcap cap_net_bind_service=ep "$(command -v rootlesskit)"
systemctl --user restart docker
```

## Step 2: Running the Spur stack

Since only a few files are required, a simple `~/spur` directory will suffice.

```bash
cd
mkdir -p spur
cd spur
```

The `bootstrap.sh` utility is available in the main Spur repo.

```bash
wget https://raw.githubusercontent.com/noahkawaguchi/spur/main/deploy/bootstrap.sh
chmod +x bootstrap.sh
```

This script provides three commands: `files`, `run`, and `reset`. (Run it with anything else or nothing for a usage message.) `files` downloads the other required files from the main Spur repo.

```bash
./bootstrap.sh files
```

At this point, `.env` (which was created automatically if it didn't already exist) must be manually filled out with all the required environment variables as documented in the comments and `.env.example`.

```bash
vim .env
```

Finally, the stack is ready to be run.

```bash
./bootstrap.sh run
```

Since this project's data is mostly for demonstration purposes, the database may need to be reset to its original seeded state at some point.

```bash
./bootstrap.sh reset
```
