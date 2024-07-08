#!/bin/bash

xvfb-run --server-args="-screen 0 3840x2160x16 -ac +extension GLX +render -noreset" /usr/src/projectm_lambda/target/release/projectm_lambda "$@"
