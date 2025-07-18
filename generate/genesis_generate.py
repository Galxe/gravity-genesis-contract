#!/usr/bin/env python3
"""
Genesis generation script for Gravity blockchain.
This script combines genesis template with account allocation data.
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
    template_file: str = "generate/genesis_template.json",
    account_alloc_file: str = "account_alloc.json",
    output_file: str = "genesis.json"
) -> None:
    """
    Create the final genesis.json file by combining template with account allocation.
    
    Args:
        template_file: Path to genesis_template.json
        account_alloc_file: Path to account_alloc.json (output from combine_account_alloc.py)
        output_file: Output genesis.json file path
    """
    print("🔄 Starting genesis.json generation...")
    
    # Load genesis template
    print(f"📖 Loading genesis template from {template_file}")
    genesis = load_json_file(template_file)
    
    # Load account allocation data
    print(f"📖 Loading account allocation from {account_alloc_file}")
    account_alloc = load_json_file(account_alloc_file)
    
    # Merge account allocation into genesis alloc field
    print("🔧 Merging account allocation into genesis...")
    # Create a new alloc dict starting with account_alloc data
    merged_alloc = {}
    
    # Add account_alloc data first (at the beginning)
    merged_alloc.update(account_alloc)
    
    # Then add existing genesis template alloc data
    if "alloc" in genesis:
        merged_alloc.update(genesis["alloc"])
    
    # Replace the alloc field with merged data
    genesis["alloc"] = merged_alloc
    
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
    parser.add_argument("--template", default="generate/genesis_template.json",
                       help="Path to genesis_template.json")
    parser.add_argument("--account-alloc", default="account_alloc.json",
                       help="Path to account_alloc.json (output from combine_account_alloc.py)")
    parser.add_argument("--output", default="genesis.json",
                       help="Output genesis.json file path")
    
    args = parser.parse_args()
    
    # Check if input files exist
    for file_path in [args.template, args.account_alloc]:
        if not os.path.exists(file_path):
            print(f"❌ Error: Input file {file_path} does not exist")
            sys.exit(1)
    
    create_genesis_json(
        template_file=args.template,
        account_alloc_file=args.account_alloc,
        output_file=args.output
    )


if __name__ == "__main__":
    main() 