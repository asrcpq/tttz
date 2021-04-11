#!/bin/bash
set -e
i=1
if [[ "$1" =~ ^[0-9]*$ ]]; then
	i=$1
	shift
fi
head=`ls replay | cut -d'-' -f1 | sort -nu | tail -$i | head -1`
tttz-replayer path ./replay/$head-0.tttz_replay path ./replay/$head-1.tttz_replay $@
