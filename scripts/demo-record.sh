#!/usr/bin/env bash
set -euo pipefail

for scenario in 0 1 2; do
    bash ./scripts/demo-prepare.sh
    (
        cd ./.temp || exit 1
        vhs "../docs/demo/scenario_${scenario}.tape" \
            --output "../docs/demo/scenario_${scenario}.gif"
        cd ..
    )
done
