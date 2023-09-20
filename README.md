# FORCERELAY CKB Contracts

[![License]](#license)
[![GitHub Actions]](https://github.com/synapseweb3/forcerelay-ckb-contracts/actions)

> :warning: **WARNING** This repository is still in the proof-of-concept stage.

This project consists of two CKB contracts serving for `Forcerelay/Eth` and `Forcerelay/Axon` respectively, which are all in one project, [Forcerelay](https://github.com/synapseweb3/forcerelay).

[License]: https://img.shields.io/badge/License-MIT-blue.svg
[GitHub Actions]: https://github.com/synapseweb3/forcerelay-ckb-contracts/workflows/CI/badge.svg

## Contracts for Forcerelay/Eth

### Mock Contracts

- ["Can Update Without Ownership" Lock](contracts/mock_contracts/can_update_without_ownership_lock)

### ETH Light Client

- [ETH Light Client](contracts/eth_light_client)

## Contracts for Forcerelay/Axon

`ibc-ckb-contracts` for `Forcerelay/Axon` are IBC-compatible contracts that validate CKB transactions and parse payloads therefrom. These payloads, in the format of `bytes`, can be converted into IBC objects, which will be used to complete further verification representing the underlying logic of the IBC protocol. `ibc-ckb-contracts` mainly include Connection, Channel, and Packet contracts.

Note: To see the fundamental verification logic, please refer to the [ckb-ics](https://github.com/synapseweb3/ckb-ics) library.

### Transaction Structure

We designed connection, channel, and packet cells, each of them contains unique data that corresponds to its specific IBC protocol that defines how these cells operate respectively.

For instance, in a transaction representing the `MsgChannelOpenInit` message, a new channel cell is created simultaneously with adjustments to the current connection cell status. This results in the following transaction structure:

```makefile
celldeps:
    connection contract cell

inputs:
    old connection cell

outputs:
    new connection cell
    new channel cell

witnesses:
    preimage of connection cell
    preimage of channel cell
```

## License

Licensed under [MIT License].

[IBC]: https://github.com/cosmos/ibc
[CKB]: https://github.com/nervosnetwork/ckb

[MIT License]: LICENSE
