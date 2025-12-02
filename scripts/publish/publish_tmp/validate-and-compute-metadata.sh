#!/usr/bin/env bash
set -euo pipefail

event="${GITHUB_EVENT_NAME}"
tag_input="${INPUT_TAG:-}"
dry_run_input_env="${INPUT_DRY_RUN:-false}"
ref_input_env="${INPUT_REF:-}"
release_tag="${EVENT_RELEASE_TAG:-}"
dispatch_tag="${EVENT_DISPATCH_TAG:-}"
dispatch_dry_run="${EVENT_DISPATCH_DRY_RUN:-}"
dispatch_ref="${EVENT_DISPATCH_REF:-}"

case "${event}" in
  workflow_dispatch)
    tag="${tag_input}"
    dry_run_input="${dry_run_input_env}"
    ref_input="${ref_input_env}"
    ;;
  release)
    tag="${release_tag}"
    dry_run_input="false"
    ref_input="refs/tags/${tag}"
    ;;
  repository_dispatch)
    tag="${dispatch_tag}"
    dry_run_input="${dispatch_dry_run}"
    ref_input="${dispatch_ref}"
    ;;
  *)
    tag="${GITHUB_REF_NAME:-}"
    dry_run_input="false"
    ref_input=""
    if [[ "${tag}" == *-pre* ]]; then
      dry_run_input="true"
    fi
    ;;
esac

if [[ -z "${tag}" ]]; then
  echo "Release tag could not be determined" >&2
  exit 1
fi

if [[ "${tag}" != v* ]]; then
  echo "Tag must start with 'v' (e.g., v2.6.0)" >&2
  exit 1
fi

version="${tag#v}"

if [[ -n "${ref_input}" ]]; then
  ref="${ref_input}"
else
  ref="refs/tags/${tag}"
fi

if [[ "${ref}" =~ ^[0-9a-f]{40}$ ]]; then
  checkout_ref="refs/heads/main"
  target_sha="${ref}"
elif [[ "${ref}" =~ ^refs/ ]]; then
  checkout_ref="${ref}"
  target_sha=""
else
  checkout_ref="refs/heads/${ref}"
  target_sha=""
fi

if [[ "${ref}" =~ ^[0-9a-f]{40}$ ]]; then
  matrix_ref="main"
elif [[ "${ref}" =~ ^refs/heads/(.+)$ ]]; then
  matrix_ref="${BASH_REMATCH[1]}"
elif [[ "${ref}" =~ ^refs/tags/(.+)$ ]]; then
  matrix_ref="${BASH_REMATCH[1]}"
else
  matrix_ref="${ref}"
fi

dry_run="${dry_run_input}"
if [[ -z "${dry_run}" ]]; then
  dry_run="false"
fi

if [[ "${ref}" =~ ^refs/tags/ ]]; then
  is_tag="true"
else
  is_tag="false"
fi

cat <<JSON > release-metadata.json
{
  "tag": "${tag}",
  "version": "${version}",
  "ref": "${ref}",
  "checkout_ref": "${checkout_ref}",
  "target_sha": "${target_sha}",
  "matrix_ref": "${matrix_ref}",
  "dry_run": ${dry_run},
  "is_tag": ${is_tag}
}
JSON

{
  echo "tag=${tag}"
  echo "version=${version}"
  echo "ref=${ref}"
  echo "dry_run=${dry_run}"
  echo "checkout_ref=${checkout_ref}"
  echo "target_sha=${target_sha}"
  echo "matrix_ref=${matrix_ref}"
  echo "is_tag=${is_tag}"
} >> "${GITHUB_OUTPUT}"
