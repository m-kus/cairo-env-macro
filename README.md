# **`env!`** macro for Cairo

Inspect environment variables at compile time.

## Usage

In your `Scarb.toml`:

```toml
[dependencies]
env_macro = "0.1.0"
```

In your code:

```cairo
const VERSION: usize = env!("VERSION");
```

## Features

- Specify a default value if the environment variable is not set:
    ```cairo
    const VERSION: usize = env!("VERSION", 1);
    ```
- Only numeric values are supported at the moment.
