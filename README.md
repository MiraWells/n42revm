### Revm

[![CI](https://github.com/bluealloy/revm/actions/workflows/ci.yml/badge.svg)][gh-ci]
[![License](https://img.shields.io/badge/License-MIT-orange.svg)][mit-license]
[![Chat][tg-badge]][tg-url]

Revm is a highly efficient and stable implementation of the Ethereum Virtual Machine (EVM) written in Rust.

![banner](https://raw.githubusercontent.com/bluealloy/revm/refs/heads/main/assets/logo/revm-banner.png)

[mit-license]: https://opensource.org/license/mit/
[gh-ci]: https://github.com/bluealloy/revm/actions/workflows/ci.yml
[tg-url]: https://t.me/+Ig4WDWOzikA3MzA0
[tg-badge]: https://img.shields.io/badge/chat-telegram-blue

Renowned for its reliability, it is not only one of the most widely used libraries but also a vital part of the Ethereum ecosystem. Revm is instrumental in numerous projects, with nearly all development tools and block builders relying on it. It is integrated into Reth, various Layer 2 solutions, and other clients, and is increasingly regarded as a standard for zkVMs.

Revm offers two primary applications: firstly, it functions as an executor where users can set up block info and process mainnet transactions; secondly, it acts as a framework that facilitates the extension and support of different EVM variants such as revm-optimism.

### How to use:

Here is a straightforward example of using the Execution API: It allows us to create an Ethereum Virtual Machine (EVM) and execute transactions. Additionally, it can be utilized to generate traces with the inspector or more complex example of foundry cheatcodes.

```rust,ignore
let mut evm = Context::mainnet().with_block(block).build_mainnet();
let out = evm.transact(tx);

// or you can use powerful inspection tool to trace it
let mut evm = evm.with_inspector(tracer);
let out = evm.inspect_with_tx(tx);
```

The Evm Framework API is somewhat complex to use, but this document provides a detailed explanation. It enables users to extend logic, incorporate various context types, and offers built-in support for inspection. For a practical example, you can refer to the revm-optimism crate.

### Users:

As previously noted, there are several groups of projects that utilize this technology:

* **Major block builders**.
* **Clients**: [Reth](https://github.com/paradigmxyz/reth), [Helios](https://github.com/a16z/helios), [Trin](https://github.com/ethereum/trin),..
* **Tooling**: [Foundry](https://github.com/foundry-rs/foundry/), [Hardhat](https://github.com/NomicFoundation/hardhat),..
* **L2s**: [Optimism](https://github.com/bluealloy/revm/tree/main/crates/optimism), [Coinbase](https://www.base.org/), [Scroll](https://github.com/scroll-tech/revm),..
* **zkVM**: [Risc0](https://github.com/risc0/risc0-ethereum), [Succinct](https://github.com/succinctlabs/sp1-reth),..

The full list of projects that use Revm is available in the awesome-revm section of the book.

### How to, dev section

Note that book and code docs are still in WIP stage and they are being updated!

Part of the links point to the code documentation or the book. code docs are there to explain usage of particular part of the code where book is to get more of the overview on architecture or how components/projects fit toggether.

* How to build and use revm can be found here. (code)
* Architecture overview can be seen here. (book)
* Structure of the project (list of crates) can be seen here. (book)
* How to use Revm Framework can be found here. (book)
* Release procedure and changelogs explanation. (book)
* How to use revme (Revm binary with few commands) can be found here. (code)
* How to run Ethereum test can be found here: (book)
* How to run examples and benchmark with `samply` to check performance. (book)
* If there is more explanations please open PR request for it.

### Community:
For questions please open an github issue or join public telegram group: [https://t.me/+Ig4WDWOzikA3MzA0](https://t.me/+Ig4WDWOzikA3MzA0)

### Licence
Revm is licensed under MIT Licence.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in these crates by you, shall be licensed as above, without any additional terms or conditions.

### Security

For any security questions or findings, please reach out to me directly via email at dragan0rakita@gmail.com or contact me on Keybase under the username [draganrakita](https://keybase.io/draganrakita/).
