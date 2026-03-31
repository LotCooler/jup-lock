rpc_url=https://api.mainnet.solana.com
keypair_path=../keys/localnet/9tRh1bM5poHw4A6fmxxPhHDFjgzqhC9S6bixq1Smz9vQ.json
wallet_path=./trash/testClaim2.csv

../target/debug/cli --rpc-url $rpc_url --keypair-path $keypair_path initialize-claim-escrow-from-file2 --wallet-path $wallet_path
