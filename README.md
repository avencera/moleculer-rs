# moleculer-rs — work in progress

Inspired and compatible with [Moleculer JS](https://github.com/moleculerjs/moleculer)

Currently it only does the following:

- Is discoverable by other moleculer clients
- Only NATS transporter
- Only JSON serialization/deserialization
- Can `emit` and `broadcast` events
- Can receive events

Big missing pieces:

- Sending actions [#8](https://github.com/primcloud/moleculer-rs/issues/8)
- Receiving and responding to actions [#3](https://github.com/primcloud/moleculer-rs/issues/3)
