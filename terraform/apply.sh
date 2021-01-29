#!/usr/bin/env bash

source .env
export TF_VAR_replication=3
terraform destroy
