# Activating GitHub Actions

The repository keeps the CI and release workflow definitions in this directory because the authenticated publishing identity does not have permission to create or update `.github/workflows/*` through GitHub's API.

After cloning the repository with an account or token that has the `workflow` permission, copy the files into the active GitHub Actions directory and commit them:

```bash
mkdir -p .github/workflows
cp workflow-templates/*.yml .github/workflows/
git add .github/workflows
git commit -m "ci: activate validation and release workflows"
git push
```

`ci.yml` runs Rust tests and validates the Node.js plugin host. `release.yml` builds Linux x86_64, macOS x86_64, macOS ARM64, and Windows x86_64 archives when a version tag is pushed.
