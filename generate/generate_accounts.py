#!/usr/bin/env python3
"""
以太坊账户生成器
根据以太坊助记词要求生成账户，输出公钥、私钥和地址
"""

import argparse
import json
from eth_account import Account
from eth_keys import keys
import os

def generate_accounts(num_accounts=4):
    """
    生成指定数量的以太坊账户
    
    Args:
        num_accounts (int): 要生成的账户数量，默认4个
    
    Returns:
        list: 包含账户信息的列表
    """
    accounts = []
    
    for i in range(num_accounts):
        # 生成新的账户
        account = Account.create()
        
        # 获取私钥（十六进制格式，去掉0x前缀）
        private_key = account.key.hex()[2:]
        
        # 从私钥生成公钥
        private_key_bytes = account.key
        public_key_bytes = keys.PrivateKey(private_key_bytes).public_key
        public_key = public_key_bytes.to_hex()[2:]  # 去掉0x前缀
        
        # 获取地址
        address = account.address
        
        account_info = {
            "account_index": i + 1,
            "address": address,
            "public_key": public_key,
            "private_key": private_key,
            "mnemonic": account.mnemonic if hasattr(account, 'mnemonic') else None
        }
        
        accounts.append(account_info)
    
    return accounts

def save_accounts_to_file(accounts, filename="account_info.json"):
    """
    将账户信息保存到文件
    
    Args:
        accounts (list): 账户信息列表
        filename (str): 输出文件名
    """
    output_data = {
        "total_accounts": len(accounts),
        "accounts": accounts
    }
    
    with open(filename, 'w', encoding='utf-8') as f:
        json.dump(output_data, f, indent=2, ensure_ascii=False)
    
    print(f"账户信息已保存到: {filename}")

def print_accounts_summary(accounts):
    """
    打印账户信息摘要
    
    Args:
        accounts (list): 账户信息列表
    """
    print(f"\n=== 生成了 {len(accounts)} 个以太坊账户 ===")
    print("=" * 60)
    
    for i, account in enumerate(accounts, 1):
        print(f"\n账户 {i}:")
        print(f"  地址: {account['address']}")
        print(f"  公钥: {account['public_key'][:20]}...{account['public_key'][-20:]}")
        print(f"  私钥: {account['private_key'][:20]}...{account['private_key'][-20:]}")
        print("-" * 40)

def main():
    parser = argparse.ArgumentParser(description='生成以太坊账户信息')
    parser.add_argument(
        '-n', '--num_accounts', 
        type=int, 
        default=4, 
        help='要生成的账户数量 (默认: 4)'
    )
    parser.add_argument(
        '-o', '--output', 
        type=str, 
        default='account_info.json', 
        help='输出文件名 (默认: account_info.json)'
    )
    parser.add_argument(
        '--no-save', 
        action='store_true', 
        help='不保存到文件，只在控制台显示'
    )
    
    args = parser.parse_args()
    
    # 验证参数
    if args.num_accounts <= 0:
        print("错误: 账户数量必须大于0")
        return
    
    if args.num_accounts > 100:
        print("警告: 生成大量账户可能需要一些时间...")
    
    print(f"正在生成 {args.num_accounts} 个以太坊账户...")
    
    try:
        # 生成账户
        accounts = generate_accounts(args.num_accounts)
        
        # 打印摘要
        print_accounts_summary(accounts)
        
        # 保存到文件（除非指定不保存）
        if not args.no_save:
            save_accounts_to_file(accounts, args.output)
        
        print(f"\n✅ 成功生成 {len(accounts)} 个账户!")
        
    except Exception as e:
        print(f"❌ 生成账户时发生错误: {e}")
        return

if __name__ == "__main__":
    main()
