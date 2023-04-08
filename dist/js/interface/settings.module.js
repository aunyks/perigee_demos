import GUI from '/js/debug/lil-gui.module.js'
import Stats from '/js/debug/stats.module.js'
import { isInDebugMode } from '/js/misc/utils.module.js'

function bindSettings({ sim, audioListener }, setupDebugGui) {
  const perfStatistics = new Stats()
  document.body.appendChild(perfStatistics.dom)

  const masterVolumeSlider = document.getElementById('master-volume-slider')
  const horizSensSlider = document.getElementById('horiz-sens-slider')
  const vertSensSlider = document.getElementById('vert-sens-slider')
  const perfStatsCheckbox = document.getElementById('perf-stats-toggle')
  const debugToolsCheckbox = document.getElementById('debug-tools-toggle')

  if (!isInDebugMode()) {
    for (const debugSettingLabel of Array.from(
      document.querySelectorAll('label[for][data-debug]')
    )) {
      const debugInputId = debugSettingLabel.htmlFor
      if (!debugInputId) {
        continue
      }
      const debugInput = document.getElementById(debugInputId)
      debugInput.style.display = 'none'
      debugSettingLabel.style.display = 'none'
    }
  }

  const settings = {
    sim: {
      leftRightLookSensitivity: parseInt(horizSensSlider.value),
      upDownLookSensitivity: parseInt(vertSensSlider.value),
    },
    interface: {
      masterVolume: parseFloat(masterVolumeSlider.value) / 100,
      perfStatisticsEnabled: perfStatsCheckbox.checked,
      debugToolsEnabled: debugToolsCheckbox.checked,
    },
  }
  sim.setLeftRightLookSensitivity(settings.sim.leftRightLookSensitivity)
  sim.setUpDownLookSensitivity(settings.sim.upDownLookSensitivity)
  if (settings.interface.perfStatisticsEnabled) {
    perfStatistics.showPanel()
  }

  masterVolumeSlider.addEventListener('change', (e) => {
    settings.interface.masterVolume = parseFloat(e.target.value) / 100
    if (audioListener) {
      audioListener.setMasterVolume(settings.interface.masterVolume)
    }
  })

  horizSensSlider.addEventListener('change', (e) => {
    sim.setLeftRightLookSensitivity(parseInt(e.target.value))
  })

  vertSensSlider.addEventListener('change', (e) => {
    sim.setUpDownLookSensitivity(parseInt(e.target.value))
  })

  perfStatsCheckbox.addEventListener('change', (e) => {
    const checked = e.currentTarget.checked
    settings.interface.perfStatisticsEnabled = checked

    if (checked) {
      perfStatistics.showPanel()
    } else {
      perfStatistics.hideAllPanels()
    }
  })

  let debugGui = null
  debugToolsCheckbox.addEventListener('change', (e) => {
    const checked = e.currentTarget.checked
    settings.interface.debugToolsEnabled = checked

    if (checked) {
      if (debugGui === null) {
        debugGui = new GUI()
        setupDebugGui(debugGui)
      } else {
        debugGui.show()
      }
    } else {
      debugGui.hide()
    }
  })

  return perfStatistics
}

export { bindSettings }
