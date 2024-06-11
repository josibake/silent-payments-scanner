# Silent Payments Scanning

Tool for scanning the blockchain for silent payments. Demo of the `libbitcoinkernel` library.

For building/running, both `PKG_CONFIG_PATH` and `LD_LIBRARY_PATH` need to be set, e.g. :

```
LD_LIBRARY_PATH=/usr/local/lib PKG_CONFIG_PATH=/usr/local/lib/pkgconfig cargo run -- --datadir "/path/to/datadir" --network "testnet" --scankey "scan_key" --spendpubkey "spend_pub_key"
```
