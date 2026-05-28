# Mercury Indexer Configurations

This directory contains the configurations for the Mercury Indexer to sync FluxaPay smart contract events and data to the database.

## Configs
- `sync.yml`: Defines the DB sync configuration profiles (e.g., testnet_quick_sync, mainnet_full_sync, sandbox_local) and supports quick mappings for events to DB tables.

## Quick Mappings
Quick mappings allow mapping contract events to database tables automatically.
Enable `enable_quick_mappings` in the settings to use this feature.
