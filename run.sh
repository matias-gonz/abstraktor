cd SUT/dqlite/dqlite-raft && sudo make clean && echo "" > mipass.log && cd ../../../llvm && make && cd .. && cargo run instrument --path ./SUT/dqlite/dqlite-raft/src
