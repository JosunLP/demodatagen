# Security Policy

We take the security of **demodatagen** seriously. This document describes which versions receive
security fixes, how to report a vulnerability privately, what response times to expect, and the
security-relevant scope of the tool.

## Supported Versions

Security updates are provided for the following release series:

| Version | Supported          |
| ------- | ------------------ |
| 0.5.x   | :white_check_mark: |
| < 0.5   | :x:                |

Older release series no longer receive security updates. Please upgrade to the latest `0.5.x`
release to stay supported.

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Report vulnerabilities through either of these private channels:

1. **GitHub Security Advisories (preferred).** Use the repository's
   [private vulnerability reporting](https://github.com/josunlp/demodatagen/security/advisories/new)
   feature to open a private advisory. This keeps the report confidential while we investigate.
2. **Email.** Send the details to <support@josunlp.de>.

When reporting, please include as much of the following as you can:

- A description of the vulnerability and its potential impact.
- The affected version(s) and platform(s).
- Step-by-step instructions to reproduce the issue, including any sample schema, command-line
  invocation, or input file.
- Any proof-of-concept code, logs, or stack traces.

## Response Time Expectations

We aim to meet the following timelines for reports received through the channels above:

- **Acknowledgement:** within **3 business days** of receiving your report.
- **Initial assessment:** a triage and severity assessment within **7 business days**.
- **Status updates:** at least once every **2 weeks** while the issue is being worked on.
- **Fix and disclosure:** for confirmed vulnerabilities, we will work to release a fix in a
  supported `0.4.x` version as quickly as is practical, and will coordinate public disclosure
  (including a GitHub Security Advisory and credit to the reporter, if desired) once a fix is
  available.

If you do not receive an acknowledgement within the stated window, please follow up via email to
<support@josunlp.de>.

## Scope

demodatagen is designed to be safe by construction, which narrows its security surface
considerably:

- **Synthetic, offline data generation.** The tool generates *synthetic* demo/test data locally.
  It does not transmit, collect, or phone home with generated data or usage information.
- **No network calls during generation.** Core generation runs fully offline. There are **no**
  network calls in the normal data-generation path.
- **Single exception — opt-in self-update.** The only network activity is the **opt-in**
  self-update feature, which downloads release artifacts from this project's GitHub Releases over
  TLS (using `rustls`). Self-update only runs when the user explicitly invokes it; it is never
  triggered automatically.

Because generated data is synthetic and produced offline, the most relevant security concerns are
things like input handling (e.g., crafted schema or input files that could cause panics, excessive
resource use, or unsafe file writes) and the integrity of the self-update path. Reports in these
areas are especially welcome.

Thank you for helping keep demodatagen and its users safe.
