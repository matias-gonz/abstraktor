# Setup Guide

## Prerequisites

Before proceeding, make sure you have the following tools installed:

- **Vagrant**: Download the appropriate version for your platform from the [official page](https://developer.hashicorp.com/vagrant/downloads).
- **VirtualBox**: This is the default option for VM management. It's not mandatory, but you can configure a different VM in the Vagrant options if you prefer.

## VM Setup

In the root of the project, execute the following commands:

```
cd docker
vagrant up
```

The first time you run this, it may take several minutes to complete.

## Accessing the VM

To enter the VM, run:

```
vagrant ssh
```

### Setting Up the Mediator

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
cd /jepsen/docker
sudo ./bin/up
```

This may take over 10 minutes on the first run.

### Running the Mediator

Once Jepsen is set up, you can run the mediator. This module intercepts messages between nodes and sends them to Mallory. Open a new terminal tab, log into Vagrant again, and run:

```
cd /jepsen/mediator && target/x86_64-unknown-linux-musl/release/mediator qlearning event_history 0.7
```

### Running Jepsen Tests

Finally, to run Jepsen tests, access the control plane in another terminal tab and execute:

```
cd /jepsen/docker
sudo ./bin/console
```

Navigate to the test you want to execute and each folder it will tell you how to do it

