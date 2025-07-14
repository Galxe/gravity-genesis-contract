import json
import sys


def transform_accounts(
    genesis_accounts_path, genesis_contracts_path, output_path="account_alloc.json"
):
    with open(genesis_accounts_path, "r") as f:
        accounts_a = json.load(f)

    with open(genesis_contracts_path, "r") as f:
        accounts_b = json.load(f)

    for addr, account_obj in accounts_a.items():
        info = account_obj.get("info", {})
        _ = info.pop("code_hash", None)
        code = accounts_b.get(addr)
        if code is not None:
            info["code"] = code
        else:
            info["code"] = None
        account_obj["info"] = info

    with open(output_path, "w") as f:
        json.dump(accounts_a, f, indent=2)

    print(f"✅ Successfully wrote to {output_path}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python script.py <file_a.json> <file_b.json>")
        sys.exit(1)

    file_a = sys.argv[1]
    file_b = sys.argv[2]

    transform_accounts(file_a, file_b)
