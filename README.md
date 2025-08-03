# No-allocation SLIP

## Instructions

This Rust crate provides a no-std SLIP (Serial Line Internet Protocol) encoder and decoder implementation. SLIP is a simple protocol for sending packets over serial lines.

### Usage

This library has to be used in conjunction with the [`noalloc-vec-rs`](https://github.com/jaudiger/noalloc-vec-rs) crate.

- Example of encoding a packet:

```rust
use noalloc_slip_rs::slip::{END_CHAR, SlipEncoder, SlipDecoder};
use noalloc_vec_rs::vec::Vec;

const MAX_LENGTH: usize = 12;

let mut packet = Vec::<u8, MAX_LENGTH>::from([0x00, 0x01, 0x02, 0x03]);
SlipEncoder::encode(&mut packet).unwrap();

assert_eq!(*packet, [END_CHAR, 0x00, 0x01, 0x02, 0x03, END_CHAR]);
```

- Example of decoding a packet:

```rust
use noalloc_slip_rs::slip::{END_CHAR, SlipEncoder, SlipDecoder};
use noalloc_vec_rs::vec::Vec;

const MAX_LENGTH: usize = 12;

let mut decoder = SlipDecoder::<MAX_LENGTH>::default();
decoder.insert(END_CHAR).unwrap();
decoder.insert(0x00).unwrap();
decoder.insert(END_CHAR).unwrap();

assert!(decoder.is_buffer_completed());
assert_eq!(decoder.get_buffer(), &[0x00]);
```

## CI / CD

The CI/CD pipeline is configured using GitHub Actions. The workflow is defined in the [`.github/workflows`](.github/workflows) folder:

- Static Analysis (source code, GitHub Actions)
- Tests (unit tests with code coverage generated)
- Code Audit (on each Cargo dependencies update, or run each day through CronJob)
- Deployment

Additionally, Dependabot is configured to automatically update dependencies (GitHub Actions, Cargo dependencies).

## Repository configuration

The settings of this repository are managed from the [gitops-deployments](https://github.com/jaudiger/gitops-deployments) repository using Terraform. The actual configuration applied is located in the Terraform module [`modules/github-repository`](https://github.com/jaudiger/gitops-deployments/tree/main/modules/github-repository).
