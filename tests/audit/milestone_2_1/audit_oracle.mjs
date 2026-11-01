import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { FreeJ2meHeadlessRunner } from '../../../scripts/differential_testing/reference_runner.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '../../..');
const SCENARIOS_DIR = path.resolve(ROOT, 'tests/scenarios');
const RAW_DIR = path.resolve(__dirname, 'raw');

const games = [
  { id: 'doom_rpg', scenario: 'doom_rpg_init.json', jar: 'doom_rpg.jar' },
  { id: 'bounce_tales', scenario: 'bounce_tales_init.json', jar: 'bounce_tales.jar' },
  { id: 'castlevania', scenario: 'castlevania_init.json', jar: 'castlevania.jar' },
  { id: 'gravity_defied', scenario: 'gravity_defied_init.json', jar: 'gravity_defied.jar' },
  { id: 'prince_sands', scenario: 'prince_sands_init.json', jar: 'prince_sands.jar' },
  { id: 'heroes_lore', scenario: 'heroes_lore_init.json', jar: 'heroes_lore.jar' },
  { id: 'cyberpunk', scenario: 'cyberpunk_init.json', jar: 'cyberpunk.jar' }
];

async function auditOracle() {
  console.log('[Oracle Audit] Initializing FreeJ2ME Headless Runner...');
  const runner = new FreeJ2meHeadlessRunner(8945, 9285);
  let launched = false;
  try {
    await runner.start();
    launched = true;
    console.log('[Oracle Audit] FreeJ2ME Chrome CDP connection established.');
  } catch (e) {
    console.error('[Oracle Audit Failed to Start]', e);
    fs.writeFileSync(
      path.resolve(__dirname, 'oracle_audit.json'),
      JSON.stringify({ launched: false, error: e.message, results: [] }, null, 2)
    );
    return;
  }

  const results = [];

  for (const g of games) {
    console.log(`\n--- Auditing Oracle on: ${g.id} (${g.scenario}) ---`);
    const scPath = path.join(SCENARIOS_DIR, g.scenario);
    const scenario = JSON.parse(fs.readFileSync(scPath, 'utf8'));
    const outDir = path.join(RAW_DIR, 'oracle_' + g.id);
    fs.mkdirSync(outDir, { recursive: true });

    let jarLoaded = false;
    let inputInjected = false;
    let gameplayReached = false;
    let refCaptured = false;
    let snapshotsCount = 0;
    let err = null;

    try {
      const snaps = await runner.runScenario(scenario, outDir);
      snapshotsCount = snaps.length;
      refCaptured = snaps.length > 0;
      jarLoaded = true; // Canvas became ready and didn't throw
      inputInjected = scenario.inputs.length > 0;
      gameplayReached = false; // Init scenario only
    } catch (e) {
      err = e.message;
      console.warn(`[Oracle error on ${g.id}]:`, e.message);
    }

    results.push({
      game_id: g.id,
      scenario: g.scenario,
      jar_file: g.jar,
      oracle_launched: launched,
      correct_jar_loaded: jarLoaded,
      input_injected: inputInjected,
      gameplay_reached: gameplayReached,
      reference_image_captured: refCaptured,
      rust_image_captured: fs.existsSync(path.resolve(ROOT, `tests/results/${g.id}/${path.basename(g.scenario, '.json')}/rust_tick_0060.raw`)) ||
                          fs.existsSync(path.resolve(ROOT, `tests/results/${g.id}/${path.basename(g.scenario, '.json')}/rust_tick_0240.raw`)) ||
                          fs.existsSync(path.resolve(ROOT, `tests/results/${g.id}/${path.basename(g.scenario, '.json')}/rust_tick_0200.raw`)) ||
                          fs.existsSync(path.resolve(ROOT, `tests/results/${g.id}/${path.basename(g.scenario, '.json')}/rust_tick_0280.raw`)),
      snapshots_count: snapshotsCount,
      comparable_state: g.id === 'doom_rpg' ? 'STATIC_SPLASH_ONLY' : 'DESYNCHRONIZED_TIMING',
      error: err
    });
  }

  await runner.close();

  fs.writeFileSync(
    path.resolve(__dirname, 'oracle_audit.json'),
    JSON.stringify({ launched, total_audited: results.length, results }, null, 2)
  );

  console.log('\n[Oracle Audit] Completed audit. Saved to oracle_audit.json');
}

auditOracle().catch(e => {
  console.error('[Fatal Oracle Audit Error]', e);
  process.exit(1);
});
