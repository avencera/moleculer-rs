# moleculer-rs — work in progress

Inspired and compatible with [Moleculer JS](https://github.com/moleculerjs/moleculer)

You can currently do all the basics of `emit`, `broadcast` and `call`.

However it only works with the `NATS` transporter and `JSON` serializer/deserializer.

What it does:

- Is discoverable by other moleculer clients
- Only NATS transporter
- Only JSON serialization/deserialization
- Can `emit` and `broadcast` events
- Can respond to events from other molecular clients using callbacks (see: [simple event example](https://github.com/primcloud/moleculer-rs/blob/master/examples/simple_event.rs))
- Can create actions, and respond to requests ([#19](https://github.com/primcloud/moleculer-rs/pull/19))
- Can `call` to send request and wait for response ([#20](https://github.com/primcloud/moleculer-rs/pull/20))

Big missing pieces:

- Documentation [#16](https://github.com/primcloud/moleculer-rs/issues/16)
- Tests [#17](https://github.com/primcloud/moleculer-rs/issues/17)
