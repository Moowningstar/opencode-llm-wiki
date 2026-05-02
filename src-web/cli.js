#!/usr/bin/env node

import { program } from 'commander';
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';
import { spawn } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const VERSION = '1.0.0';

program
  .name('llm-wiki')
  .description('LLM Wiki - Self-building personal knowledge base')
  .version(VERSION);

program
  .command('init [directory]')
  .description('Initialize a new LLM Wiki project')
  .option('-t, --template <type>', 'Project template (research|reading|personal|business|general)', 'general')
  .action(async (directory, options) => {
    try {
      const targetDir = directory || process.cwd();
      await initProject(targetDir, options.template);
      console.log(`✓ Project initialized at: ${path.resolve(targetDir)}`);
      console.log(`\nNext steps:`);
      console.log(`  1. ${directory ? `cd ${directory}` : 'Already in project directory'}`);
      console.log(`  2. Add documents to .raw/ directory`);
      console.log(`  3. llm-wiki serve`);
    } catch (error) {
      console.error(`✗ Failed to initialize project: ${error.message}`);
      process.exit(1);
    }
  });

program
  .command('serve [directory]')
  .description('Start MCP server for a project directory')
  .option('-p, --port <port>', 'Port number (for future HTTP mode)', '19828')
  .action(async (directory, options) => {
    const projectPath = path.resolve(directory || process.cwd());
    
    try {
      await fs.access(projectPath);
    } catch {
      console.error(`✗ Directory not found: ${projectPath}`);
      console.error(`  Run 'llm-wiki init ${directory}' to create a new project`);
      process.exit(1);
    }

    console.log(`Starting LLM Wiki MCP server...`);
    console.log(`Project: ${projectPath}`);
    
    const serverPath = path.join(__dirname, 'server.js');
    const child = spawn('node', [serverPath], {
      env: {
        ...process.env,
        LLM_WIKI_PROJECT: projectPath
      },
      stdio: 'inherit'
    });

    child.on('error', (error) => {
      console.error(`✗ Failed to start server: ${error.message}`);
      process.exit(1);
    });

    child.on('exit', (code) => {
      if (code !== 0) {
        console.error(`✗ Server exited with code ${code}`);
        process.exit(code);
      }
    });

    process.on('SIGINT', () => {
      child.kill('SIGINT');
      process.exit(0);
    });

    process.on('SIGTERM', () => {
      child.kill('SIGTERM');
      process.exit(0);
    });
  });

program
  .command('scan [directory]')
  .description('Scan .raw/ directory and trigger ingest for new files')
  .action(async (directory) => {
    const projectPath = path.resolve(directory || process.cwd());
    console.log(`Scanning ${projectPath}/.raw/ for new files...`);
    console.log(`Note: This requires the desktop app or MCP server to be running.`);
    console.log(`Use 'llm-wiki serve ${directory || '.'}' to start the server.`);
  });

async function initProject(directory, template) {
  const projectPath = path.resolve(directory || process.cwd());
  
  try {
    await fs.access(projectPath);
    throw new Error(`Directory already exists: ${projectPath}`);
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }

  const dirs = [
    '.raw',
    'raw/sources',
    'raw/assets',
    '.wiki/entities',
    '.wiki/concepts',
    '.wiki/sources',
    '.wiki/queries',
    '.wiki/comparisons',
    '.wiki/synthesis',
    '.llm-wiki'
  ];

  for (const dir of dirs) {
    await fs.mkdir(path.join(projectPath, dir), { recursive: true });
  }

  const today = new Date().toISOString().split('T')[0];

  const schemaContent = `# Wiki Schema

## Page Types

| Type | Directory | Purpose |
|------|-----------|---------|
    | entity | .wiki/entities/ | Named things (models, companies, people, datasets) |
    | concept | .wiki/concepts/ | Ideas, techniques, phenomena |
    | source | .wiki/sources/ | Papers, articles, talks, blog posts |
    | query | .wiki/queries/ | Open questions under investigation |
    | comparison | .wiki/comparisons/ | Side-by-side analysis of related entities |
    | synthesis | .wiki/synthesis/ | Cross-cutting summaries and conclusions |

## Naming Conventions

- Files: \`kebab-case.md\`
- Entities: match official name where possible (e.g., \`gpt-4.md\`, \`openai.md\`)
- Concepts: descriptive noun phrases (e.g., \`chain-of-thought.md\`)
- Sources: \`author-year-slug.md\` (e.g., \`wei-2022-chain-of-thought.md\`)
- Queries: question as slug (e.g., \`does-scale-improve-reasoning.md\`)

## Frontmatter

All pages must include YAML frontmatter:

\`\`\`yaml
---
type: entity | concept | source | query | comparison | synthesis | overview
title: Human-readable title
tags: []
related: []
created: YYYY-MM-DD
updated: YYYY-MM-DD
---
\`\`\`

Source pages also include:
\`\`\`yaml
authors: []
year: YYYY
url: ""
venue: ""
\`\`\`
`;

  const purposeTemplates = {
    research: `# Purpose

## Goals
- Build a comprehensive knowledge base on [research topic]
- Track key papers, concepts, and methodologies
- Identify research gaps and opportunities

## Key Questions
- What are the fundamental concepts in this field?
- What are the current state-of-the-art approaches?
- What are the open problems and future directions?

## Scope
- Focus: [specific research area]
- Time period: [date range]
- Sources: Academic papers, conference proceedings, technical blogs
`,
    reading: `# Purpose

## Goals
- Organize insights from books, articles, and papers
- Build connections between ideas across different sources
- Create a personal reference library

## Key Questions
- What are the main ideas from each source?
- How do different authors' perspectives relate?
- What practical applications can I derive?

## Scope
- Topics: [your interests]
- Sources: Books, articles, essays, podcasts
`,
    personal: `# Purpose

## Goals
- Track personal growth and learning journey
- Document skills, experiences, and reflections
- Build a knowledge base for self-improvement

## Key Questions
- What have I learned recently?
- What skills am I developing?
- What patterns do I notice in my growth?

## Scope
- Focus: Personal development, skills, experiences
- Sources: Notes, reflections, courses, projects
`,
    business: `# Purpose

## Goals
- Organize business knowledge and insights
- Track market trends and competitive intelligence
- Build strategic decision-making resources

## Key Questions
- What are the key market trends?
- Who are the major players and what are their strategies?
- What opportunities and threats exist?

## Scope
- Industry: [your industry]
- Focus: Strategy, operations, market analysis
- Sources: Reports, case studies, news, analysis
`,
    general: `# Purpose

## Goals
- Build a personal knowledge base
- Organize information and insights
- Create connections between ideas

## Key Questions
- What topics am I exploring?
- What patterns and connections exist?
- What questions am I investigating?

## Scope
- Open-ended knowledge collection
- Multiple topics and sources
- Evolving focus based on interests
`
  };

  const purposeContent = purposeTemplates[template] || purposeTemplates.general;

  const indexContent = `# Index

*This file is automatically maintained by the LLM. It serves as the content catalog and navigation entry point.*

## Entities

## Concepts

## Sources

## Queries

## Comparisons

## Synthesis
`;

  const logContent = `# Log

*This file records all operations performed on the wiki in reverse chronological order.*

## ${today}

- Project initialized
`;

  const overviewContent = `# Overview

*This file is automatically generated and updated by the LLM after each ingest.*

This wiki is currently empty. Add documents to the \`.raw/\` directory to begin building your knowledge base.
`;

  const projectConfig = {
    id: generateUUID(),
    name: path.basename(projectPath),
    created: new Date().toISOString(),
    template
  };

  await fs.writeFile(path.join(projectPath, 'schema.md'), schemaContent);
  await fs.writeFile(path.join(projectPath, 'purpose.md'), purposeContent);
  await fs.writeFile(path.join(projectPath, '.wiki', 'index.md'), indexContent);
  await fs.writeFile(path.join(projectPath, '.wiki', 'log.md'), logContent);
  await fs.writeFile(path.join(projectPath, '.wiki', 'overview.md'), overviewContent);
  await fs.writeFile(
    path.join(projectPath, '.llm-wiki', 'project.json'),
    JSON.stringify(projectConfig, null, 2)
  );

  const readmeContent = `# ${path.basename(projectPath)}

LLM Wiki project initialized on ${today}.

## Quick Start

1. Add documents to \`.raw/\` directory
2. Start the MCP server: \`llm-wiki serve .\`
3. Use OpenCode or other MCP clients to query your knowledge base

## Directory Structure

\`\`\`
${path.basename(projectPath)}/
├── .raw/              # Drop files here for auto-ingest
├── raw/
│   ├── sources/       # Imported source documents
│   └── assets/        # Images and media
├── .wiki/
│   ├── entities/      # Named things
│   ├── concepts/      # Ideas and techniques
│   ├── sources/       # Source summaries
│   ├── queries/       # Research questions
│   ├── comparisons/   # Side-by-side analysis
│   └── synthesis/     # Cross-cutting summaries
├── purpose.md         # Wiki goals and scope
├── schema.md          # Structure rules
└── .llm-wiki/         # Internal data
\`\`\`

## MCP Tools

- \`wiki_read\` - Read a wiki page
- \`wiki_list\` - List all pages
- \`wiki_search\` - Search by keyword
- \`wiki_query_with_context\` - Intelligent context retrieval
- \`wiki_get_graph\` - Get knowledge graph
- \`wiki_graph_insights\` - Analyze graph structure
- \`wiki_deep_research\` - Multi-hop reasoning

For more information, visit: https://github.com/nashsu/llm_wiki
`;

  await fs.writeFile(path.join(projectPath, 'README.md'), readmeContent);
}

program
  .command('project')
  .description('Manage sub-projects in a global wiki')
  .addCommand(
    program.createCommand('add')
      .argument('<name>', 'Project name')
      .option('-d, --description <desc>', 'Project description')
      .option('-t, --template <type>', 'Project template', 'general')
      .action(async (name, options) => {
        try {
          const globalRoot = process.cwd();
          await addProject(globalRoot, name, options);
          console.log(`✓ Project '${name}' added successfully`);
        } catch (error) {
          console.error(`✗ Failed to add project: ${error.message}`);
          process.exit(1);
        }
      })
  )
  .addCommand(
    program.createCommand('list')
      .description('List all projects')
      .action(async () => {
        try {
          const globalRoot = process.cwd();
          await listProjects(globalRoot);
        } catch (error) {
          console.error(`✗ Failed to list projects: ${error.message}`);
          process.exit(1);
        }
      })
  )
  .addCommand(
    program.createCommand('remove')
      .argument('<name>', 'Project name')
      .action(async (name) => {
        try {
          const globalRoot = process.cwd();
          await removeProject(globalRoot, name);
          console.log(`✓ Project '${name}' removed successfully`);
        } catch (error) {
          console.error(`✗ Failed to remove project: ${error.message}`);
          process.exit(1);
        }
      })
  );

async function addProject(globalRoot, name, options) {
  const projectsDir = path.join(globalRoot, 'projects');
  const projectPath = path.join(projectsDir, name);
  const configDir = path.join(globalRoot, '.llm-wiki');
  const projectsConfigPath = path.join(configDir, 'projects.json');

  try {
    await fs.access(projectPath);
    throw new Error(`Project '${name}' already exists`);
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }

  const dirs = [
    '.raw',
    '.wiki/entities',
    '.wiki/concepts',
    '.wiki/sources',
    '.wiki/queries',
    '.wiki/comparisons',
    '.wiki/synthesis',
    '.llm-wiki'
  ];

  for (const dir of dirs) {
    await fs.mkdir(path.join(projectPath, dir), { recursive: true });
  }

  const today = new Date().toISOString().split('T')[0];
  const projectConfig = {
    name,
    description: options.description || '',
    template: options.template,
    created: today,
    id: generateUUID()
  };

  await fs.writeFile(
    path.join(projectPath, '.llm-wiki', 'project.json'),
    JSON.stringify(projectConfig, null, 2)
  );

  let projectsConfig = { projects: [] };
  try {
    const data = await fs.readFile(projectsConfigPath, 'utf-8');
    projectsConfig = JSON.parse(data);
  } catch {}

  projectsConfig.projects.push(projectConfig);
  await fs.mkdir(configDir, { recursive: true });
  await fs.writeFile(projectsConfigPath, JSON.stringify(projectsConfig, null, 2));
}

async function listProjects(globalRoot) {
  const configDir = path.join(globalRoot, '.llm-wiki');
  const projectsConfigPath = path.join(configDir, 'projects.json');

  let projectsConfig = { projects: [] };
  try {
    const data = await fs.readFile(projectsConfigPath, 'utf-8');
    projectsConfig = JSON.parse(data);
  } catch {}

  if (projectsConfig.projects.length === 0) {
    console.log('No projects found.');
    return;
  }

  console.log('\nProjects:');
  for (const project of projectsConfig.projects) {
    console.log(`  • ${project.name}`);
    if (project.description) {
      console.log(`    ${project.description}`);
    }
    console.log(`    Created: ${project.created}`);
  }
}

async function removeProject(globalRoot, name) {
  const projectsDir = path.join(globalRoot, 'projects');
  const projectPath = path.join(projectsDir, name);
  const configDir = path.join(globalRoot, '.llm-wiki');
  const projectsConfigPath = path.join(configDir, 'projects.json');

  try {
    await fs.access(projectPath);
  } catch {
    throw new Error(`Project '${name}' not found`);
  }

  await fs.rm(projectPath, { recursive: true, force: true });

  let projectsConfig = { projects: [] };
  try {
    const data = await fs.readFile(projectsConfigPath, 'utf-8');
    projectsConfig = JSON.parse(data);
  } catch {}

  projectsConfig.projects = projectsConfig.projects.filter(p => p.name !== name);
  await fs.writeFile(projectsConfigPath, JSON.stringify(projectsConfig, null, 2));
}

function generateUUID() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

program.parse();
