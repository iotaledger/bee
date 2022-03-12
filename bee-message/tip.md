ce4feebb00f0dd7968b58334465651b95bd21722

### Native Tokens in Outputs

#### Additional semantic transaction validation rules:

- The transaction is balanced in terms of native tokens, that is, the sum of native token balances in consumed outputs
  equals that of the created outputs.
- When the transaction is **imbalanced** and there is a surplus of native tokens on the:
  - **output side of the transaction:** the foundry outputs controlling outstanding native token balances must be present in the transaction. The validation of the foundry output(s) determines if the minting operations are valid.
  - **input side of the transaction:** the transaction destroys tokens. The presence and validation of the foundry outputs of the native tokens determines whether the tokens are burned (removed from the ledger) or melted in the foundry.

##### Storage Deposit Return Unlock Condition

This unlock condition is employed to achieve conditional sending. An output that has
<i>Storage Deposit Return Unlock Condition</i> specified can only be consumed in a transaction that deposits
`Return Amount` IOTA coins into `Return Address`. When several of such outputs are consumed, their return amounts per
`Return Addresses` are summed up and the output side must deposit this total sum per `Return Address`.

###### Additional syntactic transaction validation rule:

- `Return Amount` must be ≥ than `Minimum Storage Deposit` and must not be `0`.
- It must hold true, that `0` ≤ `Amount` - `Return Amount` ≤ `Required Storage Deposit of the Output`.

###### Additional semantic transaction validation rule:

- An output that has <i>Storage Deposit Return Unlock Condition</i> specified must only be consumed and unlocked in a
  transaction that deposits `Return Amount` IOTA coins to `Return Address` via an output that has no additional spending
  constraints. (<i>Basic Output</i> with only an <i>Address Unlock Condition</i>)
- When several outputs with <i>Storage Deposit Return Unlock Condition</i> and the same `Return Address` are consumed,
  their return amounts per `Return Addresses` are summed up and the output side of the transaction must deposit this
  total sum per `Return Address`.

##### Timelock Unlock Conditions

The timelock can be specified as a unix timestamp or as a milestone index. When specified in both ways, both conditions
have to pass in order for the unlock to be valid. The zero value of one if the timestamp fields signals that it should be
ignored during validation.

###### Additional semantic transaction validation rules:
- An output that has <i>Timelock Unlock Condition</i> specified must only be consumed and unlocked in a
  transaction, if the confirming milestone index is ≥ than the `Milestone Index` specified in the unlock condition.
- An output that has <i>Timelock Unlock Condition</i> specified must only be consumed and unlocked in a
  transaction, if the timestamp of the confirming milestone is equal or past the `Unix Time` specified in the unlock
  condition.

##### Expiration Unlock Conditions

###### Additional semantic transaction validation rules:

- If `Milestone Index` != `0`, an output that has <i>Expiration Unlock Condition</i> set must only be consumed and
  unlocked by the target `Address` (defined in <i>Address Unlock Condition</i>) in a transaction that has a confirming
  milestone index < than the `Milestone Index` defined in the unlock condition.
  `0` value of the `Milestone Index` field is a special flag that signals to the validation that this check
  must be ignored.
- If `Milestone Index` != `0`, an output that has <i>Expiration Unlock Condition</i> set must only be consumed and
  unlocked by `Return Address` in a transaction that has a confirming milestone index ≥ than the `Milestone Index`
  defined in the unlock condition.
  `0` value of the `Milestone Index` field is a special flag that signals to the validation that this check
  must be ignored.
- If `Unix Time` != `0`, an output that has <i>Expiration Unlock Condition</i> set must only be consumed and
  unlocked by the target `Address` (defined in <i>Address Unlock Condition</i>) in a transaction that has a confirming
  milestone timestamp earlier than the `Unix Time` defined in the unlock condition.
  `0` value of the `Unix Time` field is a special flag that signals to the validation that this check  must be ignored.
- If `Unix Time` != `0`, an output that has <i>Expiration Unlock Condition</i> set must only be consumed and unlocked
  by the `Return Address` in a transaction that has a confirming milestone timestamp same or later than the `Unix Time`
  defined in the unlock condition.
  `0` value of the `Unix Time` field is a special flag that signals to the validation that this check  must be ignored.
- Semantic validation of an output that has <i>Expiration Unlock Condition</i> set and is unlocked by the
  `Return Address` must ignore:
  - [Semantic validation of <i>Storage Deposit Return Unlock Condition</i>](#storage-deposit-return-unlock-condition) if present.

##### Immutable Alias Address Unlock Condition

An unlock condition defined for chain constrained UTXOs that can only be unlocked by a permanent <i>Alias Address</i>.

Output unlocking is functionally equivalent to an <i>Address Unlock Condition</i> with an <i>Alias Address</i>,
however there are additional transition constraints: the next state of the UTXO machine must have the same
<i>Immutable Alias Address Unlock Condition</i>.

###### Additional semantic transaction validation rules:
 - The output must be unlocked with an <i>[Alias Unlock Block](#alias-unlock-block-semantic-validation)</i>.
 - The next state of the UTXO state machine must have the same <i>Immutable Alias Address Unlock Condition</i> defined.

##### Sender Block

Every transaction consumes several elements from the UTXO set and creates new outputs. However, certain applications
(smart contracts) require to associate each output with exactly one sender address. Here, the sender block is used to
specify the validated sender of an output.

Outputs that support the <i>Sender Block</i> may specify a `Sender` address which is validated by the protocol during
transaction validation.

###### Additional semantic transaction validation rule:
- The <i>Sender Block</i>, and hence the output and transaction that contain it, is valid, if and only if an output
  with the corresponding `Address` is consumed and unlocked in the transaction. If `Address` is either
  <i>Alias Address</i> or <i>NFT Address</i>, their corresponding outputs (defined by `Alias ID` and `NFT ID`) must be
  unlocked in the transaction.

##### Issuer Block

The issuer block is a special case of the sender block that is only supported by outputs that implement a UTXO state
machine with [chain constraint](#chain-constraint-in-utxo) (alias, NFT).
Only when the state machine is created (e.g. minted) it is checked during transaction validation that an output
corresponding to the `Issuer` address is consumed. In every future transition of the state machine, it is instead
checked that the issuer block is still present and unchanged.

###### Additional semantic transaction validation rule:
- When an <i>Issuer Block</i> is present in an output representing the initial state of an UTXO state machine, the
  transaction that contains this output is valid, if and only if an output with the corresponding `Address` is consumed
  and unlocked in the transaction. If `Issuer` is either <i>Alias Address</i> or
  <i>NFT Address</i>, their corresponding outputs (defined by `Alias ID` and `NFT ID`) must be unlocked in the transaction.

##### Metadata Block

###### Additional syntactic transaction validation rules:
- An output with <i>Metadata Block</i> is valid, if and only if 0 < `Data Length` ≤ `Max Metadata Length`.

#### Tag Block

##### Additional syntactic transaction validation rules:
- An output with <i>Tag Block</i> is valid, if and only if 0 < `Tag Length` ≤
  `Max Tag Length`.

## Basic Output

### Additional Transaction Syntactic Validation Rules

- `Amount` field must fulfill the dust protection requirements and must not be `0`.
- It must hold true that `1` ≤ `Unlock Conditions Count` ≤ `4`.
- `Unlock Condition Type` of an <i>Unlock Condition</i> must define on of the following types:
  - <i>Address Unlock Condition</i>
  - <i>Storage Deposit Return Unlock Condition</i>
  - <i>Timelock Unlock Condition</i>
  - <i>Expiration Unlock Condition</i>
- <i>Unlock Conditions</i> must be sorted in ascending order based on their `Unlock Condition Type`.
- <i>Address Unlock Condition</i> must be present.
- It must hold true that `0` ≤ `Blocks Count` ≤ `3`.
- `Block Type` of a <i>Block</i> must define on of the following types:
  - <i>Sender Block</i>
  - <i>Metadata Block</i>
  - <i>Tag Block</i>
- <i>Blocks</i> must be sorted in ascending order based on their `Block Type`.

### Additional Transaction Semantic Validation Rules

#### Consumed Outputs

- The unlock block of the input must correspond to `Address` field in the <i>Address Unlock Condition</i> and the
  unlock must be valid.
- The unlock is valid if and only if all unlock conditions and feature blocks present in the output validate.

#### Created Outputs

- All <i>Unlock Condition</i> imposed transaction validation criteria must be fulfilled.
- All <i>Feature Block</i> imposed transaction validation criteria must be fulfilled.

## Alias Output

### Additional Transaction Syntactic Validation Rules

#### Output Syntactic Validation

- `Amount` field must fulfill the dust protection requirements and must not be `0`.
- It must hold true that `Unlock Conditions Count` = `2`.
- `Unlock Condition Type` of an <i>Unlock Condition</i> must define one of the following types:
  - <i>State Controller Address Unlock Condition</i>
  - <i>Governor Address Unlock Condition</i>
- <i>Unlock Conditions</i> must be sorted in ascending order based on their `Unlock Condition Type`.
- It must hold true that `0` ≤ `Blocks Count` ≤ `2`.
- `Block Type` of a <i>Block</i> in `Blocks` must define on of the following types:
  - <i>Sender Block</i>
  - <i>Metadata Block</i>
- It must hold true that `0` ≤ `Immutable Blocks Count` ≤ `2`.
- `Block Type` of a <i>Block</i> in `Immutable Blocks` must define on of the following types:
  - <i>Issuer Block</i>
  - <i>Metadata Block</i>
- <i>Blocks</i> must be sorted in ascending order based on their `Block Type` both in `Blocks` and `Immutable Blocks`
  fields.
- When `Alias ID` is zeroed out, `State Index` and `Foundry Counter` must be `0`.
- `State Metadata Length` must not be greater than `Max Metadata Length`.
- `Address` of <i>State Controller Address Unlock Condition</i> and `Address` of
  <i>Governor Address Unlock Condition</i> must be different from the alias address derived from `Alias ID`.

### Additional Transaction Semantic Validation Rules

- Explicit `Alias ID`: `Alias ID` is taken as the value of the `Alias ID` field in the alias output.
- Implicit `Alias ID`: When an alias output is consumed as an input in a transaction and `Alias ID` field is zeroed out
  while `State Index` and `Foundry Counter` are zero, take the BLAKE2b-160 hash of the `Output ID` of the input as
  `Alias ID`.
- The BLAKE2b-160 hash function outputs a 20 byte hash as opposed to the 32 byte hash size of BLAKE2b-256. `Alias ID`
  is sufficiently secure and collision free with 20 bytes already, as it is actually a hash of an `Output ID`, which
  already contains the 32 byte hash `Transaction ID`.
- For every non-zero explicit `Alias ID` on the output side there must be a corresponding alias on the input side. The
  corresponding alias has the explicit or implicit `Alias ID` equal to that of the alias on the output side.

#### Consumed Outputs

There are two types of transitions: `state transition` and `governance transition`.
- State transition:
    - The unlock block must correspond to the `Address` of <i>State Controller Address Unlock Condition</i>.
- Governance transition:
    - The unlock block must correspond to the `Address` of <i>Governor Address Unlock Condition</i>.

#### Created Outputs

- When <i>Issuer Block</i> is present in an output and explicit `Alias ID` is zeroed out, an input with `Address` field
  that corresponds to `Issuer` must be unlocked in the transaction.

## Foundry Output

Upon creation of the foundry, the alias defined in the `Address` field of the
<i>Immutable Alias Address Unlock Condition</i> must be unlocked in the same transaction, and its `Foundry Counter`
field must increment. This incremented value defines `Serial Number`, while the `Token Scheme` can be chosen freely.

`Foundry ID` is not allowed to change after deployment, therefore neither `Address`, nor `Serial Number` or
`Token Scheme` can change during the lifetime of the foundry.

Foundries control the supply of tokens with unique identifiers, so-called `Token IDs`. The **`Token ID` of tokens
controlled by a specific foundry is the concatenation of `Foundry ID` || `Token Tag`.**

### Additional Transaction Syntactic Validation Rules

#### Output Syntactic Validation

- `Amount` field must fulfill the dust protection requirements and must not be `0`.
- It must hold true that `Unlock Conditions Count` = `1`.
- `Unlock Condition Type` of an <i>Unlock Condition</i> must define on of the following types:
  - <i>Immutable Alias Address Unlock Condition</i>
- It must hold true that `0` ≤ `Blocks Count` ≤ `1`.
- `Block Type` of a <i>Block</i> in `Blocks` must define on of the following types:
  - <i>Metadata Block</i>
- It must hold true that `0` ≤ `Immutable Blocks Count` ≤ `1`.
- `Block Type` of a <i>Block</i> in `Immutable Blocks` must define on of the following types:
  - <i>Metadata Block</i>
- `Token Scheme Type` must match one of the supported schemes. Any other value results in invalid output.
- `Minted Tokens` must not be greater than `Maximum Supply`.
- `Melted Tokens` must not be greater than `Minted Tokens`.
- `Maximum Supply` must be larger than zero.

### Additional Transaction Semantic Validation Rules

- The **current state of the foundry** with `Foundry ID` `X` in a transaction is defined as the consumed foundry output
  where `Foundry ID` = `X`.
- The **next state of the foundry** with `Foundry ID` `X` in a transaction is defined as the created foundry output
  where `Foundry ID` = `X`.
- `Foundry Diff` is the pair of the **current and next state** of the foundry output in the transaction.

- The foundry output must be unlocked like any other output type where the <i>Address Unlock Condition</i> defines an
  <i>Alias Address</i>, by transitioning the alias in the very same transaction. See section
  [alias unlocking](#unlocking-chain-script-locked-outputs) for more details.
- When the current state of the foundry with `Foundry ID` is empty, it must hold true for `Serial Number` in the next
  state, that:
    - `Foundry Counter(InputAlias) < Serial Number <= Foundry Counter(OutputAlias)`
    - An alias can create several new foundries in one transaction. It was written for the alias output that freshly
      created foundry outputs must be sorted in the list of outputs based on their `Serial Number`. No duplicates are
      allowed.
    - The two previous rules make sure that each foundry output produced by an alias has a unique `Serial Number`,
      hence each `Foundry ID` is unique.
- Native tokens present in a transaction are all native tokens present in inputs and outputs of the transaction. Native
  tokens of a transaction must be a set based on their `Token ID`.
- There must be at most one `Token ID` in the native token set of the transaction that maps to a specific `Foundry ID`.
- Let `Token Diff` denote the difference between native token balances of the input and the output side of the
  transaction of the single `Token ID` that maps to the `Foundry ID`. Minting results in excess of tokens on the output
  side (positive diff), melting results in excess on the input side (negative diff). Now, the following conditions must
  hold for `Token Diff`:
  1. When `Token Diff` > 0
    - `Current State(Minted Tokens) + Token Diff = Next State(Minted Tokens)`.
    - `Current State(Melted Tokens) = Next State(Melted Tokens)`
  2. When `Token Diff` < 0, it must hold true that:
    - `Current State(Melted Tokens) <= Next State(Melted Tokens)`
    - `[Next State(Melted Tokens) - Current State(Melted Tokens)] <= |Token Diff|`.
    - When `Current State(Melted Tokens) != Next State(Melted Tokens)`, it must be true that `Current State(Minted Tokens) = Next State(Minted Tokens)`
  3. When `Current State` is empty, `Current State(Minted Tokens) = 0` and `Current State(Melted Tokens) = 0`.
  4. When `Next State` is empty, condition `1` and `2` are ignored. It must hold true, that
     `Current State(Minted Tokens) + Token Diff = Current State(Melted Tokens)`
- `Token Scheme Validation` takes `Token Diff` and `Foundry Diff` and validates if the scheme constraints are respected.
  This can include validating `Token Tag` part of the `Token IDs` and the `Token Scheme` fields inside the foundry
  output.
    - `Simple Token Scheme` validates that the `Token Tag` part of the `Token ID` (last 12 bytes) matches the
      `Token Tag` field of the foundry output.
    - Additional token schemes will be defined that make use of the `Foundry Diff` as well, for example validating that
      a certain amount of tokens can only be minted/melted after a certain date.

Check that created foundries are within the bounds of the associated alias ?

## NFT Output

### Additional Transaction Syntactic Validation Rules

#### Output Syntactic Validation

- `Amount` field must fulfill the dust protection requirements and must not be `0`.
- It must hold true that `1` ≤ `Unlock Conditions Count` ≤ `4`.
- `Unlock Condition Type` of an <i>Unlock Condition</i> must define on of the following types:
  - <i>Address Unlock Condition</i>
  - <i>Storage Deposit Return Unlock Condition</i>
  - <i>Timelock Unlock Condition</i>
  - <i>Expiration Unlock Condition</i>
- <i>Unlock Conditions</i> must be sorted in ascending order based on their `Unlock Condition Type`.
- <i>Address Unlock Condition</i> must be present.
- It must hold true that `0` ≤ `Blocks Count` ≤ `3`.
- `Block Type` of a <i>Block</i> in `Blocks` must define on of the following types:
  - <i>Sender Block</i>
  - <i>Metadata Block</i>
  - <i>Tag Block</i>
- It must hold true that `0` ≤ `Immutable Blocks Count` ≤ `2`.
- `Block Type` of a <i>Block</i> in `Immutable Blocks` must define on of the following types:
  - <i>Issuer Block</i>
  - <i>Metadata Block</i>
- <i>Blocks</i> must be sorted in ascending order based on their `Block Type` both in `Blocks` and `Immutable Blocks`
  fields.
- `Address` field of the <i>Address Unlock Condition</i> must not be the same as the NFT address derived from `NFT ID`.

### Additional Transaction Semantic Validation Rules

- Explicit `NFT ID`: `NFT ID` is taken as the value of the `NFT ID` field in the NFT output.
- Implicit `NFT ID`: When an NFT output is consumed as an input in a transaction and `NFT ID` field is zeroed out, take
  the BLAKE2b-160 hash of the `Output ID` of the input as `NFT ID`.
- For every non-zero explicit `NFT ID` on the output side there must be a corresponding NFT on the input side. The
  corresponding NFT has the explicit or implicit `NFT ID` equal to that of the NFT on the output side.

#### Consumed Outputs
- The unlock block of the input corresponds to `Address` field of the <i>Address Unlock Condition</i> and the unlock is
  valid.
- The unlock is valid if and only if all unlock conditions and feature blocks present in the output validate.
- When a consumed NFT output has no corresponding NFT output on the output side, the NFT it is being burned. Funds
  and assets inside the burned NFT output must be redistributed to other outputs in the burning transaction.

#### Created Outputs
- When `Issuer Block` is present in an output and explicit `NFT ID` is zeroed out, an input with `Address` field that
  corresponds to `Issuer` must be unlocked in the transaction. If `Address` is either <i>Alias Address</i> or
  <i>NFT Address</i>, their corresponding outputs (defined by `Alias ID` and `NFT ID`) must be unlocked in the transaction.
- All <i>Unlock Condition</i> imposed transaction validation criteria must be fulfilled.
- All <i>Feature Block</i> imposed transaction validation criteria must be fulfilled.

## Unlocking Chain Script Locked Outputs

### Alias Locking & Unlocking

A transaction may consume a (non-alias) output that belongs to an <i>Alias Address</i> by also consuming (and thus
unlocking) the alias output with the matching `Alias ID`. This serves the exact same purpose as providing a signature
to unlock an output locked under a private key backed address, such as <i>Ed25519 Addresses</i>.

This unlock block is similar to the <i>Reference Unlock Block</i>. However, it is valid if and only if the input of the
transaction at index `Alias Reference Unlock Index` is an alias output with the same `Alias ID` as the one derived from
the `Address` field of the to-be unlocked output.

Additionally, the <i>Alias Unlock Blocks</i> must also be ordered to prevent circular dependencies:

If the i-th *Unlock Block* of a transaction is an *Alias Unlock Block* and has `Alias Reference Unlock Index` set to k,
it must hold that i > k. Hence, an <i>Alias Unlock Block</i> can only reference an *Unlock Block* (unlocking the
corresponding alias) at a smaller index.

For example the scenario where `Alias A` is locked to the address of `Alias B` while `Alias B` is in locked to the
address of `Alias A` introduces a circular dependency and is not well-defined. By requiring the *Unlock Blocks* to be
ordered as described above, a transaction consuming `Alias A` as well as `Alias B` can never be valid as there would
always need to be one *Alias Unlock Block* referencing a greater index.

#### Alias Unlock Block Semantic Validation

 - The address of the unlocking condition of the input being unlocked must be an <i>Alias Address</i>.
 - The index `i` of the <i>Alias Unlock Block</i> is the index of the input in the transaction that it unlocks.
   `Alias Reference Unlock Index` must be < `i`.
 - `Alias Reference Unlock Index` defines a previous input of the transaction and its unlock block. This input must
   be an <i>Alias Output</i> with `Alias ID` that refers to the <i>Alias Address</i> being unlocked.
 - The referenced <i>Alias Output</i> must be unlocked for state transition.

### NFT Locking & Unlocking

`NFT ID` field is functionally equivalent to `Alias ID` of an alias output. It is generated the same way, but it can
only exist in NFT outputs. Following the same analogy as for alias addresses, NFT addresses are iota addresses that are
controlled by whoever owns the NFT output itself.

Outputs that are locked under `NFT Address` can be unlocked by unlocking the NFT output in the same transaction that
defines `NFT Address`, that is, the NFT output where `NFT Address Type Byte || NFT ID = NFT Address`.

An <i>NFT Unlock Block</i> looks and behaves like an <i>Alias Unlock Block</i>, but the referenced input at the index must
be an NFT output with the matching `NFT ID`.

An *NFT Unlock Block* is only valid if the input in the transaction at index `NFT Reference Unlock Index` is the NFT
output with the same `NFT ID` as the one derived from the `Address` field of the to-be unlocked output.

If the i-th *Unlock Block* of a transaction is an *NFT Unlock Block* and has `NFT Reference Unlock Index` set to k, it
must hold that i > k. Hence, an <i>NFT Unlock Block</i> can only reference an *Unlock Block* at a smaller index.

#### NFT Unlock Block Semantic Validation

 - The address of the input being unlocked must be an <i>NFT Address</i>.
 - The index `i` of the <i>NFT Unlock Block</i> is the index of the input in the transaction that it unlocks.
   `NFT Reference Unlock Index` must be < `i`.
 - `NFT Reference Unlock Index` defines a previous input of the transaction and its unlock block. This input must
   be an <i>NFT Output</i> with `NFT ID` that refers to the <i>NFT Address</i> being unlocked.
