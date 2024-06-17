# rust-bitcoinkernel demo

This repo is a demo of using the new `libbitcoinkernel` API, with rust bindings via [rust-bitcoinkernel](https://github.com/theCharlatan/rust-bitcoinkernel).

> [!WARNING]
> This is for demonstration purposes only. Expect to find bugs and do not use this in production

## one-time setup

The libbitcoinkernel project is still under active development and expected to change frequently. As such, we need to compile and install the C-compatible header from source and generate the rust bindings. See [rust-bitcoinkernel](https://github.com/theCharlatan/rust-bitcoinkernel) for more detailed instructions. This is "one-time" setup in that once you've completed this setup you don't need to repeat these steps to use [rust-bitcoinkernel] in any of your rust projects. You will, however, need to recompile Bitcoin Core and install the header any time the kernel API changes.

To get the C-header, compile the `kernelApi` branch and install:

```
git clone https://github.com/TheCharlatan/bitcoin kernel
cd kernel
git checkout kernelApi
./autogen.sh
./configure --with-experimental-kernel-lib --enable-shared
make install
```

Next, clone [rust-bitcoinkernel](https://github.com/theCharlatan/rust-bitcoinkernel):

```
git clone git@github.com:theCharlatan/rust-bitcoinkernel
```

and add to your project:

```
# Cargo.toml

[dependencies]

...

libbitcoinkernel-sys = { path = "../rust-bitcoinkernel/libbitcoinkernel-sys" }
```

## silent payments scanning utility

Build:

```
cargo b
```

Run:

```
cargo run -- --datadir "/path/to/datadir" \
             --network "regtest" \
             --scankey "WIF_encoded_scan_key" \
             --spendpubkey "hex_encoded_compressed_spend_pub_key"
```
> [!NOTE]
> You may need to update `.carg/config` to specify the correct install location for the libbitcoinkernel shared library (currently defaults to `/usr/local/lib`. Alternatively, you can specify `LD_LIBRARY_PATH=<path/to/lib> cargo run ..`.

