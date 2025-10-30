#!/bin/bash


BRANCH="main"
REMOTE="origin"
COMMIT_MSG="$1"

if [ -z "$COMMIT_MSG" ]; then
  echo "Usage: ./deploy.sh \"Deployment\""
  exit 1
fi

echo "Starting Git deployment..."
echo "Current branch: $BRANCH"
echo "Remote: $REMOTE"

# Add all changes
git add .

# Commit changes
git commit -m "$COMMIT_MSG"

# Pull latest from remote to avoid conflicts
git pull $REMOTE $BRANCH --rebase

# Push to remote
git push $REMOTE $BRANCH

echo "Deployment complete!"
