# GitHub Pages Setup Instructions

## Issue
The GitHub Pages site at https://carlotta94c.github.io/aitour26-WRK541-real-world-code-migration-with-github-copilot-agent-mode/ is returning a 404 error.

## Root Cause Analysis

After investigation, the following was discovered:

1. **MkDocs Configuration**: The `site_url` parameter was missing from `mkdocs.yml`, which has now been added.
2. **gh-pages Branch**: A `gh-pages` branch exists with properly generated HTML files including `index.html`.
3. **GitHub Actions**: A CI workflow exists that automatically deploys to gh-pages on pushes to `main`.

## Potential Causes of 404 Error

The 404 error could be caused by one of the following:

### 1. GitHub Pages Not Enabled
GitHub Pages might not be enabled in the repository settings.

**Solution**: 
- Go to repository Settings → Pages
- Under "Source", select "Deploy from a branch"
- Select the `gh-pages` branch and `/ (root)` folder
- Click "Save"

### 2. GitHub Pages Source Branch Misconfigured
GitHub Pages might be configured to deploy from the wrong branch or folder.

**Solution**:
- Verify in Settings → Pages that the source is set to `gh-pages` branch and `/ (root)` folder

### 3. Recent Changes Not Deployed
The `site_url` fix needs to be merged to `main` and deployed.

**Solution**:
- Merge this PR to the `main` branch
- The GitHub Actions workflow will automatically run `mkdocs gh-deploy`
- Wait for the deployment to complete (usually takes a few minutes)

### 4. Cache or Propagation Delay
GitHub Pages might be caching an old version or still propagating changes.

**Solution**:
- Wait 10-15 minutes after deployment
- Clear your browser cache
- Try accessing the site in an incognito/private window

## Changes Made

Added the following line to `mkdocs.yml`:
```yaml
site_url: https://carlotta94c.github.io/aitour26-WRK541-real-world-code-migration-with-github-copilot-agent-mode/
```

This ensures that MkDocs generates correct absolute URLs for all pages, which is important for the i18n plugin to work correctly with GitHub Pages.

## Verification Steps

After merging this PR and deploying:

1. Check that GitHub Pages is enabled: Go to Settings → Pages
2. Verify the deployment status: Check the "Actions" tab for the latest workflow run
3. Test the site: Visit https://carlotta94c.github.io/aitour26-WRK541-real-world-code-migration-with-github-copilot-agent-mode/
4. Check that all pages load correctly (not just the homepage)

## Manual Deployment (if needed)

If automatic deployment doesn't work, you can manually deploy:

```bash
# Install dependencies
pip install -r requirements.txt

# Deploy to gh-pages
mkdocs gh-deploy --force
```

**Note**: Manual deployment requires push permissions to the repository.
