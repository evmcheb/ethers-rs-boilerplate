# ethers-rs script boilerplate

ethers-rs is a great blockchain scripting framework! This repo contains the essentials I find are useful when reading/writing to EVM chains.

It uses [ethers-rs](https://github.com/gakonst/ethers-rs) and includes examples for block+event subscriptions, tx signing, and debug tracing. It also automates ABI bindings so you get nice typed Rust structs for interacting with your contracts.

## Automated ABI Build Script

- **Automated ABI Bindings**: Simply place the contract ABI JSON files into the `/abi` directory, and Rust bindings will be generated automatically during the build process.

## Getting Started

### Environment Setup

.env will be read on startup

```env
PRIVATE_KEY=your_private_key_here
RPC=your_rpc_url_here
```

### Building

Compile the project using:

bash

```bash
cargo build
```

### Running

Execute the compiled binary:

bash

```bash
cargo run
```

Modules
-------

*   `bindings`: Contains the Rust bindings for interacting with Ethereum smart contracts.
*   `trace`: Functionality for tracing and flattening Ethereum transactions.


Contributing
------------

Feel free to fork the repository and submit pull requests for any enhancements, features, or bug fixes.

License
-------

[MIT License](LICENSE)

Acknowledgments
---------------

Powered by [ethers-rs](https://github.com/gakonst/ethers-rs), an Ethereum & Celo library and wallet implementation, written in pure Rust.

Support
-------

For any questions or support, please open an issue or contact [cheb](https://twitter.com/evmcheb).
