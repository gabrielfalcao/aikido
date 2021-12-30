# tomb

- [X] toggle visibility
- [X] pass context to process_keyboard and tick callbacks
- [x] tick every route of window
- [x] filter secrets
- [x] handle errors gracefully
- [x] match routes with crate router-recognizer
- [x] Confirmation dialog
  - [x] delete a secret
- [ ] Store file path in AES256Tomb because it has to be capable of
      reloading its contents from disk.
- [ ] Multiline modal editor
  - [ ] edit a secret

- [ ] render subroutes

# aes-256-cbc

- [X] "unit" tests
- [X] load default key from config if --key-file is not specified
- [ ] use open file instead of BufWriter
