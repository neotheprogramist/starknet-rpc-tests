# Output Directory

This directory is responsible for storing the input for the `t8n` tool.

You are allowed to modify these files.

# Update envs

Next, add the path to this file to your environment variables named `TXNS_PATH` and `ACC_PATH` in the `.cargo/config.toml` file. This will ensure that the t8n tool can locate and use these files.

It shall look like this:

```rust
ACC_PATH = /path/to/the/file
TXNS_PATH = /path/to/the/file
```
