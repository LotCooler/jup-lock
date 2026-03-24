rpc_url=https://api.devnet.solana.com
keypair_path=../keys/localnet/admin-bossj3JvwiNK7pvjr149DqdtJxf2gdygbcmEPTkb2F1.json
wallet_path=./trash/test2.csv
token_mint=FuAvQnkKSVkUWdMMzrYYXLoS46CWtL8UjVHjmLVzbonk
vesting_start_time=1774382366
cliff_time=1774382366
frequency=604800
number_of_period=4
update_recipient_mode=0

../target/debug/cli --rpc-url $rpc_url --keypair-path $keypair_path initialize-lock-escrow-from-file2 --wallet-path $wallet_path --token-mint $token_mint --vesting-start-time $vesting_start_time --cliff-time $cliff_time --frequency $frequency --number-of-period $number_of_period --update-recipient-mode $update_recipient_mode --cancel-mode 0

read -p "Press Enter to continue..."