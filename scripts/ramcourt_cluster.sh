#!/bin/bash
set -eux

num=5
action="setup"

while getopts n:a: flag
do
    case "${flag}" in
        n) num=${OPTARG};;
        a) action=${OPTARG};;
    esac
done
echo "Number: $num";
echo "Action: $action";

if [ "$action" == "setup" ]; then
    # Build image
    docker build -t ramcourt:latest --label ramcourt.cluster=true .

    # setup
    docker network create --subnet 10.0.19.0/24 ramcourt-cluster || true
    for i in $(seq 1 $num); do
        docker run -d --name ramcourt-cluster-$i --label ramcourt.cluster=true --network ramcourt-cluster --ip 10.0.19.$((i+1)) -p $((19000+i)):8080 -v $(pwd)/data-${i}:/data --cap-add=NET_ADMIN ramcourt:latest
    done
elif [ "$action" == "teardown" ]; then
    # teardown
    docker rm -f $(docker ps -a -q --filter label=ramcourt.cluster=true) || true
    docker network rm ramcourt-cluster
    docker image prune -f --filter label=ramcourt.cluster=true
fi
