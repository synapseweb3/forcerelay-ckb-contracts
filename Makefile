# Docker: yangby0cryptape/ibc-ckb-contracts-build-env:0.1.0-alpha.0
BUILD_ENV_DOCKER := yangby0cryptape/ibc-ckb-contracts-build-env@sha256:7047fe6e56a6dbf2e7406ef2d217366c2c2f6b2a3d46e96a5d9988a0dfcfb1b6
RUST_TOOLCHAIN_TARGET := riscv64imac-unknown-none-elf
PROJECT_PREFIX := ibc-ckb_contracts

CONTRACTS_DIR := contracts
OUTPUT_DIR := build

ALL_CONTRACTS := eth-light-client

.PHONY: all-contracts all-contracts-in-docker
all-contracts: ${ALL_CONTRACTS}
all-contracts-in-docker: $(addsuffix -in-docker,${ALL_CONTRACTS})

.PHONY: clean-all-contracts
clean-all-contracts:
	-rm -f $(addprefix ${OUTPUT_DIR}/,${ALL_CONTRACTS})

.PHONY: clean-all-contracts-targets clean-all-contracts-targets-in-docker
clean-all-contracts-targets:
	@set -eu; \
		for contract in ${ALL_CONTRACTS}; do \
			cd "${CONTRACTS_DIR}/$${contract}"; cargo clean; cd ../; \
		done
clean-all-contracts-targets-in-docker:
	@docker run --rm -v "$$(pwd):/code" -w "/code" "${BUILD_ENV_DOCKER}" bash -c "make $(subst -in-docker,,$@)"

.PHONY: format-all-contracts format-all-contracts-in-docker
format-all-contracts:
	@set -eu; \
		for contract in ${ALL_CONTRACTS}; do \
			echo ">>> Format-check \"$${contract}\" contract ..."; \
			cd "${CONTRACTS_DIR}/$${contract}"; cargo fmt --all -- --check; cd ../; \
		done; \
		echo "[DONE] Format-check all contracts."
format-all-contracts-in-docker:
	@docker run --rm -v "$$(pwd):/code" -w "/code" "${BUILD_ENV_DOCKER}" bash -c "make $(subst -in-docker,,$@)"

.PHONY: lint-all-contracts lint-all-contracts-in-docker
lint-all-contracts:
	@set -eu; \
		for contract in ${ALL_CONTRACTS}; do \
			echo ">>> Lint \"$${contract}\" contract ..."; \
			cd "${CONTRACTS_DIR}/$${contract}"; cargo clippy --locked -- --deny warnings; cd ../; \
		done; \
		echo "[DONE] Lint all contracts."
lint-all-contracts-in-docker:
	@docker run --rm -v "$$(pwd):/code" -w "/code" "${BUILD_ENV_DOCKER}" bash -c "make $(subst -in-docker,,$@)"

${OUTPUT_DIR}/%:
	@set -eu; \
		contract_name="$(notdir $@)"; \
		contract_filename="${PROJECT_PREFIX}-$(subst -,_,$(notdir $@))"; \
		cd "${CONTRACTS_DIR}/$${contract_name}"; \
			cargo build --release; \
			cp "target/${RUST_TOOLCHAIN_TARGET}/release/$${contract_filename}" "../../${OUTPUT_DIR}/$${contract_name}"; \
			cd ..;

%-in-docker:
	@set -eu; \
		contract_name="$(subst -in-docker,,$@)"; \
		owner_and_group="$$(id -u):$$(id -g)"; \
		docker run --rm -v "$$(pwd):/code" -w "/code" "${BUILD_ENV_DOCKER}" \
			bash -c "make \"$${contract_name}\" && chown -v \"$${owner_and_group}\" \"${OUTPUT_DIR}/$${contract_name}\""

#
# Targets to Build Contracts
#

.PHONY: eth-light-client
eth-light-client: ${OUTPUT_DIR}/eth-light-client
