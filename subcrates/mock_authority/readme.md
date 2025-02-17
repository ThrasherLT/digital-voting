# Mock Authority

In order to demonstrate and test how this whole voting system works, a mock authority had been created. This mock authority does not do any proper voter validation as it is out of scope of this system, but it does blindly signs blinded public keys for the users of the blockchain. This functionality is supported both via `stdio` CLI, http requests and a UI frontend.

## Build

This whole binary can be built using `cargo build`. The frontend is automatically built by `build.rs` from the `frontend` subcrate and packaged alongside the binary in the `target` dir.

## Usage

Enter `help` into the `stdio` CLI to view the list of commands.

WIP: http requests.

The UI frontend can be accessed on the host port 8080 by default.
