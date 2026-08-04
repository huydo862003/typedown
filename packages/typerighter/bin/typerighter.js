#!/usr/bin/env -S node --no-warnings

import('../dist/cli.js').then(({
  cli,
}) => cli());
