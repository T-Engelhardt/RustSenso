#!/usr/bin/env bash
set -e
source secret
$1 -s $VaillantSerial --user $VaillantAppUSER --pwd $VaillantAppPWD