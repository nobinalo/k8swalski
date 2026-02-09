// GitHub Action Script to replace content between markers in a file
const fs = require('node:fs/promises');

module.exports = async ({ content, filePath = 'README.md', beginMarker, endMarker }) => {
    if (!content) {
        throw new Error('Content is required');
    }
    
    if (!beginMarker || !endMarker) {
        throw new Error('Begin and end markers are required');
    }

    // Read current file content
    const fileContent = await fs.readFile(filePath, 'utf8');

    // Find marker positions
    const beginIndex = fileContent.indexOf(beginMarker);
    const endIndex = fileContent.indexOf(endMarker);

    if (beginIndex === -1 || endIndex === -1) {
        throw new Error(`Markers not found in ${filePath}`);
    }

    // Construct new file content
    const beforeMarker = fileContent.substring(0, beginIndex + beginMarker.length);
    const afterMarker = fileContent.substring(endIndex);
    
    const block = (text, lang = '') => '```' + lang + '\n' + text + '\n```';
    
    const updatedContent = beforeMarker + '\n' + block(content) + '\n' + afterMarker;

    // Write updated content back to file
    await fs.writeFile(filePath, updatedContent, 'utf8');

    console.log(`${filePath} updated successfully`);
};
