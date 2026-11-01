import fs from 'fs';
import path from 'path';
import crypto from 'crypto';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '../../..');
const SCENARIOS_DIR = path.resolve(ROOT, 'tests/scenarios');
const RESULTS_DIR = path.resolve(ROOT, 'tests/results');

function sha256File(filePath) {
  if (!fs.existsSync(filePath)) return null;
  const buf = fs.readFileSync(filePath);
  return crypto.createHash('sha256').update(buf).digest('hex');
}

const scenarioFiles = fs.readdirSync(SCENARIOS_DIR).filter(f => f.endsWith('.json')).sort();
const auditResults = [];

for (const scFile of scenarioFiles) {
  const scPath = path.join(SCENARIOS_DIR, scFile);
  const scData = JSON.parse(fs.readFileSync(scPath, 'utf8'));

  const jarPath = path.resolve(ROOT, 'web', scData.jar_file);
  const diskJarSha = sha256File(jarPath);
  const stem = path.basename(scFile, '.json');
  const resultDir = path.join(RESULTS_DIR, scData.game_id, stem);
  const execJsonPath = path.join(resultDir, 'rust_execution.json');

  let execData = null;
  if (fs.existsSync(execJsonPath)) {
    execData = JSON.parse(fs.readFileSync(execJsonPath, 'utf8'));
  }

  // Check snapshots
  const snapshots = execData ? execData.snapshots : [];
  const snapshotHashes = snapshots.map(s => s.raw_sha256);
  const allSnapshotsIdentical = snapshotHashes.length > 1 && snapshotHashes.every(h => h === snapshotHashes[0]);
  const hasBlackOnlySnapshots = snapshots.some(s => s.non_black_pixels === 0);

  // Evaluate multi-dimensional verdicts
  const jarShaMatches = diskJarSha && scData.jar_sha256 ? (diskJarSha.toLowerCase() === scData.jar_sha256.toLowerCase()) : false;
  const executionSuccess = execData && execData.passed && !execData.criteria_report.criteria.host_crash;
  const lifecycleSuccess = execData && execData.criteria_report.criteria.midlet_initialized;
  const visualOutput = execData && execData.criteria_report.criteria.framebuffer_published && snapshots.some(s => s.non_black_pixels > 0);
  const visualEvolved = execData && execData.criteria_report.criteria.framebuffer_changed && !allSnapshotsIdentical;
  const inputDelivered = execData && execData.criteria_report.criteria.input_delivered;
  const inputFunctional = execData && execData.criteria_report.criteria.input_affected_game_state;
  const audioOutput = execData && execData.criteria_report.criteria.audio_generated;
  const persistenceOk = execData && execData.criteria_report.criteria.rms_write_read_ok;
  const gameplayProgress = executionSuccess && visualEvolved && (inputFunctional || scData.scenario_type === 'INIT');

  const auditEntry = {
    scenario_file: scFile,
    game_id: scData.game_id,
    scenario_type: scData.scenario_type,
    resolution: `${scData.resolution.width}x${scData.resolution.height}`,
    total_ticks: scData.total_ticks,
    input_events_in_scenario: scData.inputs.length,
    jar_file: scData.jar_file,
    jar_sha_expected: scData.jar_sha256,
    jar_sha_disk: diskJarSha,
    jar_sha_match: jarShaMatches,
    execution_report_exists: Boolean(execData),
    instructions_executed: execData ? execData.instructions_executed : 0,
    wall_time_ms: execData ? execData.wall_time_ms : 0,
    fb_publishes_count: execData ? execData.criteria_report.fb_publishes_count : 0,
    fb_changes_count: execData ? execData.criteria_report.fb_changes_count : 0,
    input_delivered: inputDelivered,
    input_affected_game_state: inputFunctional,
    snapshots_captured: snapshots.length,
    all_snapshots_identical: allSnapshotsIdentical,
    has_black_only_snapshots: hasBlackOnlySnapshots,
    previous_verdict: execData ? execData.verdict : 'MISSING',
    dimensions: {
      EXECUTION_SUCCESS: executionSuccess ? 'PASS' : 'FAIL',
      LIFECYCLE_SUCCESS: lifecycleSuccess ? 'PASS' : 'FAIL',
      VISUAL_OUTPUT: visualOutput ? 'PASS' : 'FAIL',
      VISUAL_EVOLVED: visualEvolved ? 'PASS' : 'FAIL',
      INPUT_FUNCTIONAL: scData.inputs.length === 0 ? 'N/A' : (inputFunctional ? 'PASS' : 'NOT_OBSERVED'),
      AUDIO_OUTPUT: audioOutput ? 'PASS' : 'SILENT',
      PERSISTENCE: persistenceOk ? 'PASS' : 'NOT_ACCESSED',
      GAMEPLAY_PROGRESS: gameplayProgress ? 'PASS' : (scData.scenario_type === 'INIT' ? 'N/A' : 'NOT_PROVEN')
    }
  };

  auditResults.push(auditEntry);
}

fs.writeFileSync(
  path.resolve(__dirname, 'scenario_audit.json'),
  JSON.stringify({ audit_timestamp: new Date().toISOString(), total_scenarios: auditResults.length, scenarios: auditResults }, null, 2)
);

console.log(`Audited ${auditResults.length} scenarios. Summary saved to scenario_audit.json`);
