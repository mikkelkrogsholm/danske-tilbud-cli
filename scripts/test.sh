#!/bin/sh
# Keep this run's disposable data and compiler output in one owned directory.
set -eu

for argument do
    case "$argument" in
        --target-dir|--target-dir=*)
            echo 'scripts/test.sh owns --target-dir; omit that option.' >&2
            exit 2
            ;;
    esac
done

project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_root"
run_dir=$(mktemp -d "${TMPDIR:-/tmp}/tilbud-test.XXXXXXXX")
cleanup() {
    status=$?
    trap - 0 HUP INT TERM
    if ! rm -rf -- "$run_dir"; then
        echo "Could not remove test directory: $run_dir" >&2
        if [ "$status" -eq 0 ]; then status=1; fi
    fi
    exit "$status"
}
trap cleanup 0
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

mkdir "$run_dir/tmp"
export TMPDIR="$run_dir/tmp"
export TILBUD_TEST_TMPDIR="$run_dir/tmp"
"${CARGO:-cargo}" test --target-dir "$run_dir/target" "$@"
