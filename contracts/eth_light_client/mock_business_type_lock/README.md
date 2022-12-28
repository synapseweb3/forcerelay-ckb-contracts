# IBC CKB Contracts / ETH Light Client / Mock Business Type Lock

> :warning: **WARNING** This contract is testing purpose only.

This contract is a mock contract, and it is used for testing.

It will call the [ETH Light Client / Verify Bin](../verify_bin) to verify.

The `args` of this type lock should contains 64 bytes:
- first 32 bytes is the type hash of the "client" cell.
- last 32 bytes is the type hash of the "verify bin" cell.
