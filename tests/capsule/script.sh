#!/bin/sh
formula="$1"
echo "$formula" | vagga _capsule run v35-calc bc
