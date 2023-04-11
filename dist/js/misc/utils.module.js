/**
 * Returns a random integer within the range [0, max).
 *
 * @param {number} max
 *
 * @returns { integer }
 */
function randomIntFromZero(max = 0) {
  return parseInt(Math.random() * max)
}

/**
 * Receives an element with an aria-live attribute and returns a function
 * for creating announcements from that element.
 *
 * @param {announcerElem} HTMLElement
 *
 * @returns { Function }
 */
function bindAssistiveDeviceAnnouncer(announcerElem) {
  return (announcement) => {
    announcerElem.innerText = announcement
  }
}

function noop() {}

function bindNotificationBanner(
  notificationBanner,
  assistiveDeviceAnnounce = noop
) {
  return (msg, type, sustain) => {
    notificationBanner.innerText = msg
    notificationBanner.classList.add(type, 'active')
    assistiveDeviceAnnounce(msg)
    return new Promise((resolve) => {
      setTimeout(() => {
        notificationBanner.classList.remove(type, 'active')
        setTimeout(() => {
          resolve()
        }, 250)
      }, sustain)
    })
  }
}

function isInDebugMode() {
  return !!new URLSearchParams(document.location.search).get('debug')
}

export {
  randomIntFromZero,
  bindAssistiveDeviceAnnouncer,
  bindNotificationBanner,
  noop,
  isInDebugMode,
}
