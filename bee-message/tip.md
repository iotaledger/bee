ce4feebb00f0dd7968b58334465651b95bd21722

### Native Tokens in Outputs

#### Additional syntactic output validation rules:

- `Native Tokens` must be lexicographically sorted based on `Token ID`.
- Each <i>Native Token</i> must be unique in the set of `Native Tokens` based on its `Token ID`. No duplicates are
  allowed.
- `Amount` of any <i>Native Token</i> must not be `0`.

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

This unlock condition makes it possible to send small amounts of IOTA coins or native tokens to addresses without having
to lose control of the required storage deposit. It is also a vehicle to send on-chain requests to ISCP chains that do not
require fees. To prevent the receiving party from blocking access to the storage deposit, it is advised to be used
together with the [Expiration Unlock Conditions](#expiration-unlock-conditions). The receiving party then has a sender-defined
time window to agree to the transfer by consuming the output, or the sender regains total control after expiration.

##### Timelock Unlock Conditions

The notion of time in the Tangle is introduced via milestones. Each milestone
[carries the current milestone index and the unix timestamp](../TIP-0008/tip-0008.md#structure)
corresponding to that index. Whenever a new milestone appears, nodes perform the white-flag ordering and transaction
validation on its past cone. The timestamp and milestone index of the confirming milestone provide the time as an input
parameter to transaction validation.

An output that contains a <i>Timelock Unlock Condition</i> can not be unlocked before the specified timelock has
expired. The timelock is expired when the timestamp and/or milestone index of the confirming milestone is equal or past
the timestamp and/or milestone defined in the <i>Timelock Unlock Condition</i>.

The timelock can be specified as a unix timestamp or as a milestone index. When specified in both ways, both conditions
have to pass in order for the unlock to be valid. The zero value of one if the timestamp fields signals that it should be
ignored during validation.

The two time representations help to protect against the possible downtime of the Coordinator. If the Coordinator is
down, "milestone index clock" essentially stops advancing, while "real time clock" does not. An output that specifies
time in both clocks must satisfy both conditions (AND relation).

###### Additional syntactic transaction validation rules:
- If both `Milestone Index` and `Unix Time` fields are `0`, the unlock condition, and hence the output and transaction
  that contain it, is invalid.
###### Additional semantic transaction validation rules:
- An output that has <i>Timelock Unlock Condition</i> specified must only be consumed and unlocked in a
  transaction, if the confirming milestone index is ≥ than the `Milestone Index` specified in the unlock condition.
- An output that has <i>Timelock Unlock Condition</i> specified must only be consumed and unlocked in a
  transaction, if the timestamp of the confirming milestone is equal or past the `Unix Time` specified in the unlock
  condition.

##### Expiration Unlock Conditions

The expiration feature of outputs makes it possible for the return address to reclaim an output after a given expiration
time has been passed. The expiration might be specified as a unix timestamp or as a milestone index. When specified in
both ways, both conditions have to pass in order for the unlock to be valid.

The expiration feature can be viewed as an opt-in receive feature, because the recipient loses access to the received
funds after the output expires, while the return address specified by the sender regains control over them. This feature
is a big help for on-chain smart contract requests. Those that have expiration set and are sent to dormant smart contract
chains can be recovered by their senders. Not to mention the possibility to time requests by specifying both a
timelock and an expiration unlock condition.

###### Additional syntactic transaction validation rules:
- If both `Milestone Index` and `Unix Time` fields are `0`, the unlock condition, and hence the output and transaction
  that contain it, is invalid.

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

##### State Controller Address Unlock Condition

An unlock condition defined solely for <i>Alias Output</i>. It is functionally equivalent to an
<i>Address Unlock Condition</i>, however there are additional transition constraints defined for the Alias UTXO state
machine that can only be carried out by the `State Controller Address`, hence the distinct unlock condition type.

The additional constraints are defined in [Alias Output Design](#alias-output) section.

##### Governor Address Unlock Condition

An unlock condition defined solely for <i>Alias Output</i>. It is functionally equivalent to an
<i>Address Unlock Condition</i>, however there are additional transition constraints defined for the Alias UTXO state
machine that can only be carried out by the `Governor Address`, hence the distinct unlock condition type.

The additional constraints are defined in [Alias Output Design](#alias-output) section.

##### Immutable Alias Address Unlock Condition

An unlock condition defined for chain constrained UTXOs that can only be unlocked by a permanent <i>Alias Address</i>.

Output unlocking is functionally equivalent to an <i>Address Unlock Condition</i> with an <i>Alias Address</i>,
however there are additional transition constraints: the next state of the UTXO machine must have the same
<i>Immutable Alias Address Unlock Condition</i>.

###### Additional semantic transaction validation rules:
 - The output must be unlocked with an <i>[Alias Unlock Block](#alias-unlock-block-semantic-validation)</i>.
 - The next state of the UTXO state machine must have the same <i>Immutable Alias Address Unlock Condition</i> defined.

### Feature Blocks

New output features that do not introduce unlocking conditions, but rather add new functionality and add constraints on
output creation are grouped under <i>Feature Blocks</i>.

Each output **must not contain more than one block of each type** and not all block types are supported for each output
type.

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

The main use case is proving authenticity of NFTs. Whenever an NFT is minted as an NFT output, the creator (issuer) can
fill the <i>Issuer Block</i> with their address that they have to unlock in the transaction. Issuers then can publicly
disclose their addresses to prove the authenticity of the NFT once it is in circulation.

Whenever a chain account mints an NFT on layer 1 on behalf of some user, the `Issuer` field can only contain the
chain's address, since user does not sign the layer 1 transaction. As a consequence, artist would have to mint NFTs
themselves on layer 1 and then deposit it to chains if they want to place their own address in the `Issuer` field.

##### Metadata Block

Outputs may carry additional data with them that is interpreted by higher layer applications built on the Tangle. The
protocol treats this metadata as pure binary data, it has no effect on the validity of an output except that it
increases the required storage deposit. ISC is a great example of a higher layer protocol that makes use of
<i>Metadata Block</i>: smart contract request parameters are encoded in the metadata field of outputs.

###### Additional syntactic transaction validation rules:
- An output with <i>Metadata Block</i> is valid, if and only if 0 < `Data Length` ≤ `Max Metadata Length`.

#### Tag Block

A <i>Tag Block</i> makes it possible to tag outputs with an index, so they can be retrieved through an indexer API not
only by their address, but also based on the the `Tag`. **The combination of a <i>Tag Block</i>, a
<i>Metadata Block</i> and a <i>Sender Block</i> makes it possible to retrieve data associated to an address and stored
in outputs that were created by a specific party (`Sender`) for a specific purpose (`Tag`).**

An example use case is voting on the Tangle via the [participation](https://github.com/iota-community/treasury/blob/main/specifications/hornet-participation-plugin.md) plugin.

Storing indexed data in outputs however incurs greater storage deposit for such outputs, because they create look-up
entries in nodes' databases.

##### Additional syntactic transaction validation rules:
- An output with <i>Tag Block</i> is valid, if and only if 0 < `Tag Length` ≤
  `Max Tag Length`.

### Chain Constraint in UTXO

Previously created transaction outputs are destroyed when they are consumed in a subsequent transaction as an input.
The chain constraint makes it possible to **carry the UTXO state machine state encoded in outputs across transactions.**
When an output with chain constraint is consumed, that transaction has to create a single subsequent output that
carries the state forward. The **state can be updated according to the transition rules defined for the given type of
output and its current state**. As a consequence, each such output has a unique successor, and together they form a path
or *chain* in the graph induced by the UTXO spends. Each chain is identified by its globally unique identifier.

## Output Design

In the following, we define four new output types. They are all designed with specific use cases in mind:
- **Basic Output**: transfer of funds with attached metadata and optional spending restrictions. Main use cases are
  on-ledger ISC requests, native asset transfers and indexed data storage in the UTXO ledger.
- **Alias Output**: representing ISC chain accounts on L1 that can process requests and transfer funds.
- **Foundry Output**: supply control of user defined native tokens. A vehicle for cross-chain asset transfers and asset
  wrapping.
- **NFT Output**: an output that represents a Non-fungible token with attached metadata and proof-of-origin. A NFT is
  represented as an output so that the token and metadata are transferred together, for example as a smart contract
  requests. NFTs are possible to implement with native tokens as well, but then ownership of the token does not mean
  ownership of the foundry that holds its metadata.

The validation of outputs is part of the transaction validation process. There are two levels of validation for
transactions: syntactic and semantic validation. The former validates the structure of the transaction (and outputs),
while the latter validates whether protocol rules are respected in the semantic context of the transaction. Outputs
hence are validated on both levels:
1. **Transaction Syntactic Validation**: validates the structure of each output created by the transaction.
2. **Transaction Semantic Validation**:
    - **For consumed outputs**: validates whether the output can be unlocked in a transaction given the semantic
      transaction context.
    - **For created outputs**: validates whether the output can be created in a transaction given the semantic
      transaction context.

Each new output type may add its own validation rules which become part of the transaction validation rules if the
output is placed inside a transaction. <i>Unlock Conditions</i> and <i>Feature Blocks</i> described previously also add
constraints to transaction validation when they are placed in outputs.

## Basic Output

### Additional Transaction Syntactic Validation Rules

- `Amount` field must fulfill the dust protection requirements and must not be `0`.
- `Amount` field must be ≤ `Max IOTA Supply`.
- `Native Tokens Count` must not be greater than `Max Native Tokens Count`.
- `Native Tokens` must be lexicographically sorted based on `Token ID`.
- Each <i>Native Token</i> must be unique in the set of `Native Tokens` based on its `Token ID`. No duplicates are
  allowed.
- `Amount` of any <i>Native Token</i> must not be `0`.
- It must hold true that `1` ≤ `Unlock Conditions Count` ≤ `4`.
- `Unlock Condition Type` of an <i>Unlock Condition</i> must define on of the following types:
  - <i>Address Unlock Condition</i>
  - <i>Storage Deposit Return Unlock Condition</i>
  - <i>Timelock Unlock Condition</i>
  - <i>Expiration Unlock Condition</i>
- <i>Unlock Conditions</i> must be sorted in ascending order based on their `Unlock Condition Type`.
- Syntactic validation of all present unlock conditions must pass.
- <i>Address Unlock Condition</i> must be present.
- It must hold true that `0` ≤ `Blocks Count` ≤ `3`.
- `Block Type` of a <i>Block</i> must define on of the following types:
  - <i>Sender Block</i>
  - <i>Metadata Block</i>
  - <i>Tag Block</i>
- <i>Blocks</i> must be sorted in ascending order based on their `Block Type`.
- Syntactic validation of all present feature blocks must pass.

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
- `Amount` field must be ≤ `Max IOTA Supply`.
- `Native Tokens Count` must not be greater than `Max Native Tokens Count`.
- `Native Tokens` must be lexicographically sorted based on `Token ID`.
- Each <i>Native Token</i> must be unique in the set of `Native Tokens` based on its `Token ID`. No duplicates are
  allowed.
- `Amount` of any <i>Native Token</i> must not be `0`.
- It must hold true that `Unlock Conditions Count` = `2`.
- `Unlock Condition Type` of an <i>Unlock Condition</i> must define one of the following types:
  - <i>State Controller Address Unlock Condition</i>
  - <i>Governor Address Unlock Condition</i>
- <i>Unlock Conditions</i> must be sorted in ascending order based on their `Unlock Condition Type`.
- Syntactic validation of all present unlock conditions must pass.
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
- Syntactic validation of all present feature blocks must pass.
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
- `Amount` field must be ≤ `Max IOTA Supply`.
- `Native Tokens Count` must not be greater than `Max Native Tokens Count`.
- `Native Tokens` must be lexicographically sorted based on `Token ID`.
- Each <i>Native Token</i> must be unique in the set of `Native Tokens` based on its `Token ID`. No duplicates are
  allowed.
- `Amount` of any <i>Native Token</i> must not be `0`.
- It must hold true that `Unlock Conditions Count` = `1`.
- `Unlock Condition Type` of an <i>Unlock Condition</i> must define on of the following types:
  - <i>Immutable Alias Address Unlock Condition</i>
- It must hold true that `0` ≤ `Blocks Count` ≤ `1`.
- `Block Type` of a <i>Block</i> in `Blocks` must define on of the following types:
  - <i>Metadata Block</i>
- It must hold true that `0` ≤ `Immutable Blocks Count` ≤ `1`.
- `Block Type` of a <i>Block</i> in `Immutable Blocks` must define on of the following types:
  - <i>Metadata Block</i>
- Syntactic validation of all present feature blocks must pass.
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

The NFT may contain immutable metadata set upon creation, and a verified `Issuer`. The output type supports all
non-alias specific (state controller, governor) unlock conditions and optional feature blocks so that the output can be
sent as a request to smart contract chain accounts.

### Additional Transaction Syntactic Validation Rules

#### Output Syntactic Validation

- `Amount` field must fulfill the dust protection requirements and must not be `0`.
- `Amount` field must be ≤ `Max IOTA Supply`.
- `Native Tokens Count` must not be greater than `Max Native Tokens Count`.
- `Native Tokens` must be lexicographically sorted based on `Token ID`.
- Each <i>Native Token</i> must be unique in the set of `Native Tokens` based on its `Token ID`. No duplicates are
  allowed.
- `Amount` of any <i>Native Token</i> must not be `0`.
- It must hold true that `1` ≤ `Unlock Conditions Count` ≤ `4`.
- `Unlock Condition Type` of an <i>Unlock Condition</i> must define on of the following types:
  - <i>Address Unlock Condition</i>
  - <i>Storage Deposit Return Unlock Condition</i>
  - <i>Timelock Unlock Condition</i>
  - <i>Expiration Unlock Condition</i>
- <i>Unlock Conditions</i> must be sorted in ascending order based on their `Unlock Condition Type`.
- Syntactic validation of all present unlock conditions must pass.
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
- Syntactic validation of all present feature blocks must pass.
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

Two of the introduced output types ([Alias](#alias-output), [NFT](#nft-output)) implement the so-called UTXO chain
constraint. These outputs receive their unique identifiers upon creation, generated by the protocol, and carry it
forward with them through transactions until they are destroyed. These unique identifiers (`Alias ID`, `NFT ID`) also
function as global addresses for the state machines, but unlike <i>Ed25519 Addresses</i>, they are not backed by private
keys that could be used for signing. The rightful owners who can unlock these addresses are defined in the outputs
themselves.

Since such addresses are accounts in the ledger, it is possible to send funds to these addresses. The unlock mechanism
of such funds is designed in a way that **proving ownership of the address is reduced to the ability to unlock the
corresponding output that defines the address.**

### Alias Locking & Unlocking

A transaction may consume a (non-alias) output that belongs to an <i>Alias Address</i> by also consuming (and thus
unlocking) the alias output with the matching `Alias ID`. This serves the exact same purpose as providing a signature
to unlock an output locked under a private key backed address, such as <i>Ed25519 Addresses</i>.

On protocol level, alias unlocking is done using a new unlock block type, called **Alias Unlock Block**.

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

#### Alias Unlock Block Syntactic Validation

 - It must hold that 0 ≤ `Alias Reference Unlock Index` < `Max Inputs Count`.

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

#### NFT Unlock Block Syntactic Validation

- It must hold that 0 ≤ `NFT Reference Unlock Index` < `Max Inputs Count`.

#### NFT Unlock Block Semantic Validation

 - The address of the input being unlocked must be an <i>NFT Address</i>.
 - The index `i` of the <i>NFT Unlock Block</i> is the index of the input in the transaction that it unlocks.
   `NFT Reference Unlock Index` must be < `i`.
 - `NFT Reference Unlock Index` defines a previous input of the transaction and its unlock block. This input must
   be an <i>NFT Output</i> with `NFT ID` that refers to the <i>NFT Address</i> being unlocked.
