#!/bin/sh

# Usage: sh foreach.sh f|d [ '(' {name-suffix} ')' ] 
# Example: sh foreach.sh f '(' .js ')'

SCRIPT_DIR=`dirname "$0"`
LOG_HANDLER="$SCRIPT_DIR/log-handler.sh"
FIND_TYPE_ARG="$1"
FIND_FILTER=""

shift

# ( -iname *.html -o -iname *.css -o -iname *.js )
if [ "$1" = '(' ]; then
    shift

    if [ "$1" != ')' ] && [ "$1" != '' ] ; then 
        FIND_FILTER="$1"
        shift
    fi

    if [ "$1" = ')' ]; then
        shift
    fi
fi

echo find . -type "$FIND_TYPE_ARG" -name '*'"$FIND_FILTER" -exec "$@" \; | sh "$LOG_HANDLER" "foreach"
find . -type "$FIND_TYPE_ARG" -name '*'"$FIND_FILTER" -exec echo "$@" \; -exec "$@" \; | sh "$LOG_HANDLER" "find"
