#!/usr/bin/env -S just --justfile

# Set shell configurations
set windows-shell := ["powershell"]
set shell := ["bash", "-cu"]

setup-bininstall:
    curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

setup:
    cargo binstall taplo-cli cargo-release -y
    @echo '✅ Setup complete!'

ready:
  just fmt    
  just lint 
  @echo '✅ All passed!'

fmt:
    cargo fmt --all -- --emit=files
    taplo fmt **/Cargo.toml
    @echo '✅ Format complete!'

lint: 
    cargo clippy    
    @echo '✅ Lint complete!'

test:
    cargo test
    @echo '✅ Test complete!'

review:
    cargo insta test --review
    @echo '✅ Review complete!'

doc:
    cargo doc --no-deps --open
    @echo '✅ Doc complete!'

release-patch:
    cargo release patch --no-push --no-publish --execute
    @echo '✅ Release patch complete!'

release-minor:
    cargo release minor --no-push --no-publish --execute
    @echo '✅ Release minor complete!'

release-major:
    cargo release major --no-push --no-publish --execute
    @echo '✅ Release major complete!'


