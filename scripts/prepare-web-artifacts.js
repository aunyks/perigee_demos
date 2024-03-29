import * as fs from 'https://deno.land/std@0.152.0/node/fs.ts'
import * as path from 'https://deno.land/std/path/mod.ts'
import { copySync } from 'https://deno.land/std@0.163.0/fs/copy.ts'

const cwd = Deno.cwd()
const isReleaseBuild = !!Deno.env.get('RELEASE')

// Copy player sliding sounds into the dist
fs.copyFileSync(
  path.joinGlobs([cwd, 'assets', 'audio', 'slide.mp3']),
  path.joinGlobs([cwd, 'dist', 'audio', 'player', 'slide.mp3']),
  fs.constants.COPYFILE_FICLONE
)

// Copy player jump sounds into the dist
fs.copyFileSync(
  path.joinGlobs([cwd, 'assets', 'audio', 'jump.mp3']),
  path.joinGlobs([cwd, 'dist', 'audio', 'player', 'jump.mp3']),
  fs.constants.COPYFILE_FICLONE
)

// Copy player footstep sounds into the dist
fs.copyFileSync(
  path.joinGlobs([cwd, 'assets', 'audio', 'footstep.mp3']),
  path.joinGlobs([cwd, 'dist', 'audio', 'player', 'footstep.mp3']),
  fs.constants.COPYFILE_FICLONE
)

copySync(
  path.joinGlobs([cwd, 'assets', 'gltf', 'shared']),
  path.joinGlobs([cwd, 'dist', 'gltf']),
  { overwrite: true }
)

// Copy shared sim utils to web interface
copySync(
  path.joinGlobs([
    cwd,
    'single_player',
    'wasm_js_wrappers',
    'levels',
    'shared',
  ]),
  path.joinGlobs([cwd, 'dist', 'js', 'levels', 'shared']),
  {
    overwrite: true,
  }
)

const gltfLevelsPath = path.joinGlobs([cwd, 'assets', 'gltf', 'levels'])
fs.readdirSync(gltfLevelsPath).forEach((fileOrDir) => {
  if (fs.statSync(path.joinGlobs([gltfLevelsPath, fileOrDir])).isDirectory()) {
    const levelName = fileOrDir

    // Copy the built WASM binary for the level into
    // the dist WASM folder
    fs.copyFileSync(
      path.joinGlobs([
        cwd,
        'target',
        'wasm32-unknown-unknown',
        isReleaseBuild ? 'release' : 'debug',
        `level_${levelName}.wasm`,
      ]),
      path.joinGlobs([cwd, 'dist', 'wasm', 'levels', levelName, 'sim.wasm']),
      fs.constants.COPYFILE_FICLONE
    )

    const outputSimFilePath = path.joinGlobs([
      cwd,
      'dist',
      'js',
      'levels',
      levelName,
      `Level${levelName}Sim.module.js`,
    ])

    // Copy the level's JavaScript wrapper from the WASM crate / module
    // to the dist simulations folder
    fs.copyFileSync(
      path.joinGlobs([
        cwd,
        'single_player',
        'wasm_js_wrappers',
        `levels`,
        levelName,
        'sim.js',
      ]),
      outputSimFilePath,
      fs.constants.COPYFILE_FICLONE
    )
  }
})
