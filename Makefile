# Docker: yangby0cryptape/ibc-ckb-contracts-build-env:0.2.0-alpha.0
BUILD_ENV_DOCKER := yangby0cryptape/ibc-ckb-contracts-build-env@sha256:71ef760b76260d3f9be6bf8442e6840f186328e30f1e73e8044bdf2bf467e1aa
RUST_TOOLCHAIN_TARGET := riscv64imac-unknown-none-elf
PROJECT_PREFIX := ibc-ckb_contracts

# Enable Debugging with `export CARGO_BUILD_ARGS="--features debugging"`.
CARGO_BUILD_ARGS ?=

CONTRACTS_DIR := contracts
OUTPUT_DIR := build

ALL_CONTRACTS := \
    mock_contracts-can_update_without_ownership_lock \
    eth_light_client-client_type_lock \
    eth_light_client-verify_bin \
    eth_light_client-mock_business_type_lock \
	ics-connection \
	ics-channel \
	ics-packet

.PHONY: all-contracts all-contracts-in-docker
all-contracts: ${ALL_CONTRACTS}
all-contracts-in-docker: $(addsuffix -in-docker,${ALL_CONTRACTS})

.PHONY: clean-all-contracts
clean-all-contracts:
	-rm -f $(addprefix ${OUTPUT_DIR}/,${ALL_CONTRACTS})

.PHONY: clean-all-contracts-targets clean-all-contracts-targets-in-docker
clean-all-contracts-targets:
	@set -eu; \
		rootdir="$$(pwd)"; \
		for contract in ${ALL_CONTRACTS}; do \
			contract_dir=$$(echo "$${contract}" | tr -s '-' '/'); \
			cd "${CONTRACTS_DIR}/$${contract_dir}"; cargo clean; cd "$${rootdir}"; \
		done
clean-all-contracts-targets-in-docker:
	@docker run --rm -v "$$(pwd):/code" -w "/code" "${BUILD_ENV_DOCKER}" bash -c "make $(subst -in-docker,,$@)"

.PHONY: format-all-contracts format-all-contracts-in-docker
format-all-contracts:
	@set -eu; \
		rootdir="$$(pwd)"; \
		for contract in ${ALL_CONTRACTS}; do \
			echo ">>> Format-check \"$${contract}\" contract ..."; \
			contract_dir=$$(echo "$${contract}" | tr -s '-' '/'); \
			cd "${CONTRACTS_DIR}/$${contract_dir}"; cargo fmt --all -- --check; cd "$${rootdir}"; \
		done; \
		echo "[DONE] Format-check all contracts."
format-all-contracts-in-docker:
	@docker run --rm -v "$$(pwd):/code" -w "/code" "${BUILD_ENV_DOCKER}" bash -c "make $(subst -in-docker,,$@)"

.PHONY: lint-all-contracts lint-all-contracts-in-docker
lint-all-contracts:
	@set -eu; \
		rootdir="$$(pwd)"; \
		for contract in ${ALL_CONTRACTS}; do \
			echo ">>> Lint \"$${contract}\" contract ..."; \
			contract_dir=$$(echo "$${contract}" | tr -s '-' '/'); \
			cd "${CONTRACTS_DIR}/$${contract_dir}"; cargo clippy --locked -- --deny warnings; cd "$${rootdir}"; \
		done; \
		echo "[DONE] Lint all contracts."
lint-all-contracts-in-docker:
	@docker run --rm -v "$$(pwd):/code" -w "/code" "${BUILD_ENV_DOCKER}" bash -c "make $(subst -in-docker,,$@)"

${OUTPUT_DIR}/%:
	@set -eu; \
		rootdir="$$(pwd)"; \
		contract_name="$(notdir $@)"; \
		contract_filename="${PROJECT_PREFIX}-$${contract_name}"; \
		contract_dir=$$(echo "$${contract_name}" | tr -s '-' '/'); \
		cd "${CONTRACTS_DIR}/$${contract_dir}"; \
			cargo build --release ${CARGO_BUILD_ARGS}; \
			cp "target/${RUST_TOOLCHAIN_TARGET}/release/$${contract_filename}" "$${rootdir}/${OUTPUT_DIR}/$${contract_name}"; \
			cd ..;

%-in-docker:
	@set -eu; \
		contract_name="$(subst -in-docker,,$@)"; \
		owner_and_group="$$(id -u):$$(id -g)"; \
		docker run --rm -v "$$(pwd):/code" -w "/code" "${BUILD_ENV_DOCKER}" \
			bash -c "make \"$${contract_name}\" && chown -v \"$${owner_and_group}\" \"${OUTPUT_DIR}/$${contract_name}\""

#
# Targets for Test Utilities
#

format-test-utils:
	@set -eu; \
		cd test-utils; \
		cargo fmt --all -- --check

lint-test-utils:
	@set -eu; \
		cd test-utils; \
		cargo clippy -- --deny warnings

test:
	@set -eu; \
		cd test-utils; \
		cargo test --no-fail-fast -- --nocapture --test-threads 1

#
# Targets to Build Contracts
#

.PHONY: mock_contracts-can_update_without_ownership_lock
mock_contracts-can_update_without_ownership_lock: ${OUTPUT_DIR}/mock_contracts-can_update_without_ownership_lock

.PHONY: eth_light_client-client_type_lock
eth_light_client-client_type_lock: ${OUTPUT_DIR}/eth_light_client-client_type_lock

.PHONY: eth_light_client-verify_bin
eth_light_client-verify_bin: ${OUTPUT_DIR}/eth_light_client-verify_bin

.PHONY: eth_light_client-mock_business_type_lock
eth_light_client-mock_business_type_lock: ${OUTPUT_DIR}/eth_light_client-mock_business_type_lock

.PHONY: ics-connection
ics-connection: ${OUTPUT_DIR}/ics-connection

.PHONY: ics-channel
ics-channel: ${OUTPUT_DIR}/ics-channel

.PHONY: ics-packet
ics-packet: ${OUTPUT_DIR}/ics-packet