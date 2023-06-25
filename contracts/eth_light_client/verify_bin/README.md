# IBC CKB Contracts / ETH Light Client / Verify Bin

> :warning: **WARNING** This contract is still in the proof-of-concept stage,
> and not compatible with [IBC] at present.

This contract is used as an executable binary, to verify if a transaction is
on the [ETH] chain.

This contract requires two arguments:
- first is the index of the "client" cell.
- second is the index of the witness for transaction proof and payload.

[IBC]: https://github.com/cosmos/ibc
[ETH]: https://ethereum.org
[CKB]: https://github.com/nervosnetwork/ckb
