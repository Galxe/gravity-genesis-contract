import json
import sys


def transform_accounts(
    genesis_accounts_path, genesis_contracts_path, output_path="account_alloc.json"
):
    """Combine genesis accounts and contracts into account allocation format."""
    
    # Load genesis accounts (address -> balance mapping)
    #   "0x000000000000000000000000000000000000200a": {
    #     "info": {
    #       "balance": "0x0",
    #       "nonce": 0,
    #       "code_hash": "0x5a5bdecab8d1c74359b677b4df5e30370f541dd4f4eae053ac690e96281e510b"
    #     },
    #     "storage": {
    #       "0x7": "0x1010",
    #       "0xf0c57e16840df040f15088dc2f81fe391c3923bec73e23a9662efc9c229c6a00": "0x1",
    #       "0x5": "0x101000001000080000800080",
    #       "0x3": "0x151800010"
    #     }
    #   }
    with open(genesis_accounts_path, "r") as f:
        accounts_data = json.load(f)

    # Load genesis contracts (address -> bytecode mapping)
    with open(genesis_contracts_path, "r") as f:
        contracts_data = json.load(f)

    # Create account allocation format
    account_alloc = {}
    
    # Process all accounts
    for addr, account_info in accounts_data.items():
        account_info.pop("code_hash")
        account_info["code"] = contracts_data.get(addr)
        account_alloc[addr] = account_info

    for addr, _ in contracts_data.items():
        if addr not in account_alloc:
            raise Exception(f"Contract {addr} not found in accounts")

    with open(output_path, "w") as f:
        json.dump(account_alloc, f, indent=2)

    print(f"✅ Successfully combined {len(accounts_data)} accounts and {len(contracts_data)} contracts")
    print(f"✅ Total accounts in allocation: {len(account_alloc)}")
    print(f"✅ Successfully wrote to {output_path}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python script.py <genesis_accounts.json> <genesis_contracts.json>")
        sys.exit(1)

    file_a = sys.argv[1]
    file_b = sys.argv[2]

    transform_accounts(file_a, file_b)
