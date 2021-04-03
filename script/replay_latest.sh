head=`ls replay | cut -d'-' -f1 | sort -nu | tail -1`
tttz-replayer path ./replay/$head-0.tttz_replay path ./replay/$head-1.tttz_replay constant
