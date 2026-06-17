# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 3.x.x   | :white_check_mark: |
| < 3.0   | :x:                |

## Security Scanning

Heimdal uses automated security scanning tools:

- **cargo-audit**: Checks for known security vulnerabilities in dependencies
- **cargo-deny**: Validates dependency licenses and sources
- **Clippy**: Identifies potential code issues and unsafe patterns
- **Dependabot**: Automated dependency updates

## Reporting a Vulnerability

If you discover a security vulnerability in Heimdal, please report it by:

1. **DO NOT** open a public issue
2. Email the maintainers directly at: [security email - to be configured]
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 1 week
- **Fix Timeline**: Depends on severity
  - Critical: 1-2 weeks
  - High: 2-4 weeks
  - Medium: 4-8 weeks
  - Low: Next release cycle

## Security Best Practices

When using Heimdal:

1. **Secrets Management**: Use the built-in secrets management with OS keychain
2. **Repository Security**: Use SSH keys or personal access tokens for git operations
3. **File Permissions**: Review backup and dotfile permissions
4. **Dependencies**: Keep heimdal updated to the latest version
5. **Configuration**: Validate your `heimdal.yaml` before applying changes

## Disclosure Policy

When a security issue is fixed:

1. Patch will be released
2. Security advisory published on GitHub
3. CVE requested if applicable
4. Users notified via release notes

## Security Checklist for Contributors

- [ ] No hardcoded secrets or credentials
- [ ] Input validation on user-provided data
- [ ] Safe file operations (no symlink attacks)
- [ ] Proper error handling (no panic in production code)
- [ ] Dependencies reviewed and approved
- [ ] All clippy warnings resolved
- [ ] Security audit passes (`cargo audit`)
