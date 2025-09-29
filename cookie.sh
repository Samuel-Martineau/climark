#! /usr/bin/env nix-shell
#! nix-shell -i sh -p sqlite
sqlite3 "file:$(realpath ~/.mozilla/firefox/*.default/cookies.sqlite | head)?mode=ro&immutable=1" "SELECT value FROM moz_cookies WHERE name = 'cm_session_id';"
