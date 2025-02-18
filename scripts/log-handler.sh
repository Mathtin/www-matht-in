#!/bin/sh

# Usage: cmd arg1 ... | sh log-handler.sh {module}
# Example: cat file.txt | sh log-handler.sh cat

MODULE=$1

shift

while read line; do
    echo "`date '+%Y-%m-%d %H:%M:%S.%3N'` "'['"$MODULE"'] '"$@""$line"
done
