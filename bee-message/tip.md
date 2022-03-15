ce4feebb00f0dd7968b58334465651b95bd21722

### Native Tokens in Outputs Additional semantic transaction validation rules:
- The transaction is balanced in terms of native tokens, that is, the sum of native token balances in consumed outputs equals that of the created outputs.
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

##### Expiration Unlock Conditions Additional semantic transaction validation rules:
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

##### Immutable Alias Address Unlock Condition Additional semantic transaction validation rules:
 - The output must be unlocked with an <i>[Alias Unlock Block](#alias-unlock-block-semantic-validation)</i>.
 - The next state of the UTXO state machine must have the same <i>Immutable Alias Address Unlock Condition</i> defined.

## Basic Output

### Additional Transaction Syntactic Validation Rules
- `Amount` field must fulfill the dust protection requirements and must not be `0`.

#### Consumed Outputs
- The unlock is valid if and only if all unlock conditions and feature blocks present in the output validate.

#### Created Outputs
- All <i>Unlock Condition</i> imposed transaction validation criteria must be fulfilled.
- All <i>Feature Block</i> imposed transaction validation criteria must be fulfilled.

## Alias Output

### Additional Transaction Syntactic Validation Rules
- `Amount` field must fulfill the dust protection requirements and must not be `0`.

### Additional Transaction Semantic Validation Rules
- For every non-zero explicit `Alias ID` on the output side there must be a corresponding alias on the input side. The
  corresponding alias has the explicit or implicit `Alias ID` equal to that of the alias on the output side.

## Foundry Output
Upon creation of the foundry, the alias defined in the `Address` field of the
<i>Immutable Alias Address Unlock Condition</i> must be unlocked in the same transaction, and its `Foundry Counter`
field must increment. This incremented value defines `Serial Number`, while the `Token Scheme` can be chosen freely.

### Additional Transaction Syntactic Validation Rules
- `Amount` field must fulfill the dust protection requirements and must not be `0`.

### Additional Transaction Semantic Validation Rules
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

## NFT Output

#### Output Syntactic Validation
- `Amount` field must fulfill the dust protection requirements and must not be `0`.

### Additional Transaction Semantic Validation Rules
- For every non-zero explicit `NFT ID` on the output side there must be a corresponding NFT on the input side. The
  corresponding NFT has the explicit or implicit `NFT ID` equal to that of the NFT on the output side.

#### Consumed Outputs
- The unlock is valid if and only if all unlock conditions and feature blocks present in the output validate.
- When a consumed NFT output has no corresponding NFT output on the output side, the NFT it is being burned. Funds
  and assets inside the burned NFT output must be redistributed to other outputs in the burning transaction.

#### Created Outputs
- All <i>Unlock Condition</i> imposed transaction validation criteria must be fulfilled.
- All <i>Feature Block</i> imposed transaction validation criteria must be fulfilled.

#### Alias Unlock Block Semantic Validation
 - The address of the unlocking condition of the input being unlocked must be an <i>Alias Address</i>.
 - The index `i` of the <i>Alias Unlock Block</i> is the index of the input in the transaction that it unlocks. `Alias Reference Unlock Index` must be < `i`.
 - `Alias Reference Unlock Index` defines a previous input of the transaction and its unlock block. This input must
   be an <i>Alias Output</i> with `Alias ID` that refers to the <i>Alias Address</i> being unlocked.
 - The referenced <i>Alias Output</i> must be unlocked for state transition.

#### NFT Unlock Block Semantic Validation
 - The address of the input being unlocked must be an <i>NFT Address</i>.
 - The index `i` of the <i>NFT Unlock Block</i> is the index of the input in the transaction that it unlocks. `NFT Reference Unlock Index` must be < `i`.
 - `NFT Reference Unlock Index` defines a previous input of the transaction and its unlock block. This input must
   be an <i>NFT Output</i> with `NFT ID` that refers to the <i>NFT Address</i> being unlocked.
