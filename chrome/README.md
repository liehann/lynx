# Lynx Chrome Extension

A Chrome extension for managing Lynx go links directly from your browser.

## Features

- **Smart Icon**: Shows color logo when go links exist for the current page, greyscale when none exist
- **Quick Access**: Click the toolbar icon to see existing go links or add new ones
- **Copy Links**: Easily copy go links to clipboard with one click
- **Auto-Detection**: Automatically checks for go links when you navigate to new pages
- **Go Domain Only**: Supports only the "go" domain as specified

## Installation

### Development Installation

1. Open Chrome and navigate to `chrome://extensions/`
2. Enable "Developer mode" in the top right corner
3. Click "Load unpacked" and select the `chrome` directory from your Lynx project
4. The Lynx extension should now appear in your toolbar

### Configuration

1. Click the Lynx icon in your toolbar
2. Click "Configure Lynx Server" at the bottom of the popup
3. Enter your Lynx server URL (default: `http://lynx:3000/api`)

## Usage

### Viewing Existing Go Links

1. Navigate to any webpage
2. Click the Lynx icon in your toolbar
3. If go links exist for the current URL, they will be displayed
4. Click "Copy" next to any go link to copy it to your clipboard

### Adding New Go Links

1. Navigate to the webpage you want to create a go link for
2. Click the Lynx icon in your toolbar
3. In the "Add New Go Link" section, enter your desired short name
4. Click the "+" button to create the link
5. The go link will be created as `go/yourshortname` → current URL

### Icon States

- **Color Icon**: One or more go links exist for the current page
- **Greyscale Icon**: No go links exist for the current page

## API Integration

The extension communicates with your Lynx server using these endpoints:

- `GET /api/links/reverse?target=<url>` - Find go links by target URL
- `POST /api/links` - Create new go links

## File Structure

```
chrome/
├── manifest.json          # Extension manifest
├── background.js          # Background service worker
├── popup.html            # Popup interface
├── popup.js              # Popup logic
├── content.js            # Content script for URL detection
├── icons/                # Extension icons
│   ├── lynx-16.png       # Color icons
│   ├── lynx-32.png
│   ├── lynx-48.png
│   ├── lynx-128.png
│   ├── lynx-16-grey.png  # Greyscale icons
│   ├── lynx-32-grey.png
│   ├── lynx-48-grey.png
│   └── lynx-128-grey.png
└── README.md             # This file
```

## Permissions

The extension requires these permissions:

- `activeTab` - To access the current tab's URL
- `tabs` - To monitor tab changes and updates
- `storage` - To store configuration and cache data
- `http://*/*` and `https://*/*` - To make API requests to your Lynx server

## Troubleshooting

### Extension Not Working

1. Check that your Lynx server is running and accessible
2. Verify the server URL in the extension settings
3. Check the browser console for error messages
4. Ensure the Lynx server has the reverse lookup API endpoint

### No Go Links Showing

1. Verify that go links exist for the current URL in your Lynx database
2. Check that the target URL in the database exactly matches the current page URL
3. Try refreshing the page and clicking the extension icon again

### Cannot Create Go Links

1. Check that your Lynx server is accepting POST requests to `/api/links`
2. Verify that the go link name doesn't already exist
3. Ensure the go link name contains only valid characters (letters, numbers, hyphens, underscores, forward slashes)

## Development

To modify the extension:

1. Make changes to the files in the `chrome/` directory
2. Go to `chrome://extensions/`
3. Click the refresh icon for the Lynx extension
4. Test your changes

## Security Notes

- The extension only communicates with your configured Lynx server
- All API requests are made over HTTP/HTTPS as configured
- No sensitive data is stored locally beyond the server URL configuration
