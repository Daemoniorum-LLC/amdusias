# Security Policy

## Supported Versions

| Component | Version | Supported          |
| --------- | ------- | ------------------ |
| Amdusias  | 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please report it responsibly.

### How to Report

**DO NOT** create a public GitHub issue for security vulnerabilities.

Instead, please email security vulnerabilities to:
- **Email:** security@daemoniorum.com

### What to Include

Please include the following in your report:

1. **Description** - A clear description of the vulnerability
2. **Impact** - What could an attacker accomplish?
3. **Reproduction Steps** - Step-by-step instructions to reproduce the issue
4. **Affected Component** - Which crate(s) are affected
5. **Version** - What version(s) are affected
6. **Suggested Fix** - If you have one (optional)

### Response Timeline

- **Initial Response:** Within 48 hours
- **Triage Complete:** Within 7 days
- **Fix Timeline:** Depends on severity
  - Critical: 7 days
  - High: 14 days
  - Medium: 30 days
  - Low: 60 days

### What to Expect

1. **Acknowledgment** - We'll confirm receipt of your report
2. **Investigation** - We'll investigate and determine the impact
3. **Fix Development** - We'll develop and test a fix
4. **Coordinated Disclosure** - We'll work with you on disclosure timing
5. **Credit** - We'll credit you in the security advisory (if desired)

## Security Best Practices

### For Audio Processing

- Validate sample rates and buffer sizes before processing
- Use bounded buffers to prevent memory exhaustion
- Sanitize user-provided parameters (frequencies, gains, etc.)
- Avoid unbounded loops in real-time audio paths

### For WebAssembly

- Validate all data crossing the WASM boundary
- Use SharedArrayBuffer only with proper CORS headers
- Limit memory allocation in AudioWorklet context

### For Native Platforms

- Use exclusive audio mode only when needed
- Properly release audio device handles
- Validate plugin/extension integrity before loading

## Known Security Considerations

### Real-time Audio

- **Denial of Service:** Malicious audio graphs could exhaust CPU. Implement processing budgets.
- **Memory Safety:** Lock-free data structures require careful implementation to avoid data races.

### WebAssembly

- **Cross-Origin Isolation:** SharedArrayBuffer requires specific headers
- **Memory Limits:** WASM memory can be exhausted by malicious input

## Dependency Security

We regularly audit dependencies for known vulnerabilities using:
- `cargo audit`
- `cargo deny`
- Dependabot alerts

## Contact

For non-vulnerability security questions, you can reach us at:
- **General:** hello@daemoniorum.com
- **Security:** security@daemoniorum.com
