set dotenv-load

TEMP_DIR := ".wrangler"
PID_FILE := "worker.pid"

#########################
# All Workers
#########################

# Install the required dependencies.
init:
	#!/usr/bin/env bash
	set -euo pipefail

	echo -e "\n#########################"
	echo "Installing required dependencies..."
	echo -e "#########################\n"

	cargo update || {
		echo "Failed to update Cargo dependencies." ;
		exit 1 ;
	}

	cargo install --verbose \
		cargo-watch \
		wasm-pack \
		worker-build \
		wasm-opt \
		toml-cli \
		teracli || {
			echo "Failed to install Cargo tools." ;
			exit 1 ;
		}

# Login the Cloudflare
login:
	#!/usr/bin/env bash
	set -euo pipefail

	wrangler login --browser=false

# Check all workers in the Cargo workspace.
check:
	#!/usr/bin/env bash
	set -euo pipefail

	echo -e "\n#########################"
	echo "Checking all Workers..."
	echo -e "#########################\n"

	cargo update --workspace || {
		echo "Failed to update Cargo dependencies." ;
		exit 1 ;
	}

	cargo check --workspace --target wasm32-unknown-unknown || {
		echo "Failed to check Cargo workspace." ;
		exit 1 ;
	}

	cargo clippy --target wasm32-unknown-unknown -- -D warnings || {
		echo "Failed to run Clippy." ;
		exit 1 ;
	}

# Push the latest changes and test the CI.
push:
	#!/usr/bin/env bash
	set -euo pipefail

	echo "Pushing latest changes..."

	git add --all || {
		echo "Failed to add all changes." ;
		exit 1 ;
	}

	git commit -m "test: just -q push" --no-gpg-sign || {
		echo "Failed to commit changes." ;
		exit 1 ;
	}

	git push || {
		echo "Failed to push changes." ;
		exit 1 ;
	}

#########################
# Worker
#########################

# Sync the assets to the R2 bucket.
rclone-worker worker:
	#!/usr/bin/env bash
	set -euo pipefail

	echo "Syncing R2 Bucket for worker: {{worker}}."

	cd workers/{{worker}} || {
		echo "Failed to cd into worker directory: {{worker}}." ;
		exit 1 ;
	}

	if [ -d bucket ];
	then
		echo "Syncing bucket for worker {{ worker }}..." ;
		if ! rclone listremotes 2>/dev/null | grep -q "^r2:";
		then
			echo "ERROR: R2 remote not configured. Please run 'rclone config' to set up the R2 remote first." ;
			echo "Configuration instructions:" ;
			echo "1. Run 'rclone config'" ;
			echo "2. Choose 'n' for new remote" ;
			echo "3. Name: r2" ;
			echo "4. Storage: s3" ;
			echo "5. Provider: Cloudflare" ;
			echo "6. Enter your R2 credentials" ;
			exit 1 ;
		else
			rclone sync --progress bucket r2:tresr-community-chatbot-preview || {
				echo "Failed to sync bucket for worker {{ worker }}." ;
				exit 1 ;
			} ;
		fi ;
	else
		echo "No bucket found for worker {{ worker }}." ;
	fi

# Build the Tailwind CSS.
tailwind-worker worker:
	#!/usr/bin/env bash
	set -euo pipefail

	cd workers/{{worker}} || {
		echo "Failed to cd into worker directory: {{worker}}." ;
		exit 1 ;
	}

	if [ -f tailwind.config.js ];
	then
		if [ -f astro.config.mjs ];
		then
			echo "Astro detected, skipping Tailwind CSS for worker: {{worker}}." ;
		else
			echo "Building Tailwind CSS for worker: {{worker}}." ;
			tailwindcss \
			--config tailwind.config.js \
			--input bucket/static/css/tailwind.css \
			--minify \
			--output bucket/static/css/style.css \
			--postcss || {
				echo "Failed to build Tailwind CSS for worker: {{worker}}." ;
				exit 1 ;
			}
		fi
	else
		echo "No tailwind.config.js found for worker: {{worker}}." ;
	fi

# Install the NPM dependencies.
bun-worker-install worker:
	#!/usr/bin/env bash
	set -euo pipefail

	cd workers/{{worker}} || {
		echo "Failed to cd into worker directory: {{worker}}." ;
		exit 1 ;
	}

	if [ -f package.json ];
	then
		echo "Running clean script for worker: {{worker}}." ;
		bun run clean || {
			echo "Failed to run clean script for worker: {{worker}}." ;
			exit 1 ;
		}

		echo "Installing NPM dependencies for worker: {{worker}}." ;
		bun install --save || {
			echo "Failed to install NPM dependencies for worker: {{worker}}." ;
			exit 1 ;
		}
	else
		echo "No package.json found for worker: {{worker}}." ;
	fi

# Update the NPM dependencies.
bun-worker-update worker:
	#!/usr/bin/env bash
	set -euo pipefail

	cd workers/{{worker}} || {
		echo "Failed to cd into worker directory: {{worker}}." ;
		exit 1 ;
	}

	if [ -f package.json ];
	then
		echo "Updating NPM dependencies for worker: {{worker}}." ;
		bun update --force --save || {
			echo "Failed to update NPM dependencies for worker: {{worker}}." ;
			exit 1 ;
		}
	else
		echo "No package.json found for worker: {{worker}}." ;
	fi

# Update the Cargo dependencies.
cargo-worker-update worker:
	#!/usr/bin/env bash
	set -euo pipefail

	cd workers/{{worker}} || {
		echo "Failed to cd into worker directory: {{worker}}." ;
		exit 1 ;
	}

	if [ -f Cargo.toml ];
	then
		echo "Updating Cargo dependencies for worker: {{worker}}." ;
		cargo update --recursive || {
			echo "Failed to update Cargo dependencies for worker: {{worker}}." ;
			exit 1 ;
		}
	else
		echo "No Cargo.toml found for worker: {{worker}}." ;
	fi

# Start Wrangler in development mode.
start-worker worker:
	#!/usr/bin/env bash
	set -euo pipefail

	echo "Starting Worker: {{worker}}."

	case "{{worker}}" in
		"index")
			PORT=9001
			REMOTE="false"
			EXTRA_ARGS="--assets=./bucket"
		;;
		"ui")
			PORT=9002
			REMOTE="false"
			EXTRA_ARGS=""
		;;
		"api")
			PORT=9003
			REMOTE="false"
			EXTRA_ARGS=""
		;;
		"frontend")
			PORT=9100
			REMOTE="false"
			EXTRA_ARGS=""
		;;
		"backend")
			PORT=9200
			REMOTE="false"
			EXTRA_ARGS=""
		;;
		*) echo "Unknown worker: {{worker}}" && exit 1 ;;
	esac

	cd  workers/{{worker}}

	echo  "Starting Worker {{worker}} on port $PORT..."

	wrangler dev \
		--env=development \
		--remote=$REMOTE \
		--ip=0.0.0.0 \
		--port=$PORT \
		${EXTRA_ARGS:-} \
		> {{TEMP_DIR}}/{{worker}}.log 2>&1 \
		&

	# Make sure the PID directory exists.
	mkdir -p {{TEMP_DIR}}

	# Store the PID of the worker.
	echo  $! >> {{TEMP_DIR}}/{{PID_FILE}}

	# Wait for the Workers port to be open.
	WORKER_STARTED="false"
	WORKER_ATTEMPTS=0
	while [ "${WORKER_STARTED}" = "false" ];
	do

		WORKER_ATTEMPTS=$((WORKER_ATTEMPTS + 1)) ;

		if ! curl -s http://localhost:${PORT}/health > /dev/null;
		then
			echo "Waiting for Worker {{worker}} on port ${PORT} to start (attempt ${WORKER_ATTEMPTS})..." ;
			sleep 5 ;
		else
			WORKER_STARTED="true" ;
		fi ;

		if [ ${WORKER_ATTEMPTS} -ge 10 ];
		then
			echo "Worker {{worker}} on port ${PORT} failed to start!" ;
			exit 1 ;
		fi ;

	done

	echo "Started Worker {{worker}} on port $PORT."

# Stop a running Worker.
stop-worker worker:
	#!/usr/bin/env bash
	set -euo pipefail

	echo "Stopping Worker: {{worker}}."

	cd workers/{{worker}} || {
		echo "Failed to cd into worker directory: {{worker}}." ;
		exit 1 ;
	}

	if [[ -f {{TEMP_DIR}}/{{PID_FILE}} ]];
	then
		for PID in $(cat {{TEMP_DIR}}/{{PID_FILE}});
		do
			if kill -0 ${PID} 2> /dev/null;
			then
				echo "Terminating Worker {{worker}} on PID ${PID}..."
				kill ${PID} || true
				while kill -0 "${PID}" 2> /dev/null;
				do
					echo "Waiting for Worker {{worker}} on PID ${PID} to terminate..."
					sleep 5
				done
			else
				echo "Worker {{worker}} on PID ${PID} was not found to be running."
			fi
		done
		rm -f {{TEMP_DIR}}/{{PID_FILE}}
		rm -f {{TEMP_DIR}}/{{worker}}.log
		echo "Worker {{worker}} has stopped and logs have been cleaned up."
	else
		echo "Worker {{worker}} was not found to be running."
	fi

#########################
# Shortcuts
#########################

# Shortcut to update all dependencies.
update:
	#!/usr/bin/env bash
	set -euo pipefail

	echo -e "\n#########################"
	echo "Checking all Workers..."
	echo -e "#########################\n"

	just -q check

	echo  -e "\n#########################"
	echo  "Updating all Workers..."
	echo  -e "#########################\n"

	WORKER_NAMES=$( find workers -mindepth 1 -maxdepth 1 -type d ! -name "_templates" ! -name "_disabled*" -printf "%f\n")
	for WORKER in  ${WORKER_NAMES[@]};
	do
		echo -e "\n#########################"
		echo -e "Updating Worker ${WORKER}..."
		echo -e "#########################\n"

		if [ -f workers/${WORKER}/Cargo.toml ];
		then
			just -q cargo-worker-update ${WORKER}
		fi

		if [ -f workers/${WORKER}/package.json ];
		then
			just -q bun-worker-update ${WORKER}
		fi

	done

# Shortcut to check and build all workers.
build:
	#!/usr/bin/env bash
	set  -euo pipefail

	echo  -e "\n#########################"
	echo  "Checking all Workers..."
	echo  -e "#########################\n"

	just -q check

	echo  -e "\n#########################"
	echo  "Building all Workers..."
	echo  -e "#########################\n"

	WORKER_NAMES=$(find workers -mindepth 1 -maxdepth 1 -type d ! -name "_templates" ! -name "_disabled*" -printf "%f\n")
	for WORKER in ${WORKER_NAMES[@]};
	do
		echo -e "\n#########################"
		echo -e "Building Worker ${WORKER}..."
		echo -e "#########################\n"

		if [ -f workers/${WORKER}/tailwind.config.js ];
		then
			just -q tailwind-worker ${WORKER}
		fi

		if [ -f workers/${WORKER}/package.json ];
		then
			just -q bun-worker-install ${WORKER}
		fi

	done

# Shortcut to start all workers.
start:
	#!/usr/bin/env bash
	set -euo pipefail

	just stop

	echo -e "\n#########################"
	echo "Building all Workers..."
	echo -e "#########################\n"

	just -q build

	echo -e "\n#########################"
	echo "Starting all Workers..."
	echo -e "#########################\n"

	WORKER_NAMES=$(find workers -mindepth 1 -maxdepth 1 -type d ! -name "_templates" ! -name "_disabled*" -printf "%f\n")
	for WORKER in ${WORKER_NAMES[@]};
	do
		echo -e "Starting Worker ${WORKER}..."
		just -q start-worker ${WORKER}
	done

	echo -e "\n#########################"
	echo "Workers are now starting..."
	echo -e "#########################\n"

# Shortcut to stop all workers.
stop:
	#!/usr/bin/env bash
	set -euo pipefail

	echo -e "\n#########################"
	echo "Stopping all Workers..."
	echo -e "#########################\n"

	WORKER_NAMES=$(find workers -mindepth 1 -maxdepth 1 -type d ! -name "_templates" ! -name "_disabled*" -printf "%f\n")
	for WORKER in ${WORKER_NAMES[@]};
	do
		just -q stop-worker ${WORKER}
	done

	echo -e "\n#########################"
	echo "Workers are now stopping..."
	echo -e "#########################\n"

# Shortcut to cleanup all build caches
clean:
	#!/usr/bin/env bash
	set -euo pipefail

	echo -e "\n#########################"
	echo -e "Cleaning up build caches..."
	echo -e "#########################\n"

	cargo clean --verbose

	rm -rf ~/.cargo/registry/cache ~/.cargo/git || true

	# Remove any local rust-analyzer caches (safe even if none exist)
	find . -name ".rust-analyzer*" -type d -prune -exec rm -rf {} + 2>/dev/null || true

	# Global cache cleanup (do this manually if the above doesn't suffice; restarts rust-analyzer)
	rm -rf ~/.cache/rust-analyzer || true

	# Re-generate metadata
	cargo metadata >/dev/null 2>&1
	cargo check --workspace

# Diagnose rust-analyzer/Cargo workspace issues.
diagnose:
	#!/usr/bin/env bash
	set -euo pipefail

	echo -e "\n#########################"
	echo "Rust Toolchain Info"
	echo -e "#########################\n"

	rustc --version
	cargo --version
	rust-analyzer --version || echo "rust-analyzer NOT FOUND (enter devenv shell!)"

	echo -e "\n#########################"
	echo "Cargo Metadata (workspace)"
	echo -e "#########################\n"

	cargo metadata --format-version 1 | jq 'del(.packages[] | select(.id | contains("virtual")))' || cargo metadata

	echo -e "\n#########################"
	echo "Rust Analyzer Diagnostics"
	echo -e "#########################\n"

	rust-analyzer prime-caches . -v

	rust-analyzer diagnostics . -v

	rust-analyzer analysis-stats . -v

	echo -e "\n#########################"
	echo "Workspace Root Check"
	echo -e "#########################\n"

	echo "Root Cargo.toml: $(test -f Cargo.toml && echo 'OK' || echo 'MISSING')"
	echo "Members: crates/utils OK? $(test -d crates/utils && echo 'OK' || echo 'MISSING')"
	echo "Members: workers/backend OK? $(test -d workers/backend && echo 'OK' || echo 'MISSING')"

	echo -e "\n#########################"
	echo "Next: Enter 'devenv shell', run 'cargo check --workspace', reload editor."
	echo -e "#########################\n"

# Shortcut to tail all logs.
tail:
	#!/usr/bin/env bash
	set -euo pipefail

	TAIL_PIDS=()

	# Setup trap to handle CTRL+C gracefully
	cleanup() {
		echo -e "\n#########################"
		echo -e "Stopping all tail processes..."
		echo -e "#########################\n"

		# Kill all background processes in TAIL_PIDS array
		if [ ${#TAIL_PIDS[@]} -gt 0 ];
		then
			echo -e "Killing all tail processes: ${TAIL_PIDS[@]}"
			kill ${TAIL_PIDS[@]} 2>/dev/null || true
		fi
		exit 0
	}
	trap cleanup SIGINT SIGTERM

	# Start tailing worker logs
	WORKER_NAMES=$( find workers -mindepth 1 -maxdepth 1 -type d ! -name "_templates" ! -name "_disabled*" -printf "%f\n")
	for WORKER in ${WORKER_NAMES[@]};
	do
		echo -e "\n#########################"
		echo -e "Tailing Worker ${WORKER}..."
		echo -e "#########################\n"

		tail -F workers/${WORKER}/{{TEMP_DIR}}/${WORKER}.log &
		TAIL_PIDS+=($!)
	done

	# Wait indefinitely until CTRL+C or the PIDS are gone
	while true;
	do
		echo -e "\n#########################"
		echo "Watching PIDS: ${TAIL_PIDS[@]}"
		echo "Press CTRL+C to stop tailing all logs."
		echo -e "#########################\n"
		sleep 30
	done
