# tomb

- [X] toggle visibility
- SPIKE: switch window route when changing tabs
  - not viable to implement with current architecture: the routing
    should be done in the application side, not inside of ironpunk.

  - Update 1: I went ahead and removed the history and location from
    window, only to realize that the location is necessary for
    matching the route among all the BoxedRoutes of the window.

    Next step: try to implement store a `on_route_changed` callback
    inside the window struct.

- [ ] filter secrets
- [ ] edit a secret
- [ ] delete a secret

# aes-256-cbc

- [X] "unit" tests
- [X] load default key from config if --key-file is not specified
- [ ] use open file instead of BufWriter
