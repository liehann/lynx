// Content script for Lynx Chrome extension
// This script runs on every page to detect URL changes

// Listen for URL changes (for single-page applications)
let currentUrl = location.href;

// Observer for URL changes in SPAs
const observer = new MutationObserver(() => {
  if (location.href !== currentUrl) {
    currentUrl = location.href;
    // Notify background script of URL change
    chrome.runtime.sendMessage({
      action: 'urlChanged',
      url: currentUrl
    });
  }
});

// Start observing
observer.observe(document, {
  subtree: true,
  childList: true
});

// Also listen for popstate events (back/forward navigation)
window.addEventListener('popstate', () => {
  if (location.href !== currentUrl) {
    currentUrl = location.href;
    chrome.runtime.sendMessage({
      action: 'urlChanged',
      url: currentUrl
    });
  }
});

// Initial URL report
chrome.runtime.sendMessage({
  action: 'urlChanged',
  url: currentUrl
});
