# Getting help with demodatagen

Thanks for using **demodatagen**! This page points you to the fastest way to get
unstuck. Please use the channel that best matches your need.

## Before you ask

Most questions are answered by the built-in help and reference output:

```bash
demodatagen --help            # global options and the command list
demodatagen <format> --help   # options for a specific format, e.g. `json --help`
demodatagen list              # every format, schema field type, preset, locale, and language
demodatagen presets           # the built-in schema presets and what each expands to
demodatagen info              # version, build, thread, and capability details
```

The [README](../README.md) covers installation, the schema syntax, locales, the
interface languages, and library usage in depth.

## Where to go

| I want to…                                  | Use this                                                                           |
| ------------------------------------------- | ---------------------------------------------------------------------------------- |
| Ask a usage question or share an idea       | [GitHub Discussions](https://github.com/josunlp/demodatagen/discussions)           |
| Report a reproducible bug                   | [Open a bug report](https://github.com/josunlp/demodatagen/issues/new/choose)      |
| Request a feature, format, or locale        | [Open a feature request](https://github.com/josunlp/demodatagen/issues/new/choose) |
| Report a security vulnerability (privately) | See [SECURITY.md](../SECURITY.md) — please do **not** open a public issue          |
| Contribute code or docs                     | See [CONTRIBUTING.md](../CONTRIBUTING.md)                                          |

## Writing a good question

The more of this you include, the faster we can help:

- The exact command you ran (including the `--schema`/`--preset`, `--locale`, and
  `--seed` you used).
- What you expected to happen and what actually happened.
- Your `demodatagen --version` and operating system.
- Any error output — re-running with `RUST_LOG=debug` or `--verbose` is especially
  helpful.

Please keep all interactions friendly and on-topic, in line with our
[Code of Conduct](../CODE_OF_CONDUCT.md).
