# moleculer-rs — work in progress

Inspired and compatible with [Moleculer JS](https://github.com/moleculerjs/moleculer)

Currently it only does the following:

- Is discoverable by other moleculer clients
- Only NATS transporter
- Only JSON serialization/deserialization
- Can `emit` and `broadcast` events
- Can respond to events from other molecular clients using callbacks (see: [simple event example](https://github.com/primcloud/moleculer-rs/blob/master/examples/simple_event.rs))

Big missing pieces:

- Sending actions [#8](https://github.com/primcloud/moleculer-rs/issues/8)
- Receiving and responding to actions [#3](https://github.com/primcloud/moleculer-rs/issues/3)
- Documentation [#16](https://github.com/primcloud/moleculer-rs/issues/16)
- Tests [#17](https://github.com/primcloud/moleculer-rs/issues/17)
