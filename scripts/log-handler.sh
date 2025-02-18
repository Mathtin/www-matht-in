#!/bin/sh

# Usage: cmd arg1 ... | sh log-handler.sh {module}
# Example: cat file.txt | sh log-handler.sh cat

while read line; do
    echo "`date '+%Y-%m-%d %H:%M:%S.%3N'` "'['"$1"'] '"$line"
done
