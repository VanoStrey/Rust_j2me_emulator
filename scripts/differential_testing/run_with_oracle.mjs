/**
 * Differential Testing with Live FreeJ2ME Oracle (Milestone 2)
 * Runs target scenarios on both Rust and FreeJ2ME under headless Chrome CDP,
 * extracts exact reference frames, computes all 5 metrics, and produces visual diffs.
 */

import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';
import { FreeJ2meHeadlessRunner } from './reference_runner.mjs';
import { runDifferentialScenario } from './engine.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

async function main() {
  const runner = new FreeJ2meHeadlessRunner(8940, 9280);
  console.log('[Oracle Launcher] Starting FreeJ2ME Headless Chrome environment...');
  await runner.start();
  console.log('[Oracle Launcher] FreeJ2ME Headless Chrome connected.');

  const targetScenarios = [
    'doom_rpg_init.json',
    'bounce_tales_init.json',
    'castlevania_init.json',
    'gravity_defied_init.json',
    'prince_sands_init.json'
  ];

  for (const scFile of targetScenarios) {
    try {
      console.log(`\n>>> Running Differential Testing vs Live FreeJ2ME Oracle: ${scFile}`);
      const rep = await runDifferentialScenario(scFile, runner);
      console.log(`[Oracle Result] Scenario ${scFile} completed.`);
    } catch (e) {
      console.error(`[Oracle Error on ${scFile}]`, e);
    }
  }

  await runner.close();
  console.log('\n[Oracle Launcher] FreeJ2ME test suite finished and browser closed.');
}

main().catch((e) => {
  console.error('[Fatal Error]', e);
  process.exit(1);
});
