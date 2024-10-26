# Client

The client is built as a browser extension which safely handles all the required cryptography from the user's side.

## Build

To build the browser extension use the command:

```bash
trunk build --config TrunkExtension.toml
```

Then load it into your browser using the browser-specific UI.

## Develop

For convenience this plugin can also be built as a website to for faster development and debugging.

```bash
trunk serve --open
```
