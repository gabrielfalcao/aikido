MAKEFILE_PATH		:= $(realpath $(firstword $(MAKEFILE_LIST)))
GIT_ROOT		:= $(shell dirname $(MAKEFILE_PATH))
VENV_ROOT		:= $(GIT_ROOT)/.venv
RUST_LIBEXEC_DEBUG	:= $(GIT_ROOT)/target/debug
RUST_LIBEXEC_RELEASE	:= $(GIT_ROOT)/target/debug
RUST_LIBEXEC		:= $(realpath $(RUST_LIBEXEC_DEBUG) $(RUST_LIBEXEC_RELEASE))

PYTHON_PACKAGE_NAME	:= ki_aikido
PYTHON_CLI_NAME		:= ki-aikido
RUST_CLI_NAME		:= aikido
REQUIREMENTS_FILE	:= development.txt

PYTHON_PACKAGE_PATH	:= $(GIT_ROOT)/$(PYTHON_PACKAGE_NAME)
REQUIREMENTS_PATH	:= $(GIT_ROOT)/$(REQUIREMENTS_FILE)
PYTHON_CLI_PATH		:= $(VENV_ROOT)/bin/$(PYTHON_CLI_NAME)
RUST_CLI_PATH		:= $(RUST_LIBEXEC)/bin/$(RUST_CLI_NAME)
export VENV		?= $(VENV_ROOT)

######################################################################
# Phony targets (only exist for typing convenience and don't represent
#                real paths as Makefile expects)
######################################################################



all: | $(PYTHON_CLI_PATH)  # default target when running `make` without arguments

help:
	@egrep -h '^[^:]+:\s#\s' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?# "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# creates virtualenv
venv: | $(VENV)

# updates pip and setuptools to their latest version
develop: | $(VENV)/bin/python $(VENV)/bin/pip

# installs the requirements and the package dependencies
setup: | $(PYTHON_CLI_PATH) $(VENV)

# Convenience target to ensure that the venv exists and all
# requirements are installed
dependencies: $(VENV)
	@rm -f $(PYTHON_CLI_PATH) # remove PYTHON_CLI_PATH to trigger pip install
	$(MAKE) develop setup

html docs/build/html/index.html:
	$(MAKE) -C docs html

docs: html
	open docs/build/html/index.html

test tests: $(PYTHON_CLI_PATH) # runs all tests
	cargo check
	cargo test
	$(MAKE) unit functional

# Run all tests, separately
check:
	cargo check

e2e: # XXX: ...TK

# -> unit tests
unit: | $(VENV)/bin/pytest $(PYTHON_CLI_PATH)  # runs only unit tests
	@$(VENV)/bin/pytest tests/unit

# -> functional tests
functional:| $(VENV)/bin/pytest  $(PYTHON_CLI_PATH)  # runs functional tests
	cargo test
	@$(VENV)/bin/pytest tests/functional

# run main command-line tool
run: | $(PYTHON_CLI_PATH)
	@$(PYTHON_CLI_PATH) --help

# Pushes release of this package to pypi
push-release: build-release  # pushes distribution tarballs of the current version
	$(VENV)/bin/twine upload dist/*.tar.gz
	cargo publish

# Prepares release of this package prior to pushing to pypi
build:
	$(VENV)/bin/python setup.py build
	cargo build

# Prepares release of this package prior to pushing to pypi
build-release: clean tests
	$(VENV)/bin/python setup.py build sdist
	$(VENV)/bin/twine check dist/*.tar.gz
	cargo build --release

# Convenience target that runs all tests then builds and pushes a release to pypi
release: build-release push-release

# Convenience target to delete the virtualenv
purge:
	@rm -rf $(VENV) dist target
clean:
	@rm -rf dist

# Convenience target to format code with black with PEP8's default
# 80 character limit per line
fmt: | $(VENV)/bin/black
	find src -type f -name '*.rs' -exec rustfmt -v {} \;
	@$(VENV)/bin/black -l 80 $(PYTHON_PACKAGE_PATH) tests

##############################################################
# Real targets (only run target if its file has been "made" by
#               Makefile yet)
##############################################################

# creates virtual env if necessary and installs pip and setuptools
$(VENV): | $(REQUIREMENTS_PATH)  # creates $(VENV) folder if does not exist
	echo "Creating virtualenv in $(VENV_ROOT)" && python3 -mvenv $(VENV)

# installs pip and setuptools in their latest version, creates virtualenv if necessary
$(VENV)/bin/python $(VENV)/bin/pip: $(VENV) # installs latest pip
	@test -e $(VENV)/bin/python || $(MAKE) $(VENV)
	@test -e $(VENV)/bin/pip || $(MAKE) $(VENV)
	@echo "Installing latest version of pip and setuptools"
	@$(VENV)/bin/pip install -U pip setuptools

 # installs latest version of the "black" code formatting tool
$(VENV)/bin/black: | $(VENV)/bin/pip
	$(VENV)/bin/pip install -U black

# installs this package in "edit" mode after ensuring its requirements are installed

$(VENV)/bin/pytest $(PYTHON_CLI_PATH): | $(VENV) $(VENV)/bin/pip $(VENV)/bin/python $(REQUIREMENTS_PATH)
	$(VENV)/bin/pip install -r $(REQUIREMENTS_PATH)
	$(VENV)/bin/pip install -e .

# ensure that REQUIREMENTS_PATH exists
$(REQUIREMENTS_PATH):
	@echo "The requirements file $(REQUIREMENTS_PATH) does not exist"
	@echo ""
	@echo "To fix this issue:"
	@echo "  edit the variable REQUIREMENTS_NAME inside of the file:"
	@echo "  $(MAKEFILE_PATH)."
	@echo ""
	@exit 1

###############################################################
# Declare all target names that exist for convenience and don't
# represent real paths, which is what Make expects by default:
###############################################################

.PHONY: \
	all \
	fmt \
	check \
	docs \
	html \
	build-release \
	clean \
	dependencies \
	develop \
	push-release \
	purge \
	release \
	setup \
	run \
	test \
	tests \
	unit \
	functional

.DEFAULT_GOAL	:= help
