import argparse
import json
import os
from pathlib import Path
from typing import Dict, Any

# py-solc-x is the preferred library for interacting with the Solidity compiler
import solcx
from solcx.exceptions import SolcError

def compile_and_save(contracts_dir: Path, output_dir: Path, solc_version: str):
    """
    Compiles all Solidity contracts in a given directory and saves the bytecode
    to an output directory.

    :param contracts_dir: Path to the directory containing .sol files.
    :param output_dir: Path to the directory where compiled bytecode will be saved.
    :param solc_version: The solc compiler version to use, e.g., "0.8.20".
    """
    print(f"[*] Using contracts directory: {contracts_dir}")
    print(f"[*] Bytecode output directory: {output_dir}")

    # --- 1. Ensure solc version is installed (a core advantage of py-solc-x) ---
    try:
        # 1. Define a local path for all solcx files (binaries, locks, etc.)
        #    This should be a relative or absolute path where you have write access.
        solc_storage_path = Path("./solc-bin")
        solc_storage_path.mkdir(parents=True, exist_ok=True)

        # 2. Set the environment variable BEFORE importing solcx.
        #    This is the most critical step. It tells py-solc-x where to store everything.
        os.environ['SOLCX_BINARY_PATH'] = str(solc_storage_path.resolve())
        print(f"[*] Checking/installing solc version: {solc_version}, path: {solcx.get_solcx_install_folder()}...")
        solcx.install_solc(solc_version, show_progress=True)
        print(f"[*] solc version '{solcx.get_solc_version()}' has been installed successfully.")
        solcx.set_solc_version(solc_version, silent=True)
        print(f"[+] solc version '{solcx.get_solc_version()}' has been set successfully.")
    except Exception as e:
        print(f"[!] Error: Failed to install or set solc version {solc_version}. Please check if the version number is correct.")
        print(f"    {e}")
        return

    # --- 2. Prepare the Standard JSON input for solc ---
    # This is the most reliable way to interact with solc.
    input_json = {
        "language": "Solidity",
        "sources": {},
        "settings": {
            "outputSelection": {
                "*": {  # For all files
                    "*": [  # For all contracts
                        "evm.bytecode.object"  # The output we want
                    ]
                }
            }
        }
    }

    # Walk the directory, find all .sol files, and add them to the input
    sol_files = list(contracts_dir.glob('**/*.sol'))
    if not sol_files:
        print("[!] No .sol files found in the directory.")
        return

    print(f"\n[*] Found {len(sol_files)} .sol file(s), preparing to compile...")
    for contract_path in sol_files:
        # Using relative paths as keys for solc is important.
        source_key = str(contract_path.relative_to(contracts_dir.parent))
        input_json["sources"][source_key] = {
            "content": contract_path.read_text()
        }

    # --- 3. Invoke the compiler ---
    try:
        # Use the standard_json method to compile.
        compiled_sol = solcx.compile_standard(input_json, allow_paths=".")
    except SolcError as e:
        print("\n[!!!] Solidity compilation failed!")
        print(e)
        return

    print("\n[+] Compilation successful! Extracting and saving bytecode...")

    # --- 4. Process the output and save the bytecode ---
    output_dir.mkdir(parents=True, exist_ok=True)

    # The structure of `compiled_sol['contracts']` is { "filepath": { "ContractName": { ... } } }
    for file_path, contracts in compiled_sol.get("contracts", {}).items():
        for contract_name, contract_data in contracts.items():
            bytecode = contract_data.get("evm", {}).get("bytecode", {}).get("object")
            
            if not bytecode:
                print(f"   [!] Warning: Bytecode for contract '{contract_name}' not found in '{file_path}' (it might be an interface or library).")
                continue

            # Best practice: Use FileName_ContractName.hex format to avoid naming conflicts.
            # e.g., contract "Greeter" in "Greeter.sol" -> "Greeter_Greeter.hex"
            output_filename = f"{Path(file_path).stem}_{contract_name}.hex"
            output_path = output_dir / output_filename
            
            output_path.write_text(bytecode)
            print(f"   -> Saved: {output_path}")

def main():
    """Main entry point for the script, handles command-line arguments."""
    parser = argparse.ArgumentParser(description="Compile Solidity contracts and extract their bytecode.")
    parser.add_argument(
        "--contracts-dir",
        type=Path,
        default=Path("./contracts"),
        help="Source directory containing Solidity (.sol) files."
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("./output_bytecode"),
        help="Output directory for the compiled bytecode."
    )
    parser.add_argument(
        "--solc-version",
        type=str,
        default="0.8.20",
        help="The Solidity compiler version to use."
    )
    args = parser.parse_args()

    if not args.contracts_dir.is_dir():
        print(f"[!] Error: Contracts directory '{args.contracts_dir}' does not exist or is not a directory.")
        return

    compile_and_save(args.contracts_dir, args.output_dir, args.solc_version)

# Python best practice: Use a main guard.
if __name__ == "__main__":
    main()