#!/usr/bin/env node
const { unlinkSync } = require('node:fs');
const { join, dirname } = require('node:path');

const baseDir = dirname(__filename);
const targets = ['dist', 'dist-node', 'dist-web'];

for (const dir of targets) {
  const gitignorePath = join(baseDir, '..', dir, '.gitignore');
  try {
    unlinkSync(gitignorePath);
  } catch (error) {
    if (error.code !== 'ENOENT') {
      throw error;
    }
  }
}
