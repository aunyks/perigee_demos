import {
  Object3D,
  DirectionalLight,
  Scene,
  WebGLRenderer,
  AnimationMixer,
  AnimationClip,
  AudioListener,
  ColorManagement,
  Color,
  Group,
  MeshBasicMaterial,
  Vector2,
  ACESFilmicToneMapping,
  sRGBEncoding,
  BoxGeometry,
  Mesh,
  PositionalAudio,
  Audio,
} from '/js/graphics/three.module.js'
import { EffectComposer } from '/js/graphics/postprocessing/EffectComposer.js'
import { RenderPass } from '/js/graphics/postprocessing/RenderPass.js'
import { UnrealBloomPass } from '/js/graphics/postprocessing/UnrealBloomPass.js'
import { ShaderPass } from '/js/graphics/postprocessing/ShaderPass.js'
import { FXAAShader } from '/js/graphics/postprocessing/shaders/FXAAShader.js'
import { GameInput } from '/js/input/game-input.module.js'
import { Level2Sim } from '/js/levels/2/Level2Sim.module.js'
import {
  randomIntFromZero,
  bindAssistiveDeviceAnnouncer,
  bindNotificationBanner,
  isInDebugMode,
} from '/js/misc/utils.module.js'
import {
  promiseLoadAudioBuffer,
  promiseLoadGltf,
  promiseParseGltf,
} from '/js/graphics/three-ext/utils.module.js'
import SkyDome from '/js/graphics/prefabs/skydome.module.js'
import Sun from '/js/graphics/prefabs/sun.module.js'
import { toggleModal, modalWithId } from '/js/components/modal.module.js'
import { bindSettings } from '/js/interface/settings.module.js'
import MarkerCylinder from '/js/graphics/prefabs/MarkerCylinder.module.js'

const loadingContainer = document.getElementById('loading-container')
const sceneContainer = document.getElementById('scene-container')
const sceneCanvas = document.getElementById('scene-canvas')

const adAnnounce = bindAssistiveDeviceAnnouncer(
  document.getElementById('ad-announcer')
)
adAnnounce(loadingContainer.innerText)

const notify = bindNotificationBanner(
  document.getElementById('notification-banner'),
  adAnnounce
)

const simulation = new Level2Sim()
await simulation.loadWasm('/wasm/levels/2/sim.wasm')

const assetsToLoad = [
  simulation,
  // Visuals
  promiseParseGltf(simulation.getSceneGltfBytes()),
  promiseLoadGltf('/gltf/player-camera.glb'),
  promiseLoadGltf('/gltf/sedan.glb'),
  // Audio
  promiseLoadAudioBuffer('/audio/level/main-music.mp3'),
]

// Load all assets and then we're ready to load the scene
Promise.all(assetsToLoad)
  .then(
    ([
      sim,
      // Visuals
      sceneGltf,
      animatedCameraGltf,
      sedanGltf,
      // Audio
      levelMusicAudioBuffer,
    ]) => {
      loadingContainer.remove()
      sceneContainer.classList.remove('hidden')

      const renderer = new WebGLRenderer({
        canvas: sceneCanvas,
        antialias: window.devicePixelRatio > 1 ? false : true,
      })

      renderer.setSize(sceneContainer.clientWidth, sceneContainer.clientHeight)
      renderer.setPixelRatio(window.devicePixelRatio)
      renderer.physicallyCorrectLights = true
      ColorManagement.enabled = true
      ColorManagement.legacyMode = false
      renderer.toneMappingExposure = 1
      renderer.outputEncoding = sRGBEncoding
      renderer.toneMapping = ACESFilmicToneMapping
      sceneContainer.append(renderer.domElement)

      // Prepare our scene
      const mainScene = new Scene()

      const sedanCabin = sedanGltf.scene.getObjectByName('cabin')
      mainScene.add(sedanCabin)
      const wheels = [
        sedanGltf.scene.getObjectByName('front_left_wheel'),
        sedanGltf.scene.getObjectByName('front_right_wheel'),
        sedanGltf.scene.getObjectByName('back_left_wheel'),
        sedanGltf.scene.getObjectByName('back_right_wheel'),
      ]
      for (const wheel of wheels) {
        mainScene.add(wheel)
      }

      // Create our background environment
      const backgroundEnvironment = new Group()
      backgroundEnvironment.add(new SkyDome())
      const sun = new Sun()
      sun.pivot.rotation.set(Math.PI / 6, 0, 0, 'YXZ')
      backgroundEnvironment.add(sun)
      backgroundEnvironment.renderOrder = -Number.MAX_SAFE_INTEGER
      mainScene.add(backgroundEnvironment)

      sceneGltf.scene.traverse((obj) => {
        if (!!obj.isMesh && !obj.userData.simSettings.graphics.enabled) {
          obj.geometry.dispose()
          obj.material.dispose()
          obj.visible = false
        }
      })

      mainScene.add(sceneGltf.scene)
      mainScene.add(new DirectionalLight(0xffffff, 10))

      const animatedCamera = animatedCameraGltf.cameras[0]
      animatedCamera.fov = 35
      animatedCamera.aspect =
        sceneContainer.clientWidth / sceneContainer.clientHeight
      animatedCamera.near = 0.01
      animatedCamera.far = 1000
      const cameraRig = new Object3D()
      cameraRig.add(animatedCameraGltf.scene)
      mainScene.add(cameraRig)

      const [cameraIdleAnimation, cameraRunningAnimation] =
        animatedCameraGltf.animations

      const audioListener = new AudioListener()
      animatedCamera.add(audioListener)

      const perfStatistics = bindSettings(
        { sim, audioListener },
        (debugGui) => {
          debugGui.add(document, 'title')
        }
      )

      const levelMusicPositionalAudio = new Audio(audioListener).setBuffer(
        levelMusicAudioBuffer
      )
      const playerAudioTracks = new Map([
        ['LEVEL_MUSIC', { track: levelMusicPositionalAudio, detune: null }],
      ])
      const sceneTracks = new Map([['PLAYER', playerAudioTracks]])
      const sceneMixers = new Map([
        [
          'CAMERA',
          {
            mixer: new AnimationMixer(animatedCameraGltf.scene),
            clips: animatedCameraGltf.animations,
          },
        ],
      ])

      sim.events.on('LEVEL_COMPLETED', () => {
        notify('Level complete!', 'success', 3000).then(() => {
          stopGameplay()
          if (document.exitPointerLock !== undefined) {
            document.exitPointerLock()
          }
          toggleModal(modalWithId('post-level-modal'))
        })
      })

      sim.events.on(
        'PLAY_AUDIO',
        (sceneObj, audioName, playbackRate, volume) => {
          const audioTracks = sceneTracks.get(sceneObj)
          if (audioTracks) {
            const audio = audioTracks.get(audioName)
            if (audio) {
              const audioTrack = audio.track
              if (audioTrack.isPlaying) {
                audioTrack.stop()
              }
              if (audio.detune) {
                audioTrack.detune =
                  100 * (randomIntFromZero(audio.detune[0]) - audio.detune[1])
              }
              audioTrack.setVolume(volume).setPlaybackRate(playbackRate).play()
            }
          }
        }
      )

      sim.events.on(
        'LOOP_AUDIO',
        (sceneObj, audioName, playbackRate, volume) => {
          const audioTracks = sceneTracks.get(sceneObj)
          if (audioTracks) {
            const audio = audioTracks.get(audioName)
            if (audio) {
              const audioTrack = audio.track
              if (audioTrack.isPlaying) {
                audioTrack.stop()
              }
              if (audio.detune) {
                audioTrack.detune =
                  100 * (randomIntFromZero(audio.detune[0]) - audio.detune[1])
              }
              audioTrack
                .setVolume(volume)
                .setLoop(true)
                .setPlaybackRate(playbackRate)
                .play()
            }
          }
        }
      )

      sim.events.on('STOP_AUDIO', (sceneObj, audioName) => {
        const audioTracks = sceneTracks.get(sceneObj)
        if (audioTracks) {
          const audio = audioTracks.get(audioName)
          if (audio) {
            const audioTrack = audio.track
            if (audioTrack.isPlaying) {
              audioTrack.stop()
            }
          }
        }
      })

      const ANIM_FADE_DURATION = 0.15

      sim.events.on('LOOP_ANIMATION', (sceneObj, animName, timeScale) => {
        const detailedMixer = sceneMixers.get(sceneObj)
        if (detailedMixer) {
          const anim = detailedMixer.mixer.clipAction(
            AnimationClip.findByName(detailedMixer.clips, animName).optimize()
          )
          if (anim !== null) {
            anim.timeScale = timeScale
            anim.reset().fadeIn(ANIM_FADE_DURATION).play()
          }
        }
      })

      sim.events.on('STOP_ANIMATION', (sceneObj, animName) => {
        const detailedMixer = sceneMixers.get(sceneObj)
        if (detailedMixer) {
          const anim = detailedMixer.mixer.clipAction(
            AnimationClip.findByName(detailedMixer.clips, animName).optimize()
          )
          if (anim !== null) {
            anim.fadeOut(ANIM_FADE_DURATION)
          }
        }
      })

      sim.events.on('AD_ANNOUNCEMENT', (msg) => {
        adAnnounce(msg)
      })

      sim.initialize()

      const gameInput = new GameInput({
        gamepads: [
          {
            index: 0,
            onConnect: () => {
              document
                .querySelectorAll('.hud.input')
                .forEach((hudInputElement) => {
                  hudInputElement.style.display = 'none'
                })
            },
            onDisconnect: () => {
              document
                .querySelectorAll('.hud.input')
                .forEach((hudInputElement) => {
                  hudInputElement.style.display = 'block'
                })
            },
          },
        ],
      })

      document.getElementById('pause-button').addEventListener('click', () => {
        pauseGame()
      })

      let gameLoopContext = null
      let lastTimestamp = null
      let deltaT = 0
      let activeCamera = animatedCamera
      let accumulatedTimestep = 0
      const desiredTimestep = 1 / sim.desiredFps()
      const MAX_FRAMES_TO_DROP = 3

      const postProcessComposer = new EffectComposer(renderer)
      postProcessComposer.addPass(new RenderPass(mainScene, activeCamera))
      postProcessComposer.addPass(
        new UnrealBloomPass(
          new Vector2(sceneContainer.offsetWidth, sceneContainer.offsetHeight),
          0.7,
          3,
          0.99
        )
      )
      postProcessComposer.addPass(new ShaderPass(FXAAShader))

      function onGameLoopTick(tFrame) {
        deltaT = Math.abs(tFrame - lastTimestamp)
        perfStatistics.begin()
        {
          const deltaSeconds = deltaT / 1000
          accumulatedTimestep += deltaSeconds
          if (accumulatedTimestep >= desiredTimestep * MAX_FRAMES_TO_DROP) {
            accumulatedTimestep = desiredTimestep * MAX_FRAMES_TO_DROP
          }
          while (accumulatedTimestep >= desiredTimestep) {
            accumulatedTimestep -= desiredTimestep
            gameInput.update()
            if (gameInput.pause()) {
              pauseGame()
              return
            }
            gameInput.copyToSim(sim)
            sim.step(desiredTimestep)
          }

          for (const detailedMixer of sceneMixers.values()) {
            detailedMixer.mixer.update(deltaSeconds)
          }

          sceneGltf.scene.traverse((obj) => {
            if (obj.isMesh && !obj.userData.simSettings.physics.isAnonymous) {
              const propName = obj.userData.name
              const [propRot, propTrans] = sim.propIsometry(propName)
              obj.position.fromArray(propTrans)
              obj.quaternion.fromArray(propRot)
            }
          })

          const [camGlobalRotation, camGlobalTranslation] =
            sim.cameraGlobalIsometry()
          cameraRig.position.fromArray(camGlobalTranslation)
          cameraRig.quaternion.fromArray(camGlobalRotation)

          const [cabinRotation, cabinTranslation] = sim.carCabinIsometry()
          sedanCabin.position.fromArray(cabinTranslation)
          sedanCabin.quaternion.fromArray(cabinRotation)
          for (let wheelIdx = 0; wheelIdx < wheels.length; wheelIdx++) {
            const wheel = wheels[wheelIdx]
            const [wheelRotation, wheelTranslation] =
              sim.wheelIsometry(wheelIdx)
            wheel.position.fromArray(wheelTranslation)
            wheel.quaternion.fromArray(wheelRotation)
          }

          activeCamera.getWorldPosition(backgroundEnvironment.position)

          postProcessComposer.render(deltaSeconds)
        }
        perfStatistics.end()
        lastTimestamp = tFrame
        gameLoopContext = window.requestAnimationFrame(onGameLoopTick)
      }

      function startGameplay() {
        audioListener.context.resume()
        sceneCanvas.focus()
        adAnnounce('Gameplay started')

        // Somehow this subtraction prevents abortions on gameplay resume
        // Using 45 because it's equidistant between 30fps (on lower end devices)
        // and 60 fps
        lastTimestamp = window.performance.now()
        onGameLoopTick(window.performance.now())
      }

      function stopGameplay() {
        audioListener.context.suspend()
        window.cancelAnimationFrame(gameLoopContext)
        adAnnounce('Gameplay stopped')
      }

      function resetCameraProjection() {
        const aspect = window.innerWidth / window.innerHeight
        activeCamera.aspect = aspect
        activeCamera.updateProjectionMatrix()
        renderer.setSize(window.innerWidth, window.innerHeight)
      }
      window.addEventListener('resize', resetCameraProjection, false)

      function pauseGame() {
        stopGameplay()
        toggleModal(modalWithId('pause-modal'))
        adAnnounce('Pause menu opened')
      }

      function resumeGame() {
        toggleModal(modalWithId('pause-modal'))
        startGameplay()
      }

      function resetGame() {
        sim.reset()
        adAnnounce('Game reset')
      }

      document.body.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
          pauseGame()
        }
      })

      document
        .getElementById('resume-game-button')
        .addEventListener('click', () => {
          resumeGame()
        })

      document
        .getElementById('restart-level-button')
        .addEventListener('click', () => {
          toggleModal(modalWithId('pause-modal'))
          toggleModal(modalWithId('restart-level-conf-modal'))
        })

      document
        .getElementById('restart-level-conf-button')
        .addEventListener('click', () => {
          toggleModal(modalWithId('restart-level-conf-modal'))
          resetGame()
          startGameplay()
        })

      document
        .getElementById('restart-level-deny-button')
        .addEventListener('click', () => {
          toggleModal(modalWithId('restart-level-conf-modal'))
          toggleModal(modalWithId('pause-modal'))
        })

      document
        .getElementById('settings-button')
        .addEventListener('click', () => {
          toggleModal(modalWithId('pause-modal'))
          toggleModal(modalWithId('settings-modal'))
        })

      document
        .getElementById('settings-back-button')
        .addEventListener('click', () => {
          toggleModal(modalWithId('settings-modal'))
          toggleModal(modalWithId('pause-modal'))
        })

      document
        .getElementById('quit-game-button')
        .addEventListener('click', () => {
          window.location.href = '/'
        })

      let levelStarted = false
      document.addEventListener('visibilitychange', function () {
        if (document.visibilityState !== 'visible' && levelStarted) {
          pauseGame()
        }
      })

      renderer.compile(mainScene, activeCamera)
      toggleModal(modalWithId('intro-modal'))
      const startBtn = document.getElementById('start-game-button')
      startBtn.addEventListener('click', () => {
        resetCameraProjection()
        startGameplay()
        toggleModal(modalWithId('intro-modal'))
        levelStarted = true
      })
      adAnnounce('Loading complete')
    }
  )
  .catch((e) => {
    console.error(e)
    toggleModal(modalWithId('error-modal'))
  })
