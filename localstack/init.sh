#!/usr/bin/env bash

set -x
awslocal sqs create-queue --queue-name demo-event-stream
set +x
