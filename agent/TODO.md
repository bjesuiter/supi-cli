# TODOs for supi-cli

## Testing
- [ ] **Evaluate nextest for this repo**
  - Check out [cargo-nextest](https://nexte.st/)
  - Question: What benefits does nextest bring over normal `cargo test` for this repo?
  - Consider: parallel execution, better output, test filtering, retries, JUnit output
  - This repo has 34 integration tests - might benefit from nextest's parallel execution
