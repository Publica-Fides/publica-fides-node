# MOT Node  🏢 #

## Background ##
This houses the MOT pallet. For more background on the project, see `pallets/publicafides/README.md`

## Development ##
To build the project, run:
```shell
cargo build --release
```
To run the project
```shell
./target/release/node-template --dev --tmp
```

#### Linting ####
```shell
cargo fmt
```

#### Testing ####
```shell
cargo test
```

#### TODO ####
- Finish benchmarking and finalize weights.
- TODOs for publicafides pallet