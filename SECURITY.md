# Security Policy

## Supported versions

mdpad is pre-1.0; only the latest release receives fixes. Update to the
[latest release](https://github.com/lpalbou/mdpad/releases/latest) before
reporting.

## Reporting a vulnerability

Please report vulnerabilities privately — do not open a public issue.

- Email: <contact@abstractframework.ai>
- Or use GitHub's private reporting:
  [Security Advisories](https://github.com/lpalbou/mdpad/security/advisories/new)

Include the version (`mdpad --version`), your OS and terminal, and a
reproducing input if possible. You can expect an acknowledgement within a
few days; fixes ship as a new release with credit in the changelog if you
want it.

## Scope

mdpad regularly renders untrusted markdown (piped from the network), so the
parser and renderer are hardened against pathological inputs: deep nesting
is depth-capped, malformed markup must render rather than crash, and the
integration suite includes adversarial fixtures. Reports about crashes,
hangs, resource exhaustion or terminal-state corruption triggered by a
document are all in scope.

mdpad writes files only when you explicitly save in the editor (atomic
temp-file + rename), and its clipboard writes contain only text you
selected. It makes no network connections.
