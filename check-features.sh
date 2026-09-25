#!/usr/bin/env bash
# Checks every library crate under each supported feature set, individually.
#
# Crates must be checked one at a time: `cargo check --workspace` unifies features
# across members, so e.g. fileforge-test enabling `story` would hide no_std breakage.
#
# The `thumbv7em-none-eabihf` target has no `std` at all, so it's the only check that
# catches something (a dependency, or a stray `extern crate std`) quietly pulling std in.
set -u

NO_STD_TARGET=thumbv7em-none-eabihf

failed=()

check() {
  echo "==> cargo check $*"
  if ! cargo check --quiet "$@"; then
    failed+=("$*")
  fi
}

for crate in fileforge fileforge-std fileforge-nintendo; do
  check -p "$crate"
  check -p "$crate" --no-default-features
  check -p "$crate" --no-default-features --features alloc
  check -p "$crate" --no-default-features --target "$NO_STD_TARGET"
  check -p "$crate" --no-default-features --features alloc --target "$NO_STD_TARGET"
done

check -p fileforge --no-default-features --features std
check -p fileforge --features story
check --workspace --all-targets

if [ ${#failed[@]} -ne 0 ]; then
  echo
  echo "FAILED:"
  printf '  cargo check %s\n' "${failed[@]}"
  exit 1
fi

echo
echo "All feature checks passed."
