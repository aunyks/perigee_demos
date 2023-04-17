const isOpenClass = 'modal-is-open'
const openingClass = 'modal-is-opening'
const closingClass = 'modal-is-closing'
const animationDuration = 200 // ms
let visibleModal = null

// Get scrollbar width
const getScrollbarWidth = () => {
  // Creating invisible container
  const outer = document.createElement('div')
  outer.style.visibility = 'hidden'
  outer.style.overflow = 'scroll' // forcing scrollbar to appear
  outer.style.msOverflowStyle = 'scrollbar' // needed for WinJS apps
  document.body.appendChild(outer)

  // Creating inner element and placing it in the container
  const inner = document.createElement('div')
  outer.appendChild(inner)

  // Calculating difference between container's full width and the child width
  const scrollbarWidth = outer.offsetWidth - inner.offsetWidth

  // Removing temporary elements from the DOM
  outer.parentNode.removeChild(outer)

  return scrollbarWidth
}

// Is scrollbar visible
const isScrollbarVisible = () => {
  return document.body.scrollHeight > screen.height
}

// Toggle modal
const toggleModal = (modalId) => {
  isModalOpen(modalId) ? closeModal(modalId) : openModal(modalId)
}

function modalWithId(modalId) {
  return {
    preventDefault: () => {},
    target: {
      getAttribute: () => {
        return modalId
      },
    },
  }
}

// Is modal open
const isModalOpen = (modalId) => {
  const modal = document.getElementById(modalId)
  return modal.hasAttribute('open') && modal.getAttribute('open') != 'false'
    ? true
    : false
}

// Open modal
const openModal = (modalId) => {
  const modal = document.getElementById(modalId)
  if (isScrollbarVisible()) {
    document.documentElement.style.setProperty(
      '--scrollbar-width',
      `${getScrollbarWidth()}px`
    )
  }
  document.documentElement.classList.add(isOpenClass, openingClass)
  setTimeout(() => {
    visibleModal = modal
    document.documentElement.classList.remove(openingClass)
  }, animationDuration)
  modal.setAttribute('open', true)
  modal.setAttribute('aria-hidden', 'false')
  modal.focus()
}

// Close modal
const closeModal = (modalId) => {
  const modal = document.getElementById(modalId)
  visibleModal = null
  document.documentElement.classList.add(closingClass)
  setTimeout(() => {
    document.documentElement.classList.remove(closingClass, isOpenClass)
    document.documentElement.style.removeProperty('--scrollbar-width')
    modal.removeAttribute('open')
  }, animationDuration)
  modal.setAttribute('aria-hidden', 'true')
}

// Close with a click outside
document.addEventListener('click', (event) => {
  if (
    visibleModal != null &&
    visibleModal.getAttribute('can-exit') !== 'false'
  ) {
    const modalContent = visibleModal.querySelector('article')
    const isClickInside = modalContent.contains(event.target)
    !isClickInside && closeModal(visibleModal)
  }
})

// Close with Esc key
document.addEventListener('keydown', (event) => {
  if (event.key === 'Escape' && visibleModal != null) {
    closeModal(visibleModal)
  }
})

export { toggleModal, closeModal, openModal, isModalOpen, modalWithId }
