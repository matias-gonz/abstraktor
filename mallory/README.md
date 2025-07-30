# Setup Guide

## Prerequisites

Before proceeding, make sure you have the following tools installed:

- **Docker**: Install Docker from the [official page](https://docs.docker.com/get-docker/)
- **Docker Compose**: Install Docker Compose from the [official page](https://docs.docker.com/compose/install/)

## Setup

In the root of the project, execute the following commands:

```
cd docker
sudo ./bin/up
```

The first time you run this, it may take several minutes to complete.

## Setting Up the Mediator

Before running tests in Mallory, you'll need to set up the mediator. If it's your first time, follow these steps:

1. **Install Rustup**:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **Install Musl Tools**:

```
sudo apt-get install musl-tools
```

3. **Add Musl Target for Rust**:

```
rustup target add x86_64-unknown-linux-musl
```

4. **Build the Mediator**:

```
cargo build --target=x86_64-unknown-linux-musl
```

If the command doesn't execute, make sure to add the `~/.cargo/bin` folder to your `PATH` variable or in your `.bashrc` file.

### Running the Application

Jepsen is run using Docker. It has a control plane, a main container that manages five nodes where the applications are deployed. Fortunately, a script will set up the environment for you. Simply execute:

```
cd docker
sudo ./bin/up
```

This may take over 10 minutes on the first run.

### Running the Mediator

Once Jepsen is set up, you can run the mediator. This module intercepts messages between nodes and sends them to Mallory. Open a new terminal tab and run:

```
cd mediator && sudo target/x86_64-unknown-linux-musl/debug/mediator qlearning event_history 0.7
```

**Note**: The mediator must be run with `sudo` privileges to access network interfaces and iptables.

### Troubleshooting

If you encounter issues when running the mediator or Docker setup, here are common problems and their solutions:

#### Docker Compose Issues

**Problem**: `KeyError: 'ContainerConfig'` when running `sudo ./bin/up`

**Solution**: Clean up old containers and rebuild:
```bash
cd docker
docker-compose down --volumes --remove-orphans
docker rm -f $(docker ps -aq --filter "name=jepsen")
docker rmi jepsen_control jepsen_node
sudo ./bin/up
```

#### Iptables Compatibility Issues

**Problem**: `unknown option "--queue-num"` error when running the mediator

**Solution**: Switch to legacy iptables backend:
```bash
sudo update-alternatives --set iptables /usr/sbin/iptables-legacy
sudo update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy
```

#### Permission Issues

**Problem**: Permission denied errors when writing log files

**Solution**: Create a directory with proper permissions:
```bash
sudo mkdir -p /tmp/mediator-logs
sudo chmod 777 /tmp/mediator-logs
```

The mediator configuration has been updated to use `/tmp/mediator-logs/` for all log files.

#### Port Conflicts

**Problem**: `Address in use` error when starting the mediator

**Solution**: The mediator has been configured to use port 5001 instead of 5000 to avoid conflicts with the Docker control container. If you need to change the port, edit `mediator/Rocket.toml`.

#### Network Interface Issues

**Problem**: Mediator can't find experiment network interfaces

**Solution**: Ensure the experiment network configuration in `mediator/Mediator.toml` matches your Docker network setup. The default is `10.1.0.0/16`.

### Running Jepsen Tests

Finally, to run Jepsen tests, access the control plane in another terminal tab and execute:

```
cd docker
sudo ./bin/console
```

Navigate to the test you want to execute and each folder it will tell you how to do it

