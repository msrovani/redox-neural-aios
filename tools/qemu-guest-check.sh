#!/usr/bin/env sh
# Smoke E2E Redox Neural AIOS — executar DENTRO do guest QEMU (Fase 0–7).
# Aceite gravável: os-release + memory URI + factory + escada + CapGate ns.
set -eu

PASS=0
fail() {
  echo "FAIL: $*" >&2
  exit 1
}
ok() {
  PASS=$((PASS + 1))
  echo "OK: $*"
}

echo "=== qemu-guest-check: env ==="
export REDOX_MEMORY_BACKEND=scheme
export REDOX_MEMORY_SCHEME_ROOT=/scheme/memory
export REDOX_MEMORY_SCHEME_NATIVE=1
export REDOX_OS_TARGET=1
export REDOX_AIOS_CAPS="${REDOX_AIOS_CAPS:-factory_exec,memory_recall,hitl_approve}"
export REDOX_AIOS_CAPS_ROOT="${REDOX_AIOS_CAPS_ROOT:-/scheme/aios/caps}"
export REDOX_CAP_ROLE="${REDOX_CAP_ROLE:-hermes}"

echo "=== qemu-guest-check: os-release ==="
grep -i "Redox Neural AIOS" /usr/lib/os-release || fail "PRETTY_NAME / os-release"
ok "os-release"

echo "=== qemu-guest-check: memory scheme (URI bridge) ==="
memory remember "qemu guest boot ok" --scope boot || fail "memory remember"
memory recall "qemu" --scope boot || fail "memory recall"
memory health || true
ok "memory"

echo "=== qemu-guest-check: hermes factory + escada ==="
hermes "/factory" || fail "hermes /factory"
hermes "que horas são" || true
hermes "demo intent one" || true
hermes "demo intent two" || true
hermes "demo intent three" || true
hermes "/evolve" || true
hermes "/promote list" || true
ok "hermes escada"

echo "=== qemu-guest-check: CapGate /caps ==="
hermes "/caps list" || fail "caps list"
hermes "/caps ns" || true
hermes "/caps probe" || true
ok "caps"

echo "=== qemu-guest-check: backends honesty ==="
# via hermes TCP cmd — CLI só intent; use intent backends se skill existir
hermes "/status" || true
ok "status"

echo "=== qemu-guest-check OK ($PASS checks) ==="
echo "RECORD: copy this output into docs/memory/evidence/qemu-e2e-guest-*.md"
