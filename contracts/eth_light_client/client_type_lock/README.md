# IBC CKB Contracts / ETH Light Client / Client Type Lock

> :warning: **WARNING** This contract is still in the proof-of-concept stage,
> and not compatible with [IBC] at present.

This contract is used as a type lock, to synchronize [ETH] state into [CKB].

## Brief Introduction

An [ETH] light client in [CKB] contains a set of cells, this type script is
used to manage them.

Since this type script has a unique ID in its script [`args`], so the size
of the set of cells is immutable after they created.

### Cells

There are 3 kinds of cells in an [ETH] light client instance:

- Client Cell

  This cell is used to store the [ETH] state.

  Each [ETH] light client should contain at least 1 client cell.

- Client Info Cell

  This cell is used to store the basic information of current [ETH] light
  client. Such as the ID of the latest client cell.

  Each [ETH] light client should contain only 1 client info cell.

- Client Sync Committee Cell

  This cell is used to store the sync committee.

  Each [ETH] light client should contain only 2 client sync committee cells.

### Operations

There are 4 kinds of operations:

- Create

  Create all cells for an [ETH] light client instance in one transaction.

  The outputs of this transaction should contain 1 client info cell, 2
  client sync committee cells and at least 1 client cell.

  In the follow part of this document, we denoted the number of client cells
  as `n`.

  In current implementation, it requires that cells must be continuous and
  in specified order:

  - The client info cell should be at the first.

  - Immediately followed by `n` client cells, and these cells should be
    ordered by their ID from smallest to largest.

  - After above cells, there should be 2 client sync committee cells.

  The structure of this kind of transaction is as follows:

  ```yaml
  Cell Deps:
  - Client Type Lock
  - ... ...
  Inputs:
  - Enough Capacity Cells
  Outputs:
  - Light Client Info Cell (last_client_id=0)
  - Light Client Cell (id=0)
  - Light Client Cell (id=1)
  - Light Client Cell (id=2)
  - ... ...
  - Light Client Cell (id=n-2)
  - Light Client Cell (id=n-1)
  - Light Client Sync Committee Cell
  - Light Client Sync Committee Cell
  - ... ...
  Witnesses:
  - Client Bootstrap
  - ... ...
  ```

- Destroy

  All cells that use the same instance of this type lock should be destroyed
  together in one transaction.

  The structure of this kind of transaction is as follows:

  ```yaml
  Cell Deps:
  - Client Type Lock
  - ... ...
  Inputs:
  - Light Client Info Cell (last_client_id=0)
  - Light Client Cell (id=0)
  - Light Client Cell (id=1)
  - Light Client Cell (id=2)
  - ... ...
  - Light Client Cell (id=n-2)
  - Light Client Cell (id=n-1)
  - Light Client Sync Committee Cell
  - Light Client Sync Committee Cell
  - ... ...
  Outputs:
  - Unrelated Cell
  - ... ...
  Witnesses:
  - Unrelated Witness
  - ... ...
  ```

- Update Client

  After creation, the `n` client cells should have same data.

  The client cell who has the same ID as the  `last_client_id` in the client
  info cell, we consider that it has the latest data.

  The client cell who has the next ID of the  `last_client_id` in the client
  info cell, we consider that it has the oldest data. The next ID of ID
  `n-1` is `0`.

  Once we update the [ETH] light client, we put the new data into the client
  cell which has the oldest data, and update the `last_client_id` in the
  client info cell to its ID.

  Do the above step in repetition.

  The structure of this kind of transaction is as follows:

  ```yaml
  Cell Deps:
  - Client Type Lock
  - Light Client Cell (id=k)
  - Light Client Sync Committee Cell (current period)
  - ... ...
  Inputs:
  - Light Client Cell (id=k+1)
  - Light Client Info Cell (last_client_id=k)
  - ... ...
  Outputs:
  - Light Client Cell (id=k+1)
  - Light Client Info Cell (last_client_id=k+1)
  - ... ...
  Witnesses:
  - Client Update
  - ... ...
  ```

- Update Sync Committee

  After creation, the 2 sync committee cells should have same data, the
  period of them is denoted as `t`.

  If the sync committee of period `t+1` is finalized, we could update any of
  them to the sync committee of period `t+1`.

  After that, the data in sync committee cells should be in 2 adjacent
  periods, `t` and `t+1`.

  From this moment, once a sync committee of a new period is finalized, we
  update that into the older cell.

  Do the above step in repetition.

  The structure of this kind of transaction is as follows:

  ```yaml
  Cell Deps:
  - Client Type Lock
  - Light Client Info Cell (last_client_id=k)
  - Light Client Cell (id=k)
  - Light Client Sync Committee Cell (period=t)
  - ... ...
  Inputs:
  - Light Client Sync Committee Cell (period=t-1)
  - ... ...
  Outputs:
  - Light Client Sync Committee Cell (period=t+1)
  - ... ...
  Witnesses:
  - Sync Committee Update
  - ... ...
  ```

[IBC]: https://github.com/cosmos/ibc
[ETH]: https://ethereum.org
[CKB]: https://github.com/nervosnetwork/ckb

[`args`]: https://github.com/nervosnetwork/rfcs/blob/v2020.01.15/rfcs/0019-data-structures/0019-data-structures.md#description-1
