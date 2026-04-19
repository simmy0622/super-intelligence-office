export function getAppScrollContainer() {
  return document.querySelector<HTMLElement>("[data-app-scroll-container]");
}

export function scrollAppToTop(behavior: ScrollBehavior = "auto") {
  const container = getAppScrollContainer();

  if (container) {
    container.scrollTo({ top: 0, behavior });
    return;
  }

  window.scrollTo({ top: 0, behavior });
}
