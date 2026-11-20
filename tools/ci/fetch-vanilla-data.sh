#!/usr/bin/env bash
# Trial (ci/parity-blacksmith-trial, 2026-11-19): one encrypted dataset of the private repository
# horizon-rust/vanilla-data, unpacked into DESTINATION, a folder outside the checkout.
#
#   VANILLA_DATA_DEPLOY_KEY=... VANILLA_DATA_PASSPHRASE=... bash tools/ci/fetch-vanilla-data.sh DATASET DESTINATION
#
# The two secrets come from the environment of this one step. They are written to files in a private temporary folder
# and unset at once; that folder, with the key, the passphrase and the clone, is removed before the script ends,
# whatever happens. Only sizes, counts and timings are printed: never a file name from the archive, never a secret.
# DATASET/MANIFEST: "sha256 <digest of the encrypted whole>", then the chunk names in order.
set -euo pipefail
umask 077
dataset=${1:?dataset}
destination=${2:?destination}
[[ "$dataset" =~ ^[A-Za-z0-9._-]+$ ]] || { echo "::error::bad dataset name"; exit 2; }
: "${VANILLA_DATA_DEPLOY_KEY:?}" "${VANILLA_DATA_PASSPHRASE:?}"
case "$(realpath -m "$destination")/" in
  "$(realpath -m "${GITHUB_WORKSPACE:-$PWD}")"/*) echo "::error::the destination must be outside the checkout"; exit 2 ;;
esac

work=$(mktemp -d "${RUNNER_TEMP:-/tmp}/vanilla-fetch.XXXXXX")
cleanup() { rm -rf "$work"; }
trap cleanup EXIT
printf '%s\n' "${VANILLA_DATA_DEPLOY_KEY//$'\r'/}" > "$work/key"
printf '%s' "${VANILLA_DATA_PASSPHRASE//[$'\r\n']/}" > "$work/passphrase"
unset VANILLA_DATA_DEPLOY_KEY VANILLA_DATA_PASSPHRASE
ssh-keygen -y -f "$work/key" > /dev/null || { echo "::error::VANILLA_DATA_DEPLOY_KEY is not a private key"; exit 1; }
# github.com's ed25519 host key, pinned: its fingerprint is SHA256:+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU, as
# GitHub publishes it (docs.github.com, "GitHub's SSH key fingerprints").
echo "github.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl" > "$work/known_hosts"
export GIT_SSH_COMMAND="ssh -i $work/key -o IdentitiesOnly=yes -o IdentityAgent=none -o UserKnownHostsFile=$work/known_hosts -o GlobalKnownHostsFile=/dev/null -o StrictHostKeyChecking=yes -o HostKeyAlgorithms=ssh-ed25519 -o BatchMode=yes"
export GIT_TERMINAL_PROMPT=0

started=$SECONDS
# Only this dataset's blobs: a shallow, blobless clone with a sparse checkout of its folder.
git clone -q --depth 1 --filter=blob:none --sparse git@github.com:horizon-rust/vanilla-data.git "$work/repo"
git -C "$work/repo" sparse-checkout set "$dataset" > /dev/null
cloned=$SECONDS
folder="$work/repo/$dataset"
[ -f "$folder/MANIFEST" ] || { echo "::error::no dataset $dataset"; exit 1; }
expected=$(sed -n 's/^sha256 \([0-9a-f]\{64\}\)$/\1/p' "$folder/MANIFEST")
mapfile -t chunks < <(sed -n '2,$p' "$folder/MANIFEST")
for chunk in "${chunks[@]}"; do
  [[ "$chunk" =~ ^[A-Za-z0-9._-]+$ && -f "$folder/$chunk" ]] || { echo "::error::bad chunk in the manifest"; exit 1; }
done
found=$(cd "$folder" && cat -- "${chunks[@]}" | sha256sum | cut -c1-64)
[ -n "$expected" ] && [ "$found" = "$expected" ] || { echo "::error::the encrypted archive does not match its manifest"; exit 1; }
encrypted=$(cd "$folder" && cat -- "${chunks[@]}" | wc -c)
verified=$SECONDS

rm -rf "$destination" && mkdir -p "$destination"
(cd "$folder" && cat -- "${chunks[@]}") |
  openssl enc -d -aes-256-cbc -pbkdf2 -iter 600000 -md sha256 -pass "file:$work/passphrase" |
  zstd -dq -T0 |
  tar -x --no-same-owner --no-same-permissions -C "$destination"
unpacked=$SECONDS
cleanup
trap - EXIT
echo "dataset $dataset: ${#chunks[@]} chunks, $encrypted bytes encrypted, $(du -sk "$destination" | cut -f1) KiB" \
  "and $(find "$destination" -type f | wc -l) files unpacked; clone $((cloned - started)) s, verify" \
  "$((verified - cloned)) s, decrypt and unpack $((unpacked - verified)) s"
