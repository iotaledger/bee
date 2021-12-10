- `Amount` field must fulfill the dust protection requirements.

| Minimum Dust Deposit | uint64 | TBD | Minimum amount of IOTA coins that need to be present in the smallest possible output in order not to be considered dust. |
| Max Metadata Length | uint32 | TBD | Maximum possible length in bytes of a `Metadata` field. |

####Native Tokens in Outputs Additional semantic transaction validation rules:
- When the transaction is imbalanced, the foundry outputs controlling outstanding native token balances must be present
  in the transaction. The validation of the foundry output(s) determines if the outstanding balances are valid.

#### Issuer Block
Only when the state machine is created (e.g. minted) it is checked during transaction validation that an output
corresponding to the `Issuer` address is consumed. In every future transition of the state machine, it is instead
checked that the issuer block is still present and unchanged.
##### Additional semantic transaction validation rule:
- When an <i>Issuer Block</i> is present in an output representing the initial state of an UTXO state machine, the
  transaction that contains this output is valid, if and only if an output with the corresponding address is consumed
  and unlocked in the transaction.

#### Dust Deposit Return Block
An output that has both <i>Sender Block</i> and <i>Dust Deposit Return Block</i>  specified can only be consumed in a transaction that
deposits `Return Amount` IOTA coins into `Sender` address. When several of such outputs are consumed, their return
amounts per `Sender` addresses are summed up and the output side must deposit this total sum per `Sender` address.

##### Additional syntactic transaction validation rule:
- `Return Amount` in a <i>Dust Deposit Return Block</i> must be ≤ than the required dust deposit of the output.

##### Additional semantic transaction validation rule:
- An output that has <i>Dust Deposit Return Block</i> specified must only be consumed and unlocked in a transaction that
  deposits `Return Amount` IOTA coins to `Sender` address via an output that has no additional spending constraints.
  (<i>ExtendedOutput</i> without feature blocks)
- When several outputs with <i>Dust Deposit Return Block</i> and the same `Sender` are consumed, their return amounts
  per `Sender` addresses are summed up and the output side of the transaction must deposit this total sum per `Sender`
  address.

This feature block makes it possible to send small amounts of IOTA coins or native tokens to addresses without having
to lose control of the required dust deposit. It is also a vehicle to send on-chain requests to ISCP chains that do not
require fees. To prevent the receiving party from blocking access to the dust deposit, it is advised to be used
together with the [Expiration Blocks](#Expiration-Blocks). The receiving party then has a sender-defined time window to
agree to the transfer by consuming the output, or the sender regains total control after expiration.

#### Expiration Blocks
The expiration feature of outputs makes it possible for the sender to reclaim an output after a given expiration time
has been passed. As a consequence, expiration blocks may only be used in the presence of the <i>Sender Block</i>.

The expiration feature can be viewed as an opt-in receive feature, because the recipient loses access to the received
funds after the output expires, while the sender regains control over them. This feature is a big help for on-chain
smart contract requests. Those that have expiration set and are sent to dormant smart contract chains can be recovered
by their senders. Not to mention the possibility to time requests by specifying both a timelock and an expiration block.

##### Additional semantic transaction validation rules:
- An output that has <i>Expiration Milestone Index Block</i> set must only be consumed and unlocked by the target
  `Address` in a transaction that has a confirming milestone index < than the `Milestone Index` defined in the block.
- An output that has <i>Expiration Milestone Index Block</i> set must only be consumed and unlocked by the `Sender`
  address in a transaction that has a confirming milestone index ≥ than the `Milestone Index` defined in the block.
- An output that has <i>Expiration Unix Block</i> set must only be consumed and unlocked by the target `Address` in a
  transaction that has a confirming milestone timestamp earlier than the `Unix Time` defined in the block.
- An output that has <i>Expiration Unix Block</i> set must only be consumed and unlocked by the `Sender` address in a
  transaction that has a confirming milestone timestamp same or later than the `Unix Time` defined in the block.
- Semantic validation of an output that has either <i>Expiration Milestone Index Block</i> or
  <i>Expiration Unix Block</i> set and is unlocked by the `Sender` address must ignore:
  - [Semantic validation of <i>Dust Deposit Return Block</i>](#dust-deposit-return-block) if present.

## Extended Output Consumed Outputs
- The unlock block of the input must correspond to `Address` field and the unlock must be valid.

## Alias Output

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
- State transition:
    - A state transition is identified by an incremented `State Index`.
    - The `State Index` must be incremented by 1.
    - The unlock block must correspond to the `State Controller`.
    - State transition can only change the following fields in the next state: `IOTA Amount`, `Native Tokens`,
      `State Index`, `State Metadata Length`, `State Metadata` and `Foundry Counter`.
    - `Foundry Counter` field must increase by the number of foundry outputs created in the transaction that map to
      `Alias ID`. The `Serial Number` fields of the created foundries must be the set of natural numbers that cover the
       open-ended interval between the previous and next values of the `Foundry Counter` field in the alias output.
    - The created foundry outputs must be sorted in the list of outputs by their `Serial Number`. Note, that any
      foundry that maps to `Alias ID` and has a `Serial Number` that is less or equal to the `Foundry Counter` of the
      input alias is ignored when it comes to sorting.
    - Newly created foundries in the transaction that map to different aliases can be interleaved when it comes to
      sorting.
- Governance transition:
    - A governance transition is identified by an unchanged `State Index` in next state. If there is no alias output on
      the output side with a corresponding explicit `Alias ID`, the alias is being destroyed. The next state is the
      empty state.
    - The unlock block must correspond to the `Governance Controller`.
    - Governance transition must only change the following fields: `State Controller`, `Governance Controller`,
      `Metadata Block`.
    - The `Metadata Block` is optional, the governor can put additional info about the chain here, for example chain
      name, fee structure, supported VMs, list of access nodes, etc., anything that helps clients to fetch info (i.e.
      account balances) about the layer 2 network.
- When a consumed alias output has an `Issuer Block` and a corresponding alias output on the output side,
  `Issuer Block` is not allowed to change.

#### Created Outputs
- When <i>Issuer Block</i> is present in an output and explicit `Alias ID` is zeroed out, an input with `Address` field
  that corresponds to `Issuer` must be unlocked in the transaction.

### Notes
- Nodes shall map the alias address of the output derived with `Alias ID` to the regular <i>address -> output</i>
  mapping table, so that given an <i>Alias Address</i>, its most recent unspent alias output can be retrieved.

## Foundry Output
**The concatenation of `Address` || `Serial Number` || `Token Scheme Type` fields defines the unique identifier of the
foundry, the `Foundry ID`.**

Upon creation of the foundry, the alias defined in the `Address` field must be unlocked in the same transaction, and
its `Foundry Counter` field must increment. This incremented value defines `Serial Number`, while the `Token Scheme`
can be chosen freely.

`Foundry ID` is not allowed to change after deployment, therefore neither `Address`, nor `Serial Number` or
`Token Scheme` can change during the lifetime of the foundry.

Foundries control the supply of tokens with unique identifiers, so-called `Token IDs`. The **`Token ID` of tokens
controlled by a specific foundry is the concatenation of `Foundry ID` || `Token Tag`.**

### Additional Transaction Semantic Validation Rules
A foundry is essentially a UTXO state machine. A transaction might either create a new foundry with a unique
`Foundry ID`, transition an already existing foundry or destroy it. The current and next states of the state machine
are encoded in inputs and outputs respectively.

| A transaction that... | Current State | Next State |
| --------------------- | --------------| -----------|
| Creates the foundry | Empty | Output with `Foundry ID` |
| Transitions the foundry | Input with `Foundry ID` | Output with `Foundry ID` |
| Destroys the foundry | Input with `Foundry ID` | Empty |

- The foundry output must be unlocked like any other output type belonging to an <i>Alias Address</i>, by transitioning
  the alias in the very same transaction.
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
  side (positive diff), burning results in excess on the input side (negative diff). Now, the following conditions must
  hold for `Token Diff`:
    - `Current State(Circulating Supply) + Token Diff = Next State(Circulating Supply)`.
    - When `Current State` is empty, `Current State(Circulating Supply) = 0`.
    - When `Next State` is empty, `Next State(Circulating Supply) = 0`.
- `Token Scheme Validation` takes `Token Diff` and `Foundry Diff` and validates if the scheme constraints are respected.
  This can include validating `Token Tag` part of the `Token IDs` and the `Token Scheme` fields inside the foundry
  output.
    - `Simple Token Scheme` validates that the `Token Tag` part of the `Token ID` (last 12 bytes) matches the
      `Token Tag` field of the foundry output.
    - Additional token schemes will be defined that make use of the `Foundry Diff` as well, for example validating that
      a certain amount of tokens can only be minted/burned after a certain date.
- When neither `Current State` nor `Next State` is empty:
    -  `Maximum Suppply` field must not change.
    -  `Address` must not change.
    -  `Serial Number` must not change.
    -  `Token Tag` must not change.
    -  `Token Scheme Type` must not change.

### Notes
- The `Foundry ID` of a foundry output should have a global mapping table in nodes, so that given a `Foundry ID`, the
  `Output ID` of the foundry output can be retrieved. `Foundry ID` behaves like an address that can't unlock anything.
  While it is not neccessarly needed for the protocol, it is needed for client side operations (what is the current
  state of the foundry? accessing token metadata in foundry based on `Foundry ID` derived from `Tokend ID`).

## NFT Output
Each NFT output gets assigned a unique identifier `NFT ID` upon creation by the protocol. `NFT ID` is BLAKE2b-160 hash
of the <i>Output ID</i>  that created the NFT. The address of the NFT is the concatenation of `NFT Address Type` ||
`NFT ID`.

### Additional Transaction Semantic Validation Rules
- Explicit `NFT ID`: `NFT ID` is taken as the value of the `NFT ID` field in the NFT output.
- Implicit `NFT ID`: When an NFT output is consumed as an input in a transaction and `NFT ID` field is zeroed out, take
  the BLAKE2b-160 hash of the `Output ID` of the input as `NFT ID`.
- For every non-zero explicit `NFT ID` on the output side there must be a corresponding NFT on the input side. The
  corresponding NFT has the explicit or implicit `NFT ID` equal to that of the NFT on the output side.

#### Consumed Outputs
- The unlock block of the input corresponds to `Address` field and the unlock is valid.
- When a consumed NFT output has a corresponding NFT output on the output side, `Immutable Metadata Length` and
  `Immutable Data` fields are not allowed to change.
- When a consumed NFT output has an `Issuer Block` and a corresponding NFT output on the output side, `Issuer Block` is
  not allowed to change.
- When a consumed NFT output has no corresponding NFT output on the output side, the NFT it is being burned. Funds
  and assets inside the burned NFT output must be redistributed to other outputs in the burning transaction.

#### Created Outputs
- When `Issuer Block` is present in an output and explicit `NFT ID` is zeroed out, an input with `Address` field that
  corresponds to `Issuer` must be unlocked in the transaction.

## Unlocking Chain Script Locked Outputs
The unlock mechanism of such funds is designed in a way that **proving ownership of the address is reduced to the ability
to unlock the corresponding output that defines the address.**

### Alias Locking & Unlocking
This unlock block is similar to the <i>Reference Unlock Block</i>. However, it is valid if and only if the input of the
transaction at index `Alias Reference Unlock Index` is an alias output with the same `Alias ID` as the `Address` field
of the to-be unlocked output.

If the i-th *Unlock Block* of a transaction is an *Alias Unlock Block* and has `Alias Reference Unlock Index` set to k,
it must hold that i > k. Hence, an <i>Alias Unlock Block</i> can only reference an *Unlock Block* (unlocking the
corresponding alias) at a smaller index.

For example the scenario where `Alias A` is locked to the address of `Alias B` while `Alias B` is in locked to the
address of `Alias A` introduces a circular dependency and is not well-defined. By requiring the *Unlock Blocks* to be
ordered as described above, a transaction consuming `Alias A` as well as `Alias B` can never be valid as there would
always need to be one *Alias Unlock Block* referencing a greater index.

#### Alias Unlock Block Semantic Validation
 - The address of the input being unlocked must be an <i>Alias Address</i>.
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
defines `NFT Address`, that is, the NFT output where `NFT ID = NFT Address`.

An **NFT Unlock Block** looks and behaves like an <i>Alias Unlock Block</i>, but the referenced input at the index must
be an NFT output with the matching `NFT ID`.

An *Alias Unlock Block* is only valid if the input in the transaction at index `NFT Reference Unlock Index` is the NFT
output with the same `NFT ID` as the `Address` field of the to-be unlocked output.

If the i-th *Unlock Block* of a transaction is an *NFT Unlock Block* and has `NFT Reference Unlock Index` set to k, it
must hold that i > k. Hence, an <i>NFT Unlock Block</i> can only reference an *Unlock Block* at a smaller index.

#### NFT Unlock Block Semantic Validation
 - The address of the input being unlocked must be an <i>NFT Address</i>.
 - The index `i` of the <i>NFT Unlock Block</i> is the index of the input in the transaction that it unlocks.
   `NFT Reference Unlock Index` must be < `i`.
 - `NFT Reference Unlock Index` defines a previous input of the transaction and its unlock block. This input must
   be an <i>NFT Output</i> with `NFT ID` that refers to the <i>NFT Address</i> being unlocked.
