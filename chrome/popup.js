// Popup script for Lynx Chrome extension

let currentUrl = '';
let goLinks = [];

// Initialize popup when DOM is loaded
document.addEventListener('DOMContentLoaded', async () => {
  await loadCurrentTabData();
  setupEventListeners();
});

// Load current tab data from background script
async function loadCurrentTabData() {
  try {
    const response = await new Promise((resolve) => {
      chrome.runtime.sendMessage({action: 'getCurrentTabData'}, resolve);
    });
    
    currentUrl = response.currentUrl || '';
    goLinks = response.goLinks || [];
    
    updateUI();
  } catch (error) {
    console.error('Error loading tab data:', error);
    showError('Failed to load go links data');
  }
}

// Update the popup UI
function updateUI() {
  const loadingState = document.getElementById('loadingState');
  const content = document.getElementById('content');
  const currentUrlElement = document.getElementById('currentUrl');
  const goLinksSection = document.getElementById('goLinksSection');
  const goLinksList = document.getElementById('goLinksList');
  
  // Hide loading, show content
  loadingState.style.display = 'none';
  content.style.display = 'block';
  
  // Update current URL display
  currentUrlElement.textContent = currentUrl;
  
  // Update go links list
  if (goLinks.length > 0) {
    goLinksSection.style.display = 'block';
    goLinksList.innerHTML = '';
    
    goLinks.forEach(link => {
      const linkElement = createGoLinkElement(link);
      goLinksList.appendChild(linkElement);
    });
  } else {
    goLinksSection.style.display = 'block';
    goLinksList.innerHTML = '<div class="no-links">No go links found for this URL</div>';
  }
}

// Create a go link element
function createGoLinkElement(link) {
  const div = document.createElement('div');
  div.className = 'go-link-item';
  
  const goLinkText = `go/${link.source}`;
  
  div.innerHTML = `
    <span class="go-link-text">${escapeHtml(goLinkText)}</span>
    <button class="copy-btn" data-link="${escapeHtml(goLinkText)}">Copy</button>
  `;
  
  // Add click handler for copy button
  const copyBtn = div.querySelector('.copy-btn');
  copyBtn.addEventListener('click', () => copyToClipboard(goLinkText));
  
  return div;
}

// Setup event listeners
function setupEventListeners() {
  const addBtn = document.getElementById('addBtn');
  const sourceInput = document.getElementById('sourceInput');
  const settingsLink = document.getElementById('settingsLink');
  
  addBtn.addEventListener('click', handleAddGoLink);
  sourceInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
      handleAddGoLink();
    }
  });
  
  settingsLink.addEventListener('click', (e) => {
    e.preventDefault();
    showSettings();
  });
  
  // Focus the input field
  sourceInput.focus();
}

// Handle adding a new go link
async function handleAddGoLink() {
  const sourceInput = document.getElementById('sourceInput');
  const addBtn = document.getElementById('addBtn');
  const source = sourceInput.value.trim();
  
  if (!source) {
    showError('Please enter a go link name');
    return;
  }
  
  // Validate source format (simple validation)
  if (!/^[a-zA-Z0-9\-_\/]+$/.test(source)) {
    showError('Go link name can only contain letters, numbers, hyphens, underscores, and forward slashes');
    return;
  }
  
  // Ensure source starts with /
  const formattedSource = source.startsWith('/') ? source : `/${source}`;
  
  try {
    addBtn.disabled = true;
    addBtn.textContent = '...';
    clearMessages();
    
    const response = await new Promise((resolve) => {
      chrome.runtime.sendMessage({
        action: 'createGoLink',
        data: {
          source: formattedSource,
          target: currentUrl
        }
      }, resolve);
    });
    
    if (response.success) {
      showSuccess(`Go link created: go${formattedSource}`);
      sourceInput.value = '';
      
      // Reload the go links
      setTimeout(async () => {
        await loadCurrentTabData();
      }, 1000);
    } else {
      showError(response.error || 'Failed to create go link');
    }
  } catch (error) {
    console.error('Error creating go link:', error);
    showError('Failed to create go link');
  } finally {
    addBtn.disabled = false;
    addBtn.textContent = '+';
  }
}

// Copy text to clipboard
async function copyToClipboard(text) {
  try {
    await navigator.clipboard.writeText(text);
    
    // Show temporary feedback
    const copyBtns = document.querySelectorAll('.copy-btn');
    copyBtns.forEach(btn => {
      if (btn.dataset.link === text) {
        const originalText = btn.textContent;
        btn.textContent = 'Copied!';
        btn.style.background = '#10b981';
        
        setTimeout(() => {
          btn.textContent = originalText;
          btn.style.background = '#3b82f6';
        }, 1500);
      }
    });
  } catch (error) {
    console.error('Failed to copy to clipboard:', error);
    showError('Failed to copy to clipboard');
  }
}

// Show error message
function showError(message) {
  const errorElement = document.getElementById('errorMessage');
  errorElement.innerHTML = `<div class="error">${escapeHtml(message)}</div>`;
  
  // Clear after 5 seconds
  setTimeout(() => {
    errorElement.innerHTML = '';
  }, 5000);
}

// Show success message
function showSuccess(message) {
  const successElement = document.getElementById('successMessage');
  successElement.innerHTML = `<div class="success">${escapeHtml(message)}</div>`;
  
  // Clear after 3 seconds
  setTimeout(() => {
    successElement.innerHTML = '';
  }, 3000);
}

// Clear all messages
function clearMessages() {
  document.getElementById('errorMessage').innerHTML = '';
  document.getElementById('successMessage').innerHTML = '';
}

// Show settings dialog
function showSettings() {
  const serverUrl = prompt('Enter Lynx server URL (e.g., http://lynx:3000/api):', 'http://lynx:3000/api');
  
  if (serverUrl !== null) {
    chrome.storage.sync.set({lynxServerUrl: serverUrl.trim()}, () => {
      showSuccess('Server URL updated');
    });
  }
}

// Escape HTML to prevent XSS
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}
