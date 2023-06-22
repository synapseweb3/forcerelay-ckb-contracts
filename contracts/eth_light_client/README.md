# IBC CKB Contracts / ETH Light Client

> :warning: **WARNING** This contract is still in the proof-of-concept stage,
> and not compatible with [IBC] at present.

An implementation of [ETH] light client in [CKB] chain.

## Purpose

We want to verify [Ethereum (a.k.a. ETH)] transactions in [Nervos CKB (CKB
in short)].

## Design

We will provide two [CKB] contracts to implement that:

- First contract is used to synchronize [ETH] state into [CKB].

  This contract will be a [type script].

  We called it "client type lock".

- Second contract is used to verify [ETH] transactions in [CKB].

  This contract will be an executable binary which saved in a [CKB] cell,
  and it could be called from another contract as a [cell dep].

  We called it "verify bin".

And when users want to verify a [ETH] transaction, they still have to write
another contract to invoke the "verify bin", and this contract contains
users own business logic.

## Implementations

- [Client Type Lock](client_type_lock)

- [Verify Bin](verify_bin)

- [Mock Business Type Lock](mock_business_type_lock)

[IBC]: https://github.com/cosmos/ibc
[Ethereum (a.k.a. ETH)]: https://ethereum.org/
[Nervos CKB (CKB in short)]: https://www.nervos.org/
[ETH]: https://ethereum.org/
[CKB]: https://www.nervos.org/

[type script]: https://github.com/nervosnetwork/rfcs/blob/v2020.01.15/rfcs/0022-transaction-structure/0022-transaction-structure.md#type-script
[cell dep]: https://github.com/nervosnetwork/rfcs/blob/v2020.01.15/rfcs/0022-transaction-structure/0022-transaction-structure.md#ckb-transaction-structure
