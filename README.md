
<h1 align="center">
  🚧 rbx-products 📦
</h1>

<div align="center">

[![Version](https://img.shields.io/crates/v/rbx-products.svg?style=flat-square)](https://github.com/outofbears/rbx-products/releases)
[![License](https://img.shields.io/github/license/outofbears/rbx-products.svg?style=flat-square)](https://github.com/outofbears/rbx-products/blob/main/LICENSE.md)
[![Stars](https://img.shields.io/github/stars/outofbears/rbx-products.svg?style=flat-square)](https://github.com/outofbears/rbx-products/stargazers)
[![Forks](https://img.shields.io/github/forks/outofbears/rbx-products.svg?style=flat-square)](https://github.com/outofbears/rbx-products/network/members)
[![Watchers](https://img.shields.io/github/watchers/outofbears/rbx-products.svg?style=flat-square)](https://github.com/outofbears/rbx-products/watchers)
[![Issues](https://img.shields.io/github/issues/outofbears/rbx-products.svg?style=flat-square)](https://github.com/outofbears/rbx-products/issues)
[![Pull Requests](https://img.shields.io/github/issues-pr/outofbears/rbx-products.svg?style=flat-square)](https://github.com/outofbears/rbx-products/pulls)
[![Last Commit](https://img.shields.io/github/last-commit/outofbears/rbx-products.svg?style=flat-square)](https://github.com/outofbears/rbx-products/commits/main)


</div>

<br />

`rbx-products` is a lightweight CLI for downloading, editing, syncing, and publishing Roblox Universe Products (developer products, game passes, etc.) via the official Roblox Web API. It helps teams keep products in version control and batch-update prices safely.



## ✨ Features

- **Download**: Export all universe products to a local TOML file.
- **Sync**: Update remote universe products to match your local TOML file.
- **Luau file generation**: If the `luau-file` key exists in your TOML, a Luau file is automatically generated in both sync and download. The format is defined below.

## 📦 Installation

If the crate is published to crates.io, you can install with:

```bash
cargo install rbx_products
```

You need Rust (stable) and Cargo.

- From source (in this repo):

```bash
cargo build --release
# Binary at target/release/rbx_products
```

## 🔐 Authentication

rbx-products calls Roblox APIs that require authentication.

- Set the `RBX_API_KEY` environment variable to your Roblox Open Cloud API key for secure access.

Windows PowerShell example:

```powershell
$env:RBX_API_KEY = "<your Roblox Open Cloud API key>"
```

## 🚀 Usage

All commands operate on the default `products.toml` file in the workspace.

### 📥 Download products

Export the current universe products to a local file:

```bash
rbx-products download
```

### 🔄 Sync products

Update remote universe products to be in-sync with your local TOML file. 

```bash
rbx-products sync
```

## 🧩 Configuration file schema

The local TOML file is structured with metadata, gamepasses, and products sections. Example:

```toml
[metadata]
universe-id = 1234
luau-file = "products.luau"
discount-prefix = "💲{}% OFF💲 "
name-filters = []

[gamepasses."ab"]
id = 123
price = 100
active = true
discount = 10
regional-pricing = true

[products."a"]
id = 456
name = "product-name"
description = "Description of the product-name."
price = 200
active = true
discount = 0
regional-pricing = true
```

See `products.example.toml` for a full template.

## 🧩 Luau File Generation

If the `luau-file` key exists in your TOML file, a Luau file is automatically generated during both sync and download. This feature is optional and only enabled if the key is present.

### Luau File Format

The generated Luau file is structured as:

```lua
export type Product = { id: number, price: number }

return {
  Gamepasses = {
    ["Gamepass Name"] = { id = 123, price = 749 },
    -- more gamepasses...
  },
  Products = {
    ["Product Name"] = { id = 1330804404, price = 1 },
    -- more products...
  }
}
```

## 🧩 Name Sanitization & Prefix

When downloading or syncing, product names are automatically sanitized and the prefix file for products is overwritten with the discount prefix specified in the TOML file. The `-o` (overwrite) flag disables prompts and confirmation messages.

## 🔧 Logging & environment

- Set `RUST_LOG` to control verbosity (defaults to `rbx_products=debug` in debug builds, `rbx_products=info` in release):

```bash
RUST_LOG=rbx_products=debug rbx-products download
```

- Optional: a `.env` file is loaded if present for `RBX_API_KEY` or other environment variables.

## 🧰 Troubleshooting

- **403 Forbidden**: Ensure `RBX_API_KEY` is valid and not expired; try re‑setting it.
- **Rate limit**: The client backs off automatically; you may need to wait.
- **Invalid products TOML**: rbx-products will log parse errors—verify your file conforms to the schema above.
- **Overwrite not working**: Ensure you use the `-o` flag with sync/upload to force updates.

## 💖 Contribution

rbx-products was developed by [@Bear](https://github.com/OutOfBears)

## 📄 License

This project is licensed under the MIT License - see the [`LICENSE`](LICENSE.md) file for details.