# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- ## Unreleased - YYYY-MM-DD

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security -->

## 0.1.0 - 2021-04-21

### Added
- Routes;
  - `health` route;
  - `plugins/debug/white_flag` route;
  - `plugins/v1/add_peer` route;
  - `plugins/v1/balance_bech32` route;
  - `plugins/v1/balance_ed25519` route;
  - `plugins/v1/info` route;
  - `plugins/v1/message` route;
  - `plugins/v1/message_children` route; 
  - `plugins/v1/message_metadata` route; 
  - `plugins/v1/message_raw` route;
  - `plugins/v1/messages_find` route;
  - `plugins/v1/milestone` route;  
  - `plugins/v1/milestone_utxo_changes` route;    
  - `plugins/v1/output` route;     
  - `plugins/v1/outputs_bech32` route;    
  - `plugins/v1/outputs_ed25519` route;  
  - `plugins/v1/peer` route;    
  - `plugins/v1/peers` route;      
  - `plugins/v1/receipts` route;        
  - `plugins/v1/receips_at` route;   
  - `plugins/v1/remove_peer` route;     
  - `plugins/v1/submit_message` route;    
  - `plugins/v1/submit_message_raw` route;  
  - `plugins/v1/tips` route;    
  - `plugins/v1/transaction_inculde_message` route;      
  - `plugins/v1/treasury` route;  
  
## 0.1.0 - 2021-04-20

### Added

- Types;
  - `BodyInner` type;
  - `SuccessBody` type;
  - `ErrorBody` type;
  - `DefaultErrorResponse` type;
  - `MessageDto` type;
  - `PayloadDto` type;
  - `TransactionPayloadDto` type;
  - `EssenceDto` type;
  - `RegularEssenceDto` type;
  - `InputDto` type;
  - `UtxoInputDto` type;
  - `TreasuryInputDto` type;
  - `OutputDto` type;
  - `SignatureLockedSingleOutputDto` type;
  - `SignatureLockedDustAllowanceOutputDto` type;
  - `AddressDto` type;
  - `Ed25519AddressDto` type;
  - `TreasuryOutputDto` type;
  - `UnlockBlockDto` type;
  - `SignatureUnlockDto` type;
  - `SignatureDto` type;
  - `Ed25519SignatureDto` type;
  - `ReferenceUnlockDto` type;
  - `MilestonePayloadDto` type;
  - `IndexationPayloadDto` type;
  - `ReceiptPayloadDto` type;
  - `MigratedFundsEntryDto` type;
  - `TreasuryTransactionPayloadDto` type;
  - `PeerDto` type;
  - `GossipDto` type;
  - `RelationDto` type;
  - `HeartbeatDto` type;
  - `MetricsDto` type;
  - `ReceiptDto` type;
  - `LedgerInclusionStateDto` type;
  - `Error` type;
  - `InfoResponse` type;
  - `TipsResponse` type;
  - `SubmitMessageResponse` type;
  - `MessagesFindResponse` type;
  - `MessageResponse` type;
  - `MessageMetadataResponse` type;
  - `MessageChildrenResponse` type;
  - `OutputResponse` type;
  - `BalanceAddressResponse` type;
  - `OutputsAddressResponse` type;
  - `ReceiptsResponse` type;
  - `TreasuryResponse` type;
  - `MilestoneResponse` type;
  - `UtxoChangesResponse` type;
  - `PeersResponse` type;
  - `AddPeerResponse` type;
  - `PeerResponse` type;
  - `WhiteFlagResponse` type;
