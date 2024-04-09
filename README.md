# kotori

Kotori is a multiplatform manga reader, aimed to be a simple and performant.

> This is still in a very early stage of development. New features will be added over time.

## Development

Kotori is built with [Tauri](https://beta.tauri.app/guides/), with most of its logic implemented in the Rust backend. The front-end uses [Vue](https://vuejs.org/guide/introduction.html), an intuitive TypeScript framework.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.76 or higher)
- [Node](https://nodejs.org) (version 20 or higher)
- [pnpm](https://pnpm.io/) (version 8 or higher)

### Setup

```bash
cargo install tauri-cli --version "^2.0.0-beta"
pnpm install
pnpm run dev
```
