#!/usr/bin/env bash
set -euo pipefail

# NPM_TOKEN authentication for scoped packages
# NOTE: Trusted publishing can be enabled AFTER the first successful npm publish.
# For now, we use NPM_TOKEN from GitHub secrets (see .npmrc configuration below).
if [[ -z "${NPM_TOKEN:-}" ]]; then
	echo "ERROR: NPM_TOKEN is not set. Required for publishing scoped @kreuzberg/* packages."
	exit 1
fi

# Configure npm authentication
cat >~/.npmrc <<'EOF'
//registry.npmjs.org/:_authToken=${NPM_TOKEN}
@kreuzberg:registry=https://registry.npmjs.org/
EOF

publish_log=$(mktemp)
set +e
npm publish --access public 2>&1 | tee "${publish_log}"
status=${PIPESTATUS[0]}
set -e
if [[ "${status}" -ne 0 ]]; then
	if grep -q "previously published versions" "${publish_log}"; then
		echo "Node package already published; skipping."
	else
		exit "${status}"
	fi
fi
