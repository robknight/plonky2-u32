## Compiled witness generation

With a matching Plonky2 fork that supports compiled witness programs, the u32
arithmetic, add-many, subtraction, range-check and comparison generators provide
typed operation descriptions. Call `witness_program_support` to check the whole
circuit, then prepare or install a witness program through the circuit-data API.
The u32 operations execute on the CPU and can feed GPU proving. Circuit
constraints, serialization and verifier behavior are unchanged.

`tests/witness_executor.rs` compares every witness value against the worklist,
checks boundary and fresh inputs, and verifies proofs after program loading.
When testing local executor changes, override the pinned Plonky2 Git dependency
with a Cargo patch to the matching local checkout.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
