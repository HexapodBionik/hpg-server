#!/usr/bin/bash

set -xe

modprobe gadgetfs
mkdir /dev/gadget
mount -t gadgetfs gadgetfs /dev/gadget
