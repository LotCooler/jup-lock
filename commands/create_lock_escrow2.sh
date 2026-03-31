rpc_url=https://api.mainnet.solana.com
keypair_path=../keys/localnet/9tRh1bM5poHw4A6fmxxPhHDFjgzqhC9S6bixq1Smz9vQ.json
wallet_path=./trash/test2.csv
token_mint=FuAvQnkKSVkUWdMMzrYYXLoS46CWtL8UjVHjmLVzbonk  #this is ignored, code uses mint in .csv file
vesting_start_time=1774986459
cliff_time=1774986459
frequency=1
number_of_period=1
update_recipient_mode=0
cancel_mode=0

../target/debug/cli --rpc-url $rpc_url --keypair-path $keypair_path initialize-lock-escrow-from-file2 --wallet-path $wallet_path --token-mint $token_mint --vesting-start-time $vesting_start_time --cliff-time $cliff_time --frequency $frequency --number-of-period $number_of_period --update-recipient-mode $update_recipient_mode --cancel-mode $cancel_mode
