# Contributing to Thinking Computer

Thank you for improving Thinking Computer. The project accepts focused issues, documentation improvements, test cases, provider adapters, hardening work, and plugins. Before making a substantial change, open an issue that explains the user outcome, the security impact, and how the change will be tested.

## Development process

Fork the repository, create a branch with one focused purpose, and keep the Rust core as the policy-enforcement boundary. New tools must declare a narrow JSON schema, operate within the selected workspace unless explicitly designed otherwise, and remain denied or confirmation-gated by default. Do not add an implicit shell, filesystem, network, or plugin permission.

| Change type | Required evidence |
| --- | --- |
| Rust logic | Add or update unit tests and run `cargo test --workspace`. |
| C++ bridge | Build the workspace on a supported platform and retain a narrow safe interface. |
| Provider adapter | Add fixture-based response parsing tests; never commit live credentials. |
| Plugin host | Check the Node.js module syntax and test both a successful and denied invocation. |
| Documentation | Confirm command names and paths against the built CLI. |

Format Rust code with `cargo fmt --all` and run the commands listed in the README before opening a pull request. Explain any user-visible behavior change in the pull-request description.

## License and attribution

Contributions are submitted under the repository's MIT License. Do not paste source code or assets from another project without identifying its license, preserving required notices, and explaining the provenance in the pull request.

