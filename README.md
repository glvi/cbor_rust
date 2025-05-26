# cbor_rust

[![Rust](https://github.com/glvi/cbor_rust/actions/workflows/cbor_rust.yml/badge.svg)](https://github.com/glvi/cbor_rust/actions/workflows/cbor_rust.yml)

CBOR decoder in Rust

I started this project for educational purposes. Primarily, I wanted to educate myself in the use of Rust. I was dealing a lot with CBOR at the time. So that seemed like an obvious choice.

Starting point is a simple CBOR decoder for use with a serial line. The CBOR decoder consumes a byte at a time, and at the same time builds an in-memory structured CBOR representation.

More use cases may follow…
