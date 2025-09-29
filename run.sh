cd SUT/dqlite/dqlite-raft && make clean && echo "" > mipass.log && cd ../../../llvm && make && cd .. && cargo run instrument --path ./SUT/dqlite/dqlite-raft
