#!/usr/bin/env bash

set -euo pipefail

# The type of release is passed as $1
# The release type can be either "debug" or "release"
# If no release type is passed, the default is "release"
WORKER_NAME="${1:-}"
RELEASE_TYPE="${2:-}"

if [[ ${WORKER_NAME:-NONE} == "NONE" ]]; then
	echo "No worker name provided. Please pass the worker name as the first argument"
	exit 1
fi

if [[ ${RELEASE_TYPE:-EMPTY} == "EMPTY" ]]; then
	echo "No release type provided. Defaulting to 'release'"
	RELEASE_TYPE="release"
fi

echo "Building Cloudflare Worker $WORKER_NAME"

# This script is run from different contexts, so we need to check if the worker dir exists.
if [[ -f "wrangler.toml" ]]; then

	echo "Running wrangler from current directory"
	WORKER_DIR="."

elif [[ -d "workers/${WORKER_NAME}" ]]; then

	WORKER_DIR="workers/${WORKER_NAME}"
	pushd "${WORKER_DIR}" || {
		echo "Failed to change directory to ${WORKER_DIR}"
		exit 1
	}

elif [[ -d "../workers/${WORKER_NAME}" ]]; then

	WORKER_DIR="../workers/${WORKER_NAME}"
	pushd "${WORKER_DIR}" || {
		echo "Failed to change directory to ${WORKER_DIR}"
		exit 1
	}

else

	echo "A directory for Worker $WORKER_NAME could not be found!"
	exit 1

fi

echo "Clearing cargo"

cargo clean || {
	echo "Failed to clean cargo"
	exit 1
}

echo "Clearing build directory"

if [[ -d build ]]; then

	rm -rf build || {
		echo "Failed to clear build directory"
		exit 1
	}

fi

echo "Installing worker-build"

if type worker-build >/dev/null 2>&1; then

	echo "worker-build already installed"

else

	cargo install --force worker-build || {
		echo "Failed to install worker-build"
		exit 1
	}

fi

echo "Building worker ($RELEASE_TYPE)"

case $RELEASE_TYPE in

debug)

	worker-build --debug || {
		echo "Failed to build worker (debug)"
		exit 1
	}

	;;

release)

	worker-build --release || {
		echo "Failed to build worker (release)"
		exit 1
	}

	;;

*)

	echo "Invalid release type: $RELEASE_TYPE"
	exit 1

	;;

esac

popd 2>/dev/null || true

echo "Build complete for Cloudflare Worker $WORKER_NAME"
exit 0
