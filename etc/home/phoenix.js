let MARGIN_X = 0;
let MARGIN_Y = 0;
let GRID_WIDTH = 16;
let GRID_HEIGHT = 9;


function getWindowAndFrame() {
  const screen = Screen.main();
  const window = Window.focused();
  console.log('screen.frame', JSON.stringify(screen.flippedVisibleFrame(), null, 4));
  console.log('window.frame', JSON.stringify(window.frame(), null, 4));
  return [screen, window]
}

Key.on('c', ['command', 'option'], () => {
  const [screen,window] = getWindowAndFrame();
  if (window) {
    window.setTopLeft({
      x: screen.x + (screen.frame().width / 2) - (window.frame().width / 2),
      y: screen.y + (screen.frame().height / 2) - (window.frame().height / 2)
    });
  }
});


Key.on('p', ['command', 'shift'], () => {
  const screen = Screen.main().flippedVisibleFrame();
  Mouse.move({
    x: (screen.width / 2),
    y: (screen.height / 2),
  });

});


Key.on('f', ['command', 'option'], () => {
  const [screen,window] = getWindowAndFrame();

  if (window) {
    window.maximize()
  }
});


Key.on('left', ['command', 'option'], () => {
  const [screen,window] = getWindowAndFrame();

  if (window) {
    window.setSize({
      width: (screen.flippedVisibleFrame().width / 2),
      height: (screen.flippedVisibleFrame().height / 2),
    });
    window.setTopLeft({
      x: 0,
      y: 0,
    });

  }
});

Key.on('right', ['command', 'option'], () => {
  const [screen,window] = getWindowAndFrame();

  if (window) {
    window.setSize({
      width: (screen.flippedVisibleFrame().width / 2),
      height: (screen.flippedVisibleFrame().height / 2),
    });
    window.setTopLeft({
      y: 0,
      x: (screen.flippedVisibleFrame().width / 2) - (window.frame().width / 2),
    });

  }
});

Key.on('left', ['control', 'command'], () => {
  const [screen,window] = getWindowAndFrame();

  if (window) {
    window.setSize({
      width: (screen.flippedVisibleFrame().width / 2),
      height: (screen.flippedVisibleFrame().height / 2),
    });

    window.setTopLeft({
      x: 0,
      y: (screen.flippedVisibleFrame().height / 2) - (window.frame().height / 2),
    });

  }
});
