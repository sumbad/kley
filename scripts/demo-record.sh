#!/bin/bash

sh ./scripts/demo-prepare.sh
cd ./.temp
vhs ../docs/demo/scenario_1.tape --output "../docs/demo/scenario_1.gif"
cd ..

sh ./scripts/demo-prepare.sh
cd ./.temp
vhs ../docs/demo/scenario_2.tape --output "../docs/demo/scenario_2.gif"
