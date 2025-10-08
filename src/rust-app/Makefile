# Default target
.DEFAULT_GOAL := help

# Variables
CARGO := cargo
BINARY := rust-app

## help: Show this help message
help:
	@echo "Usage: make [target]"
	@echo
	@echo "Targets:"
	@grep -E '^##' Makefile | sed -E 's/^## (.*): /  \1\t/'

## run: Build and start the API server with cargo run
run:
	$(CARGO) run

## check: Run cargo check to verify the code compiles
check:
	$(CARGO) check

## test: Execute all unit and integration tests
test:
	$(CARGO) test

## fmt: Format the codebase using rustfmt
fmt:
	$(CARGO) fmt

## clippy: Lint the project and treat warnings as errors
clippy:
	$(CARGO) clippy -- -D warnings

## clean: Remove build artifacts
clean:
	$(CARGO) clean

.PHONY: help run check test fmt clippy clean
