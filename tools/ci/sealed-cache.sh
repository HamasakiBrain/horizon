#!/usr/bin/env bash
# Trial (ci/parity-speed-trial, 2026-11-19): game-derived folders sealed for a cache, and unsealed from one.
#
#   VANILLA_DATA_PASSPHRASE=... bash tools/ci/sealed-cache.sh seal ARCHIVE FOLDER...
#   VANILLA_DATA_PASSPHRASE=... bash tools/ci/sealed-cache.sh unseal ARCHIVE
#
# A cache entry can be read by a fork's pull request (it restores the default branch's entries, and its own copy of a
# workflow can name any key), so anything made from the vanilla data goes into one only as this archive: a POSIX tar
# (nanosecond modification times: tools/parity_shim.py replays an oracle run only if its outputs kept theirs), zstd,
# then the dataset's own encryption (openssl enc -aes-256-cbc -pbkdf2 -iter 600000 -md sha256, the passphrase of
# horizon-rust/vanilla-data). FOLDERs are absolute paths under the checkout or $RUNNER_TEMP; unseal puts them back
# where they were and refuses an archive holding anything else. The passphrase comes from this step's environment,
# is written to a private temporary file and unset at once; the file goes before the script ends, whatever happens.
# Only sizes, counts and seconds are printed, never a file name or the secret.
set -euo pipefail
umask 077
mode=${1:?seal or unseal}
archive=${2:?archive}
shift 2
: "${VANILLA_DATA_PASSPHRASE:?}" "${GITHUB_WORKSPACE:?}" "${RUNNER_TEMP:?}"
workspace=$(realpath -m "$GITHUB_WORKSPACE")
temp=$(realpath -m "$RUNNER_TEMP")
allowed() { case "$1/" in "$workspace"/?*|"$temp"/?*) return 0 ;; *) return 1 ;; esac; }

work=$(mktemp -d "$temp/sealed-cache.XXXXXX")
cleanup() { rm -rf "$work"; }
trap cleanup EXIT
printf '%s' "${VANILLA_DATA_PASSPHRASE//[$'\r\n']/}" > "$work/passphrase"
unset VANILLA_DATA_PASSPHRASE
cipher=(-aes-256-cbc -pbkdf2 -iter 600000 -md sha256 -pass "file:$work/passphrase")
started=$SECONDS

case "$mode" in
  seal)
    members=()
    for folder in "$@"; do
      folder=$(realpath -m "$folder")
      allowed "$folder" || { echo "::error::a sealed folder must be under the checkout or RUNNER_TEMP"; exit 2; }
      [ -e "$folder" ] && members+=("${folder#/}")
    done
    [ "${#members[@]}" -gt 0 ] || { echo "nothing to seal"; exit 0; }
    mkdir -p "$(dirname "$archive")"
    tar -C / --format=posix --pax-option=delete=atime,delete=ctime -cf - -- "${members[@]}" |
      zstd -q -T0 -3 |
      openssl enc -e "${cipher[@]}" > "$archive"
    echo "sealed ${#members[@]} folders, $(du -sk "${members[@]/#//}" | awk '{s+=$1} END {print s}') KiB," \
      "into $(( $(stat -c %s "$archive") / 1024 )) KiB in $((SECONDS - started)) s"
    ;;
  unseal)
    [ -f "$archive" ] || { echo "no archive"; exit 1; }
    # Only the passphrase's holders can make an archive that decrypts to a tar at all. Its member names go to a private
    # file (never the log) and must all lie in the two folders seal takes them from; tar refuses ".." itself.
    openssl enc -d "${cipher[@]}" < "$archive" | zstd -dq -T0 |
      tar -C / -xv --no-same-owner > "$work/members" 2> "$work/tar.err" ||
      { echo "::error::unsealing failed ($(wc -l < "$work/tar.err") tar messages)"; exit 1; }
    outside=0
    while IFS= read -r member; do allowed "/${member%/}" || outside=$((outside + 1)); done < "$work/members"
    [ "$outside" -eq 0 ] || { echo "::error::the archive held $outside paths outside the checkout and RUNNER_TEMP"; exit 1; }
    echo "unsealed $(( $(stat -c %s "$archive") / 1024 )) KiB in $((SECONDS - started)) s"
    ;;
  *) echo "::error::seal or unseal"; exit 2 ;;
esac
