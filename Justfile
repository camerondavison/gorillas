info:
  just --list
lint: fmt
  cargo clippy --fix --allow-dirty
fmt:
  cargo fmt
setup-dev:
  brew install shaderc cmake
