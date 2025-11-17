# Release and Publishing Documentation

This document contains maintainer-specific information about publishing releases and artifacts to various package repositories.

## Publishing to Maven Central (Java)

The Java bindings are published to Maven Central under `io.github.goldziher:html-to-markdown`.

### Prerequisites

1. **Maven Central Account** at [central.sonatype.com](https://central.sonatype.com)
   - Register an account
   - Verify ownership of the `io.github.goldziher` namespace

2. **GitHub Secrets** (already configured):
   - `MAVEN_USERNAME`: Central Portal username
   - `MAVEN_PASSWORD`: Central Portal password/token

3. **GPG Keys** (TODO - needs manual setup):
   ```bash
   # Generate GPG key
   gpg --full-generate-key
   # Choose: RSA and RSA, 4096 bits, no expiration
   # Use your GitHub email

   # List keys to get KEY_ID
   gpg --list-secret-keys --keyid-format=long

   # Export private key (store as GPG_PRIVATE_KEY secret)
   gpg --armor --export-secret-keys <KEY_ID>

   # Export public key and upload to key server
   gpg --keyserver keyserver.ubuntu.com --send-keys <KEY_ID>
   gpg --keyserver keys.openpgp.org --send-keys <KEY_ID>

   # Store GPG_PASSPHRASE as secret
   ```

4. **GitHub Secrets to Configure**:
   - `GPG_PRIVATE_KEY`: The armored private key from above
   - `GPG_PASSPHRASE`: The passphrase for the GPG key

### Publishing Workflow

The `java-jars` job in `.github/workflows/publish.yaml` builds platform-specific JARs:

- `html-to-markdown-{version}-linux.jar` (Ubuntu x86_64)
- `html-to-markdown-{version}-macos.jar` (macOS universal)
- `html-to-markdown-{version}-windows.jar` (Windows x86_64)

Plus sources and javadoc JARs for Maven Central requirements:
- `html-to-markdown-{version}-sources.jar`
- `html-to-markdown-{version}-javadoc.jar`

### Current Status

The workflow currently builds and uploads JARs as GitHub artifacts but **does not yet deploy to Maven Central**. This requires:

1. **GPG Keys**: Generate and configure as GitHub secrets (see above)
2. **Remove `-Dgpg.skip=true`**: Update the Maven build command to enable signing
3. **Add deployment job**: Create a job that:
   - Downloads all platform JARs from artifacts
   - Signs them with GPG
   - Uses `./mvnw -f packages/java/pom.xml deploy` with the `central-publishing-maven-plugin`
   - Tests with `dry_run: true` first

### Maven Central Publishing Plugin

We use the modern **Central Publishing Plugin** (2025+) instead of the legacy OSSRH/Nexus Staging plugin.

Configuration in `packages/java/pom.xml`:
```xml
<plugin>
    <groupId>org.sonatype.central</groupId>
    <artifactId>central-publishing-maven-plugin</artifactId>
    <version>0.6.0</version>
    <configuration>
        <publishingServerId>central</publishingServerId>
        <autoPublish>true</autoPublish>
        <waitUntil>published</waitUntil>
        <checksums>all</checksums>
    </configuration>
</plugin>
```

The `publishingServerId` references the `<server>` entry in Maven's `settings.xml` (configured via environment variables in CI).

### Testing the Publishing Workflow

Before doing a real release:

1. Generate and configure GPG keys
2. Update workflow to enable signing
3. Add deployment job with `dry_run: true`
4. Trigger a test release
5. Verify artifacts in Maven Central Portal
6. Once confirmed, set `dry_run: false` for production releases

## Other Languages

### Python (PyPI)
Published via Maturin in `.github/workflows/publish.yaml`

### PHP (Packagist + PIE)
- Composer package: Published via Git tags (auto-detected by Packagist)
- PIE extension: Published to PHP Extension Installer

### Ruby (RubyGems)
Published via Rake tasks in `.github/workflows/publish.yaml`

### JavaScript/TypeScript (npm)
- Node bindings: Published to npm as `html-to-markdown-node`
- WASM bindings: Published to npm as `html-to-markdown-wasm`
- TypeScript wrapper: Published to npm as `html-to-markdown`

### Rust (crates.io)
Published via `cargo publish` in `.github/workflows/publish.yaml`

## Release Process

1. Update version numbers in all relevant files
2. Update CHANGELOG.md
3. Create and push a Git tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
4. GitHub Actions will automatically:
   - Build artifacts for all platforms
   - Run smoke tests
   - Publish to package repositories
5. Monitor the `publish.yaml` workflow for any failures
6. Verify artifacts appear in all package repositories

## Troubleshooting

### Java Publishing Issues

**GPG Signing Fails**
- Ensure `GPG_PRIVATE_KEY` and `GPG_PASSPHRASE` secrets are set
- Verify the key hasn't expired: `gpg --list-keys`
- Check the workflow logs for pinentry errors

**Maven Central Rejects Artifact**
- Ensure all required metadata is present (name, description, URL, licenses, developers, SCM)
- Verify POM passes validation: `./mvnw -f packages/java/pom.xml validate`
- Check that sources and javadoc JARs are included

**Platform-Specific JAR Missing**
- Check that the matrix build completed for all platforms
- Verify native library was built: `ls target/release/`
- Ensure library was copied to `src/main/resources/`

## References

- [Maven Central Portal Publishing Guide](https://central.sonatype.org/publish/publish-portal-upload/)
- [Central Publishing Maven Plugin](https://central.sonatype.org/publish/publish-portal-maven/)
- [GPG Key Generation Guide](https://central.sonatype.org/publish/requirements/gpg/)
