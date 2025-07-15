#!/usr/bin/env python3
"""
Genesis generation script for Gravity blockchain.
This script combines all the generated files into a final genesis.json.
"""

import json
import sys
import os
from typing import Dict, Any
import argparse


def load_json_file(file_path: str) -> Dict[str, Any]:
    """Load and parse a JSON file."""
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"❌ Error: File {file_path} not found")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"❌ Error: Invalid JSON in {file_path}: {e}")
        sys.exit(1)


def create_genesis_json(
    accounts_file: str = "output/genesis_accounts.json",
    contracts_file: str = "output/genesis_contracts.json",
    bundle_state_file: str = "output/bundle_state.json",
    output_file: str = "genesis.json"
) -> None:
    """
    Create the final genesis.json file by combining all generated files.
    
    Args:
        accounts_file: Path to genesis_accounts.json
        contracts_file: Path to genesis_contracts.json  
        bundle_state_file: Path to bundle_state.json
        output_file: Output genesis.json file path
    """
    print("🔄 Starting genesis.json generation...")
    
    # Load all input files
    print(f"📖 Loading accounts from {accounts_file}")
    accounts = load_json_file(accounts_file)
    
    print(f"📖 Loading contracts from {contracts_file}")
    contracts = load_json_file(contracts_file)
    
    print(f"📖 Loading bundle state from {bundle_state_file}")
    bundle_state = load_json_file(bundle_state_file)
    
    # Create the genesis structure
    genesis = {
        "config": {
            "chainId": 1,
            "homesteadBlock": 0,
            "eip150Block": 0,
            "eip150Hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "eip155Block": 0,
            "eip158Block": 0,
            "byzantiumBlock": 0,
            "constantinopleBlock": 0,
            "petersburgBlock": 0,
            "istanbulBlock": 0,
            "muirGlacierBlock": 0,
            "berlinBlock": 0,
            "londonBlock": 0,
            "arrowGlacierBlock": 0,
            "grayGlacierBlock": 0,
            "mergeForkBlock": 0,
            "shanghaiTime": 0,
            "cancunTime": 0,
            "gravity": {
                "gravityBlock": 0
            }
        },
        "nonce": "0x0000000000000000",
        "timestamp": "0x0",
        "extraData": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "gasLimit": "0x80000000",
        "difficulty": "0x1",
        "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "coinbase": "0x0000000000000000000000000000000000000000",
        "alloc": {},
        "number": "0x0",
        "gasUsed": "0x0",
        "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "baseFeePerGas": "0x0"
    }
    
    # Process accounts and add them to alloc
    print("🔧 Processing accounts...")
    for address, account_data in accounts.items():
        if isinstance(account_data, dict) and "info" in account_data:
            info = account_data["info"]
            
            # Convert account info to genesis format
            genesis_account = {
                "balance": hex(info.get("balance", 0)),
                "nonce": hex(info.get("nonce", 0))
            }
            
            # Add code if present
            if "code" in info and info["code"] is not None:
                genesis_account["code"] = info["code"]
            
            # Add storage if present
            if "storage" in account_data and account_data["storage"]:
                genesis_account["storage"] = {}
                for slot, value in account_data["storage"].items():
                    genesis_account["storage"][hex(slot)] = hex(value)
            
            genesis["alloc"][address] = genesis_account
    
    # Add any additional state from bundle_state if needed
    if "state" in bundle_state:
        print("🔧 Processing bundle state...")
        for address, account_data in bundle_state["state"].items():
            if address not in genesis["alloc"]:
                # This is a new account created during initialization
                if isinstance(account_data, dict) and "info" in account_data:
                    info = account_data["info"]
                    genesis_account = {
                        "balance": hex(info.get("balance", 0)),
                        "nonce": hex(info.get("nonce", 0))
                    }
                    
                    if "code" in info and info["code"] is not None:
                        genesis_account["code"] = info["code"]
                    
                    if "storage" in account_data and account_data["storage"]:
                        genesis_account["storage"] = {}
                        for slot, value in account_data["storage"].items():
                            genesis_account["storage"][hex(slot)] = hex(value)
                    
                    genesis["alloc"][address] = genesis_account
    
    # Write the final genesis.json
    print(f"💾 Writing genesis.json to {output_file}")
    try:
        with open(output_file, 'w') as f:
            json.dump(genesis, f, indent=2)
        print(f"✅ Successfully generated {output_file}")
    except Exception as e:
        print(f"❌ Error writing {output_file}: {e}")
        sys.exit(1)
    
    # Print summary
    print("\n📊 Genesis Summary:")
    print(f"  - Total accounts: {len(genesis['alloc'])}")
    print(f"  - Chain ID: {genesis['config']['chainId']}")
    print(f"  - Gas limit: {genesis['gasLimit']}")
    print(f"  - Timestamp: {genesis['timestamp']}")
    
    # Count accounts with code (contracts)
    contract_count = sum(1 for account in genesis['alloc'].values() 
                        if 'code' in account and account['code'])
    print(f"  - Contracts: {contract_count}")
    
    print(f"\n🎉 Genesis generation completed successfully!")


def main():
    parser = argparse.ArgumentParser(description="Generate final genesis.json file")
    parser.add_argument("--accounts", default="output/genesis_accounts.json",
                       help="Path to genesis_accounts.json")
    parser.add_argument("--contracts", default="output/genesis_contracts.json", 
                       help="Path to genesis_contracts.json")
    parser.add_argument("--bundle-state", default="output/bundle_state.json",
                       help="Path to bundle_state.json")
    parser.add_argument("--output", default="genesis.json",
                       help="Output genesis.json file path")
    
    args = parser.parse_args()
    
    # Check if input files exist
    for file_path in [args.accounts, args.contracts, args.bundle_state]:
        if not os.path.exists(file_path):
            print(f"❌ Error: Input file {file_path} does not exist")
            sys.exit(1)
    
    create_genesis_json(
        accounts_file=args.accounts,
        contracts_file=args.contracts,
        bundle_state_file=args.bundle_state,
        output_file=args.output
    )


if __name__ == "__main__":
    main() 