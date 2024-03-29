import {
  beforeEach,
  describe,
  it,
} from 'https://deno.land/std@0.152.0/testing/bdd.ts'
import { Level1Sim } from './sim.js'

const isReleaseBuild = !!Deno.env.get('RELEASE')

let sim = null
beforeEach(async () => {
  sim = new Level1Sim()
  await sim.loadWasm(
    isReleaseBuild
      ? // These paths are with the working directory of the justfile
        'target/wasm32-unknown-unknown/release/level_1.wasm'
      : 'target/wasm32-unknown-unknown/debug/level_1.wasm'
  )
  sim.initialize()
})

describe('Level 1', () => {
  it('steps', () => {
    const fps = sim.desiredFps()
    const deltaSeconds = 1 / fps
    for (let i = 0; i < fps * 0.75; i++) {
      sim.inputSetMoveForward(-1)
      sim.step(deltaSeconds)
    }
  })
})
