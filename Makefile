RUST_STABLE := 1.99.1
MSRV := 1.95.0
CARGO_BUILD_JOBS ?= 2
export CARGO_BUILD_JOBS
MANIFESTS := office-sdk/Cargo.toml api-test/Cargo.toml

.PHONY: ci fmt-check check test docs audit msrv-check
ci: fmt-check check test docs audit msrv-check

fmt-check:
	@set -eu; for manifest in $(MANIFESTS); do cargo +$(RUST_STABLE) fmt --manifest-path $$manifest --all -- --check; done

check:
	@set -eu; for manifest in $(MANIFESTS); do cargo +$(RUST_STABLE) check --manifest-path $$manifest --all-targets --all-features --locked; done

test:
	@set -eu; for manifest in $(MANIFESTS); do cargo +$(RUST_STABLE) test --manifest-path $$manifest --all-targets --all-features --locked; done
	# api-test is binary-only; Cargo's --doc requires a library target.
	cargo +$(RUST_STABLE) test --manifest-path office-sdk/Cargo.toml --doc --all-features --locked

docs:
	cargo +$(RUST_STABLE) doc --manifest-path office-sdk/Cargo.toml --no-deps --all-features --locked

audit:
	@set -eu; for manifest in $(MANIFESTS); do cargo audit --file $$(dirname $$manifest)/Cargo.lock; done

msrv-check:
	@set -eu; for manifest in $(MANIFESTS); do CARGO_TARGET_DIR=$$(dirname $$manifest)/target/msrv cargo +$(MSRV) check --manifest-path $$manifest --all-targets --all-features --locked; done

# Optional narrower checks for dependency work. CI always runs the full gate
# above, including every example and integration test.
.PHONY: ci-focused check-focused test-focused msrv-focused
ci-focused: fmt-check check-focused test-focused docs audit msrv-focused

check-focused:
	cargo +$(RUST_STABLE) check --manifest-path office-sdk/Cargo.toml --lib --bins --all-features --locked
	cargo +$(RUST_STABLE) check --manifest-path api-test/Cargo.toml --all-targets --all-features --locked

test-focused:
	cargo +$(RUST_STABLE) test --manifest-path office-sdk/Cargo.toml --lib --bins --all-features --locked
	cargo +$(RUST_STABLE) test --manifest-path office-sdk/Cargo.toml --test dependency_regressions --all-features --locked
	cargo +$(RUST_STABLE) test --manifest-path office-sdk/Cargo.toml --doc --all-features --locked
	cargo +$(RUST_STABLE) test --manifest-path api-test/Cargo.toml --all-targets --all-features --locked

msrv-focused:
	CARGO_TARGET_DIR=office-sdk/target/msrv cargo +$(MSRV) check --manifest-path office-sdk/Cargo.toml --lib --bins --test dependency_regressions --all-features --locked
	CARGO_TARGET_DIR=api-test/target/msrv cargo +$(MSRV) check --manifest-path api-test/Cargo.toml --all-targets --all-features --locked
