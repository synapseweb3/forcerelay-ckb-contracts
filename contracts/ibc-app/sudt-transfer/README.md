# SUDT Transfer Module for CKB IBC

> :warning: **WARNING** This contract is still in the proof-of-concept stage.

This smart contract (CKB lock script) provides a way to send and receive [SUDT (Simple User Defined Token)](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0025-simple-udt/0025-simple-udt.md) across different chains using the IBC protocol. The design of this module is based on the [ICS20 specification](https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md), with the goal of being compatible with the Solidity implementation of the [ICS20Transfer](https://github.com/synapseweb3/ibc-solidity-contract/blob/master/contracts/apps/20-transfer/ICS20Transfer.sol) contract.

This SUDT transfer module (`st-lock`) serves as an escrow lock, similar to [the Solidity implementation](https://github.com/synapseweb3/ibc-solidity-contract/blob/6c025378ab2640fe5b1c4ffa2a9e936659d88101/contracts/apps/20-transfer/ICS20Transfer.sol#L163). The lock's arguments should include client, channel and packet information which will be checked to ensure the security of the transfer process.

The send/recv/refund of a SUDT interchain transfer should be completed in a single CKB transaction. The specific SUDT amount in the transaction is calculated from the difference of the input/output cells. A `st-lock` transaction only allows one type of SUDT and it cannot unlock other SUDT cells.
