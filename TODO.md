# tomb

- [X] toggle visibility
- SPIKE: switch window route when changing tabs
  - not viable to implement with current architecture: the routing
    should be done in the application side, not inside of ironpunk.

- [ ] filter secrets
- [ ] edit a secret
- [ ] delete a secret

# aes-256-cbc

- [X] "unit" tests
- [X] load default key from config if --key-file is not specified
- [ ] use open file instead of BufWriter
