docker compose -f services.yaml up

NOTES:
1. Each socket connection should register itself with a connection registry and de-register itself on close
2. Each socket handler should log its access to each outgoing message so that messages can be removed once they've been accessed by all handlers