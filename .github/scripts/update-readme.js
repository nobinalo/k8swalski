#!/usr/bin/env node
// Script to update README.md with CLI help output
const { execSync } = require('node:child_process');
const fs = require('node:fs/promises');

const README_FILE = 'README.md';
const BEGIN_MARKER = '<!-- BEGIN_CLI_HELP -->';
const END_MARKER = '<!-- END_CLI_HELP -->';

async function main() {
    try {
        // Generate help output from the binary
        const helpOutput = execSync('cargo run --quiet -- --help', {
            encoding: 'utf8',
            stdio: ['pipe', 'pipe', 'pipe']
        }).trim();

        if (!helpOutput) {
            console.error('Error: Failed to generate help output');
            process.exit(1);
        }

        // Read current README content
        const readmeContent = await fs.readFile(README_FILE, 'utf8');

        // Find marker positions
        const beginIndex = readmeContent.indexOf(BEGIN_MARKER);
        const endIndex = readmeContent.indexOf(END_MARKER);

        if (beginIndex === -1 || endIndex === -1) {
            console.error('Error: Markers not found in README.md');
            process.exit(1);
        }

        // Construct new README content
        const beforeMarker = readmeContent.substring(0, beginIndex + BEGIN_MARKER.length);
        const afterMarker = readmeContent.substring(endIndex);
        
        const block = (content) => '```\n' + content + '\n```';
        
        const updatedContent = beforeMarker + '\n' + block(helpOutput) + '\n' + afterMarker;

        // Write updated content back to README
        await fs.writeFile(README_FILE, updatedContent, 'utf8');

        console.log('README.md updated successfully');
    } catch (error) {
        console.error('Error:', error.message);
        process.exit(1);
    }
}

main();
