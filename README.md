# CKB Genesis Block Generator

[![version](https://img.shields.io/github/v/release/nervosnetwork/genesis-block-generator)](https://github.com/nervosnetwork/genesis-block-generator/releases/latest)

![256px-Lina_Inverse_GF](https://user-images.githubusercontent.com/3198439/67932760-79c9a080-fbff-11e9-8b59-fa44e825d45d.png)

## Usage


```shell
CKB Genesis Block Generator

USAGE:
    ckb-gbg [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --target <TARGET>    target epoch number
    -u, --url <URL>          ckb node rpc endpoint
```

This is an implementation following the [Genesis Block Generator Specification](spec.md).

Embedded CSV files are in [src/input](src/input).

## Launch Process

- Run a v0.24.0 node connecting to testnet.
- Run `ckb-gbg` to generate the chain spec for mainnet.
- Stop the v0.24.0 node.
- Use v0.25.0 binary to init and start the mainnet node using the generated chain spec:

```
ckb init --import-spec /path/to/generated/lina.toml --chain mainnet
ckb run
```

## License

Licensed under [MIT License]

[MIT License]: LICENSE-MIT

## References
[Granblue Fantasy]: [NEWS - グランブルーファンタジー](https://granbluefantasy.jp/pages/?p=6029)
