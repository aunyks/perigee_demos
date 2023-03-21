import { WasmUtils } from '../shared/wasm-ffi-utils.js'
import { GameEvents } from '../shared/events.js'

class Sim extends WasmUtils {
  static async fromWasmBinary(wasmPath) {
    const gameEventEmitter = new GameEvents()
    let wasmModule = null
    let wasmMemory = new WebAssembly.Memory({
      initial: 1,
      maximum: 2 ** 16,
    })
    const wasmFunctionImports = {
      now: () => performance.now(),
      on_level_event: (type) => {
        console.log('UNHANDLED LEVEL EVENT EMITTED: ', type)
      },
      play_audio_hook: (
        sceneObjPtr,
        sceneObjLen,
        audioNamePtr,
        audioNameLen,
        playbackRate
      ) => {
        const sceneObj = this.getString(wasmMemory, sceneObjPtr, sceneObjLen)
        const audioName = this.getString(wasmMemory, audioNamePtr, audioNameLen)
        gameEventEmitter.emit('PLAY_AUDIO', [sceneObj, audioName, playbackRate])
      },
      stop_audio_hook: (
        sceneObjPtr,
        sceneObjLen,
        audioNamePtr,
        audioNameLen
      ) => {
        const sceneObj = this.getString(wasmMemory, sceneObjPtr, sceneObjLen)
        const audioName = this.getString(wasmMemory, audioNamePtr, audioNameLen)
        gameEventEmitter.emit('STOP_AUDIO', [sceneObj, audioName])
      },
      loop_audio_hook: (
        sceneObjPtr,
        sceneObjLen,
        audioNamePtr,
        audioNameLen,
        playbackRate
      ) => {
        const sceneObj = this.getString(wasmMemory, sceneObjPtr, sceneObjLen)
        const audioName = this.getString(wasmMemory, audioNamePtr, audioNameLen)
        gameEventEmitter.emit('LOOP_AUDIO', [sceneObj, audioName, playbackRate])
      },
      loop_animation_hook: (
        sceneObjNamePtr,
        sceneObjNameLen,
        animNamePtr,
        animNameLen,
        timeScale
      ) => {
        const sceneObjName = this.getString(
          wasmMemory,
          sceneObjNamePtr,
          sceneObjNameLen
        )
        const animName = this.getString(wasmMemory, animNamePtr, animNameLen)
        gameEventEmitter.emit('LOOP_ANIMATION', [
          sceneObjName,
          animName,
          timeScale,
        ])
      },
      stop_animation_hook: (
        sceneObjNamePtr,
        sceneObjNameLen,
        animNamePtr,
        animNameLen
      ) => {
        const sceneObjName = this.getString(
          wasmMemory,
          sceneObjNamePtr,
          sceneObjNameLen
        )
        const animName = this.getString(wasmMemory, animNamePtr, animNameLen)
        gameEventEmitter.emit('STOP_ANIMATION', [sceneObjName, animName])
      },
      assistive_device_announce_hook: (
        announcementMsgPtr,
        announcementMsgLen
      ) => {
        const announcementMsg = this.getString(
          wasmMemory,
          announcementMsgPtr,
          announcementMsgLen
        )
        console.log(announcementMsg)
        gameEventEmitter.emit('AD_ANNOUNCEMENT', [announcementMsg])
      },
      on_error: (stringPtr, strLen) => {
        const str = this.getString(wasmMemory, stringPtr, strLen)
        console.error(str)
      },
      on_warn: (stringPtr, strLen) => {
        const str = this.getString(wasmMemory, stringPtr, strLen)
        console.warn(str)
      },
      on_debug: (stringPtr, strLen) => {
        const str = this.getString(wasmMemory, stringPtr, strLen)
        console.log('[DEBUG]', str)
      },
      on_info: (stringPtr, strLen) => {
        const str = this.getString(wasmMemory, stringPtr, strLen)
        console.log('[INFO]', str)
      },
      on_trace: (stringPtr, strLen) => {
        const str = this.getString(wasmMemory, stringPtr, strLen)
        console.log('[TRACE]', str)
      },
    }

    const wasmImports = {
      env: wasmFunctionImports,
      js: {
        mem: wasmMemory,
      },
    }
    if (typeof Deno !== 'undefined') {
      const wasmBuffer = Deno.readFileSync(wasmPath)
      wasmModule = await WebAssembly.instantiate(wasmBuffer, wasmImports)
    } else {
      wasmModule = await WebAssembly.instantiateStreaming(
        fetch(wasmPath),
        wasmImports
      )
    }
    const wasmExports = wasmModule.instance.exports
    wasmMemory = wasmModule.instance.exports.memory

    // const vecPtr = wasmExports.allocate_vector3f32_space()
    // const quatPtr = wasmExports.allocate_unitquaternionf32_space()
    const isoPtr =
      wasmExports.allocate_isometry_f32_unitquaternion_f32__3__space()

    const gamePointer = wasmExports.create_sim()

    const game = new Sim(
      wasmExports,
      wasmMemory,
      gamePointer,
      // vecPtr,
      // quatPtr,
      isoPtr,
      gameEventEmitter
    )
    return game
  }

  constructor(
    exports,
    memory,
    gamePtr,
    // vecPtr,
    // quatPtr,
    isoPtr,
    gameEventEmitter
  ) {
    super()
    this._wasmExports = exports
    this._wasmMemory = memory
    this._simPointer = gamePtr
    // this._vectorPointer = vecPtr
    // this._quaternionPointer = quatPtr
    this._isometryPointer = isoPtr
    this.events = gameEventEmitter
  }

  initialize() {
    this._wasmExports.initialize_sim(this._simPointer)
  }

  desiredFps() {
    return this._wasmExports.desired_fps()
  }

  reset() {
    this._wasmExports.destroy_sim(this._simPointer)
    this._simPointer = this._wasmExports.create_sim()
    this.initialize()
  }

  getSceneGltfBytes() {
    const ptrToGltf = this._wasmExports.scene_gltf_bytes_ptr(this._simPointer)
    const gltfLen = this._wasmExports.scene_gltf_bytes_len(this._simPointer)
    return this._wasmMemory.buffer.slice(ptrToGltf, ptrToGltf + gltfLen)
  }

  getPlayerGltfBytes() {
    const ptrToGltf = this._wasmExports.player_gltf_bytes_ptr(this._simPointer)
    const gltfLen = this._wasmExports.player_gltf_bytes_len(this._simPointer)
    return this._wasmMemory.buffer.slice(ptrToGltf, ptrToGltf + gltfLen)
  }

  inputSetMoveForward(newMagnitude) {
    this._wasmExports.input_set_move_forward(this._simPointer, newMagnitude)
  }

  inputSetMoveRight(newMagnitude) {
    this._wasmExports.input_set_move_right(this._simPointer, newMagnitude)
  }

  inputSetRotateUp(newMagnitude) {
    this._wasmExports.input_set_rotate_up(this._simPointer, newMagnitude)
  }

  inputSetRotateRight(newMagnitude) {
    this._wasmExports.input_set_rotate_right(this._simPointer, newMagnitude)
  }

  inputSetJump(jumpVal) {
    this._wasmExports.input_set_jump(this._simPointer, jumpVal ? 1 : 0)
  }

  inputSetCrouch(crouchVal) {
    this._wasmExports.input_set_crouch(this._simPointer, crouchVal ? 1 : 0)
  }

  inputSetAim(aimVal) {
    this._wasmExports.input_set_aim(this._simPointer, aimVal ? 1 : 0)
  }

  step(deltaSeconds) {
    this._wasmExports.step(this._simPointer, deltaSeconds)
    // this._wasmExports.get_player_interface_events(this._simPointer)
    // this._wasmExports.get_level_events(this._simPointer)
  }

  leftRightLookSensitivity() {
    return this._wasmExports.settings_left_right_look_sensitivity(
      this._simPointer
    )
  }

  upDownLookSensitivity() {
    return this._wasmExports.settings_up_down_look_sensitivity(this._simPointer)
  }

  setLeftRightLookSensitivity(newSensitivity) {
    this._wasmExports.settings_set_left_right_look_sensitivity(
      this._simPointer,
      newSensitivity
    )
  }

  setUpDownLookSensitivity(newSensitivity) {
    this._wasmExports.settings_set_up_down_look_sensitivity(
      this._simPointer,
      newSensitivity
    )
  }

  propIsometry(name) {
    this._wasmExports.prop_isometry(
      this._simPointer,
      this.ptrToString(name, this._wasmExports.alloc_string, this._wasmMemory),
      this._isometryPointer
    )
    return this.getIsometryF32(this._wasmMemory, this._isometryPointer)
  }

  playerBodyIsometry() {
    this._wasmExports.player_body_isometry(
      this._simPointer,
      this._isometryPointer
    )
    return this.getIsometryF32(this._wasmMemory, this._isometryPointer)
  }

  carCabinIsometry() {
    this._wasmExports.car_cabin_isometry(
      this._simPointer,
      this._isometryPointer
    )
    return this.getIsometryF32(this._wasmMemory, this._isometryPointer)
  }

  cameraGlobalIsometry() {
    this._wasmExports.camera_global_isometry(
      this._simPointer,
      this._isometryPointer
    )
    return this.getIsometryF32(this._wasmMemory, this._isometryPointer)
  }
}

export { Sim }
