const app = Application("Google Chrome");
app.activate();

const youtubeUrl = "youtube.com/watch?v=";
const twitchUrl = "twitch.tv/"

// Find YouTube tab
const activeTab = app.windows[0].activeTab;
if (!activeTab.url().includes(youtubeUrl) && !activeTab.url().includes(twitchUrl)) {
  for (const [winIndex, win] of app.windows().entries()) {
    const tabIndex = win
      .tabs()
      .findIndex(tab => tab.url().includes(youtubeUrl) || tab.url().includes(twitchUrl));

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
  try {
    // YouTube
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
  } catch (e) {
    // Twitch
    document.querySelector('[data-a-target="chat-input"]').focus();
  }
};

const javascript = `(${focusTextInput.toString()})()`;

app.execute(app.windows[0].activeTab, { javascript });

