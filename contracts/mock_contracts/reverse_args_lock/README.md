# IBC CKB Contracts / Mock Contracts / Reverse Args Lock

> :warning: **WARNING** This contract is testing purpose only.

This contract is a mock contract, and it is used for testing.

It will return success when any follow condition is satisfied:
- the witness is reverse of the `args` in the old lock script.
- if there is no witness, the new lock script should be the same as the old
  lock script.
