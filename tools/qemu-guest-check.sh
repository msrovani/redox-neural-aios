#!/usr/bin/env sh
# Smoke E2E Redox Neural AIOS — executar DENTRO do guest QEMU (Fase 0–7).
set -eu

echo "=== qemu-guest-check: memory scheme (URI bridge) ==="
export REDOX_MEMORY_BACKEND=scheme
export REDOX_MEMORY_SCHEME_ROOT=/scheme/memory
export REDOX_MEMORY_SCHEME_NATIVE=1
export REDOX_OS_TARGET=1
export REDOX_AIOS_CAPS=factory_exec,memory_recall,hitl_approve

memory remember "qemu guest boot ok" --scope boot
memory recall "qemu" --scope boot
memory health || true

echo "=== qemu-guest-check: hermes factory ==="
hermes "/factory" || true
hermes "que horas são" || true

echo "=== qemu-guest-check: escada (3x intent) ==="
hermes "demo intent one" || true
hermes "demo intent two" || true
hermes "demo intent three" || true
hermes "/evolve" || true
hermes "/promote list" || true

echo "=== qemu-guest-check: os-release ==="
grep -i "Redox Neural AIOS" /usr/lib/os-release

echo "=== qemu-guest-check OK ==="
