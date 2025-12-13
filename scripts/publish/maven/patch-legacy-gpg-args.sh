#!/usr/bin/env bash
set -euo pipefail

if grep -q '<arg>--loopback</arg>' packages/java/pom.xml; then
	sed -i 's/<arg>--loopback<\/arg>/<arg>loopback<\/arg>/g' packages/java/pom.xml
	echo "Patched legacy --loopback pinentry argument in packages/java/pom.xml" >>"${GITHUB_STEP_SUMMARY}"
fi
