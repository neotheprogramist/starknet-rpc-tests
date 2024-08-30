# Output Directory

This directory is responsible for storing the output generated by the `t8n` tool.

Please create a file named `state.json` in this directory. This file will be used to store the current state of the system after processing transactions.

To create the file, you can use the following command in your terminal:

```sh
touch state.json
```

# Update envs

Next, add the path to this file to your environment variable named `STATE_PATH` in the `.cargo/config.toml` file. This will ensure that the t8n tool can locate and use the state.json file.

It shall look like this: 
```rust
STATE_PATH = /path/to/the/file
```