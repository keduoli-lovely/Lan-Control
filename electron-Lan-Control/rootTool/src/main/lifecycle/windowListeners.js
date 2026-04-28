const createListeners = (win, runShutdown) => {
  return {
    min: () => win.minimize(),
    max: () => win.maximize(),
    unmaximize: () => win.unmaximize(),
    close: () => {
      runShutdown();
      win.close();
    }
  };
}

export default createListeners;