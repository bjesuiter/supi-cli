# Bonnie/bx Reference from dev_uploader project

This is a reference bonnie.toml from another rust project for inspiration.

```toml
# Run Scripts with `bx <script-name>`

version = "0.3.2"

[scripts]
    build-debug.cmd     = "cargo build --bin dev_uploader"
    build-release.cmd   = "cargo build --bin dev_uploader --release"
    build-profiling.cmd = "cargo build --bin dev_uploader --profile profiling"
    prod.cmd            = "cargo build --bin dev_uploader --release"
    prod.desc           = "Shortcut for building the release binary"
    # prod.env            = { RUSTFLAGS = "-C link-arg=-Wl,-static -C link-arg=-dead_strip" }

    otool-prod.cmd  = "otool -L target/release/dev_uploader"
    otool-prod.desc = "Show linked libraries of the release binary"

    pg.cmd                 = "cargo run --bin dev_uploader -- -H localhost -P 2022 -u playground:. -i .gitkeep -e .gitignore -U playground -W playground"
    pg-initial.cmd         = "cargo run --bin dev_uploader -- -H localhost -P 2022 -u playground:. -i .gitkeep -e .gitignore -U playground -W playground --upload-initial"
    pg-watch.cmd           = "cargo watch -x 'run --bin dev_uploader -- -H localhost -P 2022 -u playground:. -i .gitkeep -e .gitignore -U playground -W playground'"
    pg-preview-debug.cmd   = "bx build-debug && ./target/debug/sftp_dev_uploader_rust --connections 9 --host localhost -P 2022 -u playground:. -i .gitkeep -e .gitignore -U playground -W playground"
    pg-profile-debug.cmd   = "bx build-debug && samply record ./target/debug/sftp_dev_uploader_rust --connections 9 --host localhost -P 2022 -u playground:. -i .gitkeep -e .gitignore -U playground -W playground"
    pg-preview-release.cmd = "bx build-release && ./target/release/sftp_dev_uploader_rust --connections 9 --host localhost -P 2022 -u playground:. -i .gitkeep -e .gitignore -U playground -W playground"
    pg-profile-release.cmd = "bx build-profiling && samply record ./target/profiling/sftp_dev_uploader_rust --connections 9 --host localhost -P 2022 -u playground:. -i .gitkeep -e .gitignore -U playground -W playground"

    # Extra cli commands (for testing)
    helpArg.cmd      = "cargo run --bin dev_uploader -- --help"
    versionArg.cmd   = "cargo run --bin dev_uploader -- --version"
    V.cmd            = "cargo run --bin dev_uploader -- -V"
    progressbars.cmd = "cargo run --bin explore_progressbars"

    # Extra profiling commands 
    load-profile.cmd  = "samply load %%"
    load-profile.desc = "Load a profile file, add the file path as argument, like profiles/2024-11-14_12_40_profile.json"

    # call with specific test: `bx test -- test_sftp_compute_relative_filepath` 
    # run one full test file: `bx test -- ssh2_client_tests`
    test.cmd       = "cargo insta test --jobs=1 -- --nocapture %%"
    test.desc      = "Run all tests (without argument) or one test once (with the test function as arg)"
    review.cmd     = "cargo insta review"
    review.desc    = "Review all snapshots"
    test-u.cmd     = "cargo insta review"
    test-u.desc    = "Run tests and update snapshots interactively"
    test-watch.cmd = "cargo watch -x 'insta test -- --nocapture'"

    # Specialized Test commands
    test-sftp-client.cmd = "bonnie test -- sftp_client_tests"
```
