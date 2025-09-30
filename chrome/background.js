// Background script for Lynx Chrome extension
const LYNX_API_BASE = 'http://lynx:3000/api'; // Default Lynx server URL

// Store current tab's go links
let currentTabGoLinks = [];
let currentTabUrl = '';

// Listen for tab updates
chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.status === 'complete' && tab.url) {
    checkGoLinksForUrl(tab.url, tabId);
  }
});

// Listen for tab activation
chrome.tabs.onActivated.addListener((activeInfo) => {
  chrome.tabs.get(activeInfo.tabId, (tab) => {
    if (tab.url) {
      checkGoLinksForUrl(tab.url, activeInfo.tabId);
    }
  });
});

// Check if there are go links for the current URL
async function checkGoLinksForUrl(url, tabId) {
  try {
    // Skip chrome:// URLs and other non-http(s) URLs
    if (!url || !url.startsWith('http')) {
      currentTabGoLinks = [];
      updateIcon(false, tabId);
      await chrome.storage.local.set({
        [`goLinks_${tabId}`]: [],
        [`currentUrl_${tabId}`]: url
      });
      return;
    }

    currentTabUrl = url;
    
    // Get Lynx server URL from storage (allow user to configure)
    const result = await chrome.storage.sync.get(['lynxServerUrl']);
    const serverUrl = result.lynxServerUrl || LYNX_API_BASE;
    
    // Query the reverse lookup API
    const response = await fetch(`${serverUrl}/links/reverse?target=${encodeURIComponent(url)}`);
    
    if (response.ok) {
      const goLinks = await response.json();
      currentTabGoLinks = goLinks;
      
      // Update icon based on whether go links exist
      updateIcon(goLinks.length > 0, tabId);
      
      // Store the go links for the popup
      await chrome.storage.local.set({
        [`goLinks_${tabId}`]: goLinks,
        [`currentUrl_${tabId}`]: url
      });
    } else {
      // No go links found or API error
      currentTabGoLinks = [];
      updateIcon(false, tabId);
      
      await chrome.storage.local.set({
        [`goLinks_${tabId}`]: [],
        [`currentUrl_${tabId}`]: url
      });
    }
  } catch (error) {
    console.error('Error checking go links for URL:', url, error);
    currentTabGoLinks = [];
    updateIcon(false, tabId);
    
    // Store empty state
    await chrome.storage.local.set({
      [`goLinks_${tabId}`]: [],
      [`currentUrl_${tabId}`]: url
    });
  }
}

// Update the extension icon
function updateIcon(hasGoLinks, tabId) {
  const iconPath = hasGoLinks ? {
    "16": "icons/lynx-16.png",
    "32": "icons/lynx-32.png",
    "48": "icons/lynx-48.png",
    "128": "icons/lynx-128.png"
  } : {
    "16": "icons/lynx-16-grey.png",
    "32": "icons/lynx-32-grey.png",
    "48": "icons/lynx-48-grey.png",
    "128": "icons/lynx-128-grey.png"
  };
  
  chrome.action.setIcon({
    path: iconPath,
    tabId: tabId
  });
  
  // Update title
  const title = hasGoLinks 
    ? `Lynx - ${currentTabGoLinks.length} go link(s) found`
    : 'Lynx - No go links found';
    
  chrome.action.setTitle({
    title: title,
    tabId: tabId
  });
}

// Handle messages from popup
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === 'getCurrentTabData') {
    chrome.tabs.query({active: true, currentWindow: true}, async (tabs) => {
      if (tabs[0]) {
        const tabId = tabs[0].id;
        const result = await chrome.storage.local.get([
          `goLinks_${tabId}`,
          `currentUrl_${tabId}`
        ]);
        
        sendResponse({
          goLinks: result[`goLinks_${tabId}`] || [],
          currentUrl: result[`currentUrl_${tabId}`] || tabs[0].url
        });
      }
    });
    return true; // Keep message channel open for async response
  }
  
  if (request.action === 'createGoLink') {
    createGoLink(request.data)
      .then(result => sendResponse({success: true, data: result}))
      .catch(error => sendResponse({success: false, error: error.message}));
    return true; // Keep message channel open for async response
  }
});

// Create a new go link
async function createGoLink(linkData) {
  try {
    const result = await chrome.storage.sync.get(['lynxServerUrl']);
    const serverUrl = result.lynxServerUrl || LYNX_API_BASE;
    
    const response = await fetch(`${serverUrl}/links`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        host: 'go', // Fixed to 'go' as per requirements
        source: linkData.source,
        target: linkData.target
      })
    });
    
    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(errorData.error || 'Failed to create go link');
    }
    
    const newLink = await response.json();
    
    // Refresh the current tab's go links
    chrome.tabs.query({active: true, currentWindow: true}, (tabs) => {
      if (tabs[0]) {
        checkGoLinksForUrl(tabs[0].url, tabs[0].id);
      }
    });
    
    return newLink;
  } catch (error) {
    console.error('Error creating go link:', error);
    throw error;
  }
}

// Initialize when extension starts
chrome.runtime.onStartup.addListener(() => {
  // Check current active tab
  chrome.tabs.query({active: true, currentWindow: true}, (tabs) => {
    if (tabs[0] && tabs[0].url) {
      checkGoLinksForUrl(tabs[0].url, tabs[0].id);
    }
  });
});

// Also initialize when extension is installed/enabled
chrome.runtime.onInstalled.addListener(() => {
  // Check current active tab
  chrome.tabs.query({active: true, currentWindow: true}, (tabs) => {
    if (tabs[0] && tabs[0].url) {
      checkGoLinksForUrl(tabs[0].url, tabs[0].id);
    }
  });
});
