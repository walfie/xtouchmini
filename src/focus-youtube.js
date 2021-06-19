const app = Application("Google Chrome");
app.activate();

const youtubeUrl = "youtube.com/watch?v=";

// Find YouTube tab
if (!app.windows[0].activeTab.url().includes(youtubeUrl)) {
  for (const [winIndex, win] of app.windows().entries()) {
    const tabIndex = win
      .tabs()
      .findIndex(tab => tab.url().includes(youtubeUrl));

    if (tabIndex != -1) {
      win.activeTabIndex = tabIndex + 1;
      win.index = 1;
      if (winIndex != 0) {
        win.visible = false;
        win.visible = true;
      }
      break;
    }
  }
}

// This will run inside Chrome
const focusTextInput = () => {
  const el = document
    .querySelector('#chatframe')
    .contentWindow
    .document
    .querySelector('#input')
    .querySelector('#input');

  const selection = window.getSelection();
  const range = document.createRange();
  selection.removeAllRanges();
  range.selectNodeContents(el);
  range.collapse(false);
  selection.addRange(range);
  el.focus();
};

const javascript = `(${focusTextInput.toString()})()`;

app.execute(app.windows[0].activeTab, { javascript });

