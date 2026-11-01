/**
 * Scenario Generator for Differential Testing Framework (Milestone 2)
 * Creates 21 deterministic test scenarios: 7 canonical games x 3 scenarios (Init, Gameplay, Longrunning)
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SCENARIOS_DIR = path.resolve(__dirname, '../tests/scenarios');

if (!fs.existsSync(SCENARIOS_DIR)) {
  fs.mkdirSync(SCENARIOS_DIR, { recursive: true });
}

const CANONICAL_GAMES = [
  {
    id: 'bounce_tales',
    jar: 'bounce_tales.jar',
    sha256: 'f768bbd2366f6d739d170b8cfa8383a98c4830d03555f5b44a0ab88c61a8028e',
    width: 240,
    height: 320,
    init: {
      ticks: 240,
      snapshots: [60, 180, 240],
      inputs: [],
      desc: 'Cold boot through Rovio / Nokia splash screens to Title Screen.'
    },
    gameplay: {
      ticks: 450,
      snapshots: [120, 240, 360, 450],
      inputs: [
        { tick: 60, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 65, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 120, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 125, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 180, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 185, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 240, key: 54, action: 'press', free_key: 'Digit6' },
        { tick: 300, key: 53, action: 'press', free_key: 'Digit5' },
        { tick: 330, key: 53, action: 'release', free_key: 'Digit5' },
        { tick: 380, key: 54, action: 'release', free_key: 'Digit6' },
        { tick: 390, key: 52, action: 'press', free_key: 'Digit4' },
        { tick: 430, key: 52, action: 'release', free_key: 'Digit4' }
      ],
      desc: 'Start game, select Level 1, roll right (6), jump (5), and roll left (4).'
    },
    longrunning: {
      ticks: 1200,
      snapshots: [300, 600, 900, 1200],
      inputs: [
        { tick: 60, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 65, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 120, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 125, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 180, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 185, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 220, key: 54, action: 'press', free_key: 'Digit6' },
        { tick: 450, key: 54, action: 'release', free_key: 'Digit6' },
        { tick: 480, key: 52, action: 'press', free_key: 'Digit4' },
        { tick: 700, key: 52, action: 'release', free_key: 'Digit4' },
        { tick: 750, key: 53, action: 'press', free_key: 'Digit5' },
        { tick: 780, key: 53, action: 'release', free_key: 'Digit5' },
        { tick: 850, key: 54, action: 'press', free_key: 'Digit6' },
        { tick: 1100, key: 54, action: 'release', free_key: 'Digit6' }
      ],
      desc: 'Extended 1200-tick session verifying active rendering loops, GC, and physics stability.'
    }
  },
  {
    id: 'prince_sands',
    jar: 'prince_sands.jar',
    sha256: '40e7115a32e9e584106bc6565ed78cd6af189b41a426f5d61f53bbf8d0ac3e9f',
    width: 176,
    height: 208,
    init: {
      ticks: 280,
      snapshots: [60, 180, 280],
      inputs: [],
      desc: 'Cold boot through Gameloft logo to title / main menu screen.'
    },
    gameplay: {
      ticks: 500,
      snapshots: [280, 410, 460, 500],
      inputs: [
        { tick: 275, key: -7, action: 'press', free_key: 'F2' },
        { tick: 280, key: -7, action: 'release', free_key: 'F2' },
        { tick: 320, key: -7, action: 'press', free_key: 'F2' },
        { tick: 325, key: -7, action: 'release', free_key: 'F2' },
        { tick: 360, key: -7, action: 'press', free_key: 'F2' },
        { tick: 365, key: -7, action: 'release', free_key: 'F2' },
        { tick: 400, key: -7, action: 'press', free_key: 'F2' },
        { tick: 405, key: -7, action: 'release', free_key: 'F2' },
        { tick: 440, key: -4, action: 'press', free_key: 'ArrowRight' },
        { tick: 480, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 490, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 495, key: -4, action: 'release', free_key: 'ArrowRight' }
      ],
      desc: 'Start New Game, skip story dialogues, load Level 1, run right (-4) and attack (-5).'
    },
    longrunning: {
      ticks: 1200,
      snapshots: [300, 600, 900, 1200],
      inputs: [
        { tick: 275, key: -7, action: 'press', free_key: 'F2' },
        { tick: 280, key: -7, action: 'release', free_key: 'F2' },
        { tick: 320, key: -7, action: 'press', free_key: 'F2' },
        { tick: 325, key: -7, action: 'release', free_key: 'F2' },
        { tick: 360, key: -7, action: 'press', free_key: 'F2' },
        { tick: 365, key: -7, action: 'release', free_key: 'F2' },
        { tick: 400, key: -7, action: 'press', free_key: 'F2' },
        { tick: 405, key: -7, action: 'release', free_key: 'F2' },
        { tick: 450, key: -4, action: 'press', free_key: 'ArrowRight' },
        { tick: 700, key: -4, action: 'release', free_key: 'ArrowRight' },
        { tick: 750, key: -1, action: 'press', free_key: 'ArrowUp' },
        { tick: 780, key: -1, action: 'release', free_key: 'ArrowUp' },
        { tick: 850, key: -3, action: 'press', free_key: 'ArrowLeft' },
        { tick: 1100, key: -3, action: 'release', free_key: 'ArrowLeft' }
      ],
      desc: '1200 ticks of platformer mechanics, collision detection, and sprite layer rendering.'
    }
  },
  {
    id: 'doom_rpg',
    jar: 'doom_rpg.jar',
    sha256: 'a07ff089ec4b124220a52008ab3adaa205c51f40476c64258d91006604bbc6b3',
    width: 240,
    height: 320,
    init: {
      ticks: 240,
      snapshots: [60, 150, 240],
      inputs: [],
      desc: 'Cold boot to id Software / EA Mobile static title screen.'
    },
    gameplay: {
      ticks: 500,
      snapshots: [120, 220, 350, 500],
      inputs: [
        { tick: 60, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 65, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 120, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 125, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 180, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 185, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 260, key: -1, action: 'press', free_key: 'ArrowUp' },
        { tick: 300, key: -1, action: 'release', free_key: 'ArrowUp' },
        { tick: 330, key: -4, action: 'press', free_key: 'ArrowRight' },
        { tick: 360, key: -4, action: 'release', free_key: 'ArrowRight' },
        { tick: 400, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 420, key: -5, action: 'release', free_key: 'Enter' }
      ],
      desc: 'Dismiss copyright, select New Game, start Sector 1, step forward (-1), turn right (-4), interact (-5).'
    },
    longrunning: {
      ticks: 1200,
      snapshots: [300, 600, 900, 1200],
      inputs: [
        { tick: 60, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 65, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 120, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 125, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 180, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 185, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 300, key: -1, action: 'press', free_key: 'ArrowUp' },
        { tick: 350, key: -1, action: 'release', free_key: 'ArrowUp' },
        { tick: 500, key: -3, action: 'press', free_key: 'ArrowLeft' },
        { tick: 540, key: -3, action: 'release', free_key: 'ArrowLeft' },
        { tick: 700, key: -1, action: 'press', free_key: 'ArrowUp' },
        { tick: 750, key: -1, action: 'release', free_key: 'ArrowUp' },
        { tick: 900, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 940, key: -5, action: 'release', free_key: 'Enter' }
      ],
      desc: '1200 ticks of raycast rasterization, perspective wall textures, sprite sorting, and sound synthesis.'
    }
  },
  {
    id: 'castlevania',
    jar: 'castlevania.jar',
    sha256: 'dac5726ba3b9bb419a3b147e540415117b2f7c68909ea9a96b9d6de0d4da439b',
    width: 240,
    height: 320,
    init: {
      ticks: 300,
      snapshots: [50, 130, 220, 300],
      inputs: [
        { tick: 60, key: -6, action: 'press', free_key: 'F1' },
        { tick: 70, key: -6, action: 'release', free_key: 'F1' },
        { tick: 140, key: -6, action: 'press', free_key: 'F1' },
        { tick: 150, key: -6, action: 'release', free_key: 'F1' }
      ],
      desc: 'Select English (SoftLeft), accept Sound YES (SoftLeft), boot through Konami splash to Title.'
    },
    gameplay: {
      ticks: 650,
      snapshots: [140, 520, 600, 650],
      inputs: [
        { tick: 60, key: -6, action: 'press', free_key: 'F1' },
        { tick: 70, key: -6, action: 'release', free_key: 'F1' },
        { tick: 140, key: -6, action: 'press', free_key: 'F1' },
        { tick: 150, key: -6, action: 'release', free_key: 'F1' },
        { tick: 520, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 535, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 600, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 615, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 630, key: -4, action: 'press', free_key: 'ArrowRight' }
      ],
      desc: 'Pass language & sound dialogs, start New Game, move Soma right (-4).'
    },
    longrunning: {
      ticks: 1200,
      snapshots: [300, 600, 900, 1200],
      inputs: [
        { tick: 60, key: -6, action: 'press', free_key: 'F1' },
        { tick: 70, key: -6, action: 'release', free_key: 'F1' },
        { tick: 140, key: -6, action: 'press', free_key: 'F1' },
        { tick: 150, key: -6, action: 'release', free_key: 'F1' },
        { tick: 520, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 535, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 600, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 615, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 700, key: -4, action: 'press', free_key: 'ArrowRight' },
        { tick: 850, key: -4, action: 'release', free_key: 'ArrowRight' },
        { tick: 900, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 930, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 1000, key: -1, action: 'press', free_key: 'ArrowUp' },
        { tick: 1030, key: -1, action: 'release', free_key: 'ArrowUp' }
      ],
      desc: '1200 ticks of Aria of Sorrow gameplay: parallax scrolling, sprite transformation, and attack animations.'
    }
  },
  {
    id: 'gravity_defied',
    jar: 'gravity_defied.jar',
    sha256: '8c120d5eee9875061078d47016d9a850f0ceb1ec9d4b923e585d48d8d8b83f88',
    width: 176,
    height: 220,
    init: {
      ticks: 200,
      snapshots: [50, 120, 200],
      inputs: [],
      desc: 'Cold boot to Codebrew Software title screen and track menu.'
    },
    gameplay: {
      ticks: 500,
      snapshots: [120, 200, 350, 500],
      inputs: [
        { tick: 60, key: -6, action: 'press', free_key: 'F1' },
        { tick: 70, key: -6, action: 'release', free_key: 'F1' },
        { tick: 120, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 130, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 180, key: 51, action: 'press', free_key: 'Digit3' },
        { tick: 240, key: 54, action: 'press', free_key: 'Digit6' },
        { tick: 340, key: 54, action: 'release', free_key: 'Digit6' },
        { tick: 350, key: 52, action: 'press', free_key: 'Digit4' },
        { tick: 420, key: 52, action: 'release', free_key: 'Digit4' }
      ],
      desc: 'Select 100cc Track, start race, apply throttle (3), lean forward (6), and lean back (4).'
    },
    longrunning: {
      ticks: 1200,
      snapshots: [300, 600, 900, 1200],
      inputs: [
        { tick: 60, key: -6, action: 'press', free_key: 'F1' },
        { tick: 70, key: -6, action: 'release', free_key: 'F1' },
        { tick: 120, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 130, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 180, key: 51, action: 'press', free_key: 'Digit3' },
        { tick: 600, key: 54, action: 'press', free_key: 'Digit6' },
        { tick: 680, key: 54, action: 'release', free_key: 'Digit6' },
        { tick: 800, key: 52, action: 'press', free_key: 'Digit4' },
        { tick: 880, key: 52, action: 'release', free_key: 'Digit4' },
        { tick: 1100, key: 51, action: 'release', free_key: 'Digit3' }
      ],
      desc: '1200 ticks of full fixed-point ragdoll physics, spring suspensions, and vector line drawing.'
    }
  },
  {
    id: 'heroes_lore',
    jar: 'heroes_lore.jar',
    sha256: 'f7348e00fe222af207638880cc4baaa642dd1bb5a25ef9c55acc52b239611b90',
    width: 240,
    height: 320,
    init: {
      ticks: 250,
      snapshots: [60, 160, 250],
      inputs: [],
      desc: 'Cold boot through EA Mobile Korea / Hands-On logo to title cinematic.'
    },
    gameplay: {
      ticks: 550,
      snapshots: [120, 240, 400, 550],
      inputs: [
        { tick: 60, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 70, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 120, key: -6, action: 'press', free_key: 'F1' },
        { tick: 130, key: -6, action: 'release', free_key: 'F1' },
        { tick: 220, key: -6, action: 'press', free_key: 'F1' },
        { tick: 230, key: -6, action: 'release', free_key: 'F1' },
        { tick: 300, key: -4, action: 'press', free_key: 'ArrowRight' },
        { tick: 380, key: -4, action: 'release', free_key: 'ArrowRight' },
        { tick: 400, key: -2, action: 'press', free_key: 'ArrowDown' },
        { tick: 450, key: -2, action: 'release', free_key: 'ArrowDown' },
        { tick: 470, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 480, key: -5, action: 'release', free_key: 'Enter' }
      ],
      desc: 'Dismiss title, confirm character, skip intro dialogue, navigate town map right & down, interact (-5).'
    },
    longrunning: {
      ticks: 1200,
      snapshots: [300, 600, 900, 1200],
      inputs: [
        { tick: 60, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 70, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 120, key: -6, action: 'press', free_key: 'F1' },
        { tick: 130, key: -6, action: 'release', free_key: 'F1' },
        { tick: 220, key: -6, action: 'press', free_key: 'F1' },
        { tick: 230, key: -6, action: 'release', free_key: 'F1' },
        { tick: 400, key: -4, action: 'press', free_key: 'ArrowRight' },
        { tick: 600, key: -4, action: 'release', free_key: 'ArrowRight' },
        { tick: 700, key: -2, action: 'press', free_key: 'ArrowDown' },
        { tick: 900, key: -2, action: 'release', free_key: 'ArrowDown' },
        { tick: 1000, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 1030, key: -5, action: 'release', free_key: 'Enter' }
      ],
      desc: '1200 ticks of action RPG gameplay: TiledLayer world rendering, NPC pathing, dialogue scripts, RMS records.'
    }
  },
  {
    id: 'cyberpunk',
    jar: 'cyberpunk.jar',
    sha256: 'ded969828b625b374d29f81b609a6a7cc81cab9f70afea488ad85425e3367f91',
    width: 176,
    height: 208,
    init: {
      ticks: 240,
      snapshots: [60, 150, 240],
      inputs: [],
      desc: 'Cold boot through Mayhem Studio intro to Cyberpunk title banner.'
    },
    gameplay: {
      ticks: 500,
      snapshots: [140, 260, 380, 500],
      inputs: [
        { tick: 60, key: -6, action: 'press', free_key: 'F1' },
        { tick: 70, key: -6, action: 'release', free_key: 'F1' },
        { tick: 140, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 150, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 240, key: -7, action: 'press', free_key: 'F2' },
        { tick: 250, key: -7, action: 'release', free_key: 'F2' },
        { tick: 320, key: -4, action: 'press', free_key: 'ArrowRight' },
        { tick: 380, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 440, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 450, key: -4, action: 'release', free_key: 'ArrowRight' }
      ],
      desc: 'Accept Audio prompt, select New Game, skip mission briefing, move protagonist right, draw weapon.'
    },
    longrunning: {
      ticks: 1200,
      snapshots: [300, 600, 900, 1200],
      inputs: [
        { tick: 60, key: -6, action: 'press', free_key: 'F1' },
        { tick: 70, key: -6, action: 'release', free_key: 'F1' },
        { tick: 140, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 150, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 240, key: -7, action: 'press', free_key: 'F2' },
        { tick: 250, key: -7, action: 'release', free_key: 'F2' },
        { tick: 400, key: -4, action: 'press', free_key: 'ArrowRight' },
        { tick: 650, key: -4, action: 'release', free_key: 'ArrowRight' },
        { tick: 700, key: -5, action: 'press', free_key: 'Enter' },
        { tick: 750, key: -5, action: 'release', free_key: 'Enter' },
        { tick: 800, key: -3, action: 'press', free_key: 'ArrowLeft' },
        { tick: 1050, key: -3, action: 'release', free_key: 'ArrowLeft' }
      ],
      desc: '1200 ticks of Arasaka street traversal, MIDI background music playback, and enemy patrol updates.'
    }
  }
];

let count = 0;
for (const game of CANONICAL_GAMES) {
  for (const type of ['init', 'gameplay', 'longrunning']) {
    const sc = game[type];
    const scenarioObj = {
      schema_version: '1.0',
      game_id: game.id,
      jar_file: game.jar,
      jar_sha256: game.sha256,
      resolution: {
        width: game.width,
        height: game.height
      },
      scenario_type: type.toUpperCase(),
      description: sc.desc,
      total_ticks: sc.ticks,
      tick_step_ms: 16.66667,
      inputs: sc.inputs,
      snapshot_ticks: sc.snapshots,
      expected_behavior: sc.desc
    };

    const fileName = `${game.id}_${type}.json`;
    const filePath = path.join(SCENARIOS_DIR, fileName);
    fs.writeFileSync(filePath, JSON.stringify(scenarioObj, null, 2));
    count++;
    console.log(`Generated scenario [${count}/21]: ${fileName}`);
  }
}

console.log(`\n[SUCCESS] Successfully generated all ${count} deterministic test scenarios in: ${SCENARIOS_DIR}`);
