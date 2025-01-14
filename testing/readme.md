# Testing Environment

This is a simple testing environment based on docker compose.

## Platform Support

Currently this testing environment is only supported on linux

## Setup

In order to run the testing environment, follow these steps:

- Run `./requirements.sh` to install the host dependencies (note that this currently is only supported on Ubuntu, but it's easy to install the dependencies manually by referencing the script).
- Build the project using `cargo build`. The containers will mount the `target/debug/` dir and run the executables from there.
- Run `./run.sh` to run the testing environment (note that this could take up to half a minute if the configs are being generated).

## Usage

You can check the `./data` directory for port numbers and logs of all the containers.
You can also edit the `docker-compose.yml` file to adjust the number of nodes and authorities.

## Shutdown

If you want to stop the testing environment, follow these steps:

- Run `./stop.sh` to stop the containers (note that this could take up to a minute).
- If you want to clear configs (so new configs could be generated on the next run), you can run `sudo ./clean.sh`.
- The HTTP endpoints of the authority containers can be found in the `data/blockchain-config.json` file.

## TODO

The following features will/should/might be implemented:

- Multiplatform `./requirements.sh` support.
- Faster `./stop.sh` operation.
- Configurable candidates.
