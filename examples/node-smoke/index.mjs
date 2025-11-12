import { convert } from 'html-to-markdown-node';

const html = `
  <h1>Node Smoke Test</h1>
  <p>This script renders markdown via the native NAPI bindings.</p>
  <ul><li>Buffers</li><li>Options</li></ul>
`;

const markdown = convert(html, {
  headingStyle: 'Atx',
  bullets: '-',
});

if (!markdown.includes('# Node Smoke Test')) {
  console.error(markdown);
  throw new Error('html-to-markdown-node did not return the expected heading');
}

console.log('\u001b[32m✓ html-to-markdown-node produced markdown output\u001b[0m');
console.log('---');
console.log(markdown);
