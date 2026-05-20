# Tunedesk

## DISCLAIMER
Tunedesk is an IPTV client only and does not provide any links or content itself.

## What is it?
Tunedesk is an IPTV client for Windows and Linux (Mac should work in theory but isn't tested). It is NOT an IPTV provider.

## Why
I was having a hard time finding a desktop client for IPTV that I liked so I made one myself.

## Features:
- Import your channels from xtream links (M3U link should work but hasn't been tested)
- Cached data for faster response
- Search channels across profiles

## Install

### Windows
Download the exe from the [releases](https://github.com/NotNoss/Tunedesk/releases/tag/1.0.0). The exe is not signed because when I was looking it up, it seemed like a lot to get it signed. Maybe one day, feel free to open an issue if you need help getting it installed.

### Ubuntu/Debian
Download the tunedesk.deb from the [releases](https://github.com/NotNoss/Tunedesk/releases/tag/1.0.0)
```
sudo dpkg -i ./tunedesk.deb
```

### Arch Linux
Replace yay with your AUR helper
```
yay -S tunedesk
```

### Manual build
```
git clone https://github.com/NotNoss/Tunedesk.git
cd ./Tunedesk
pnpm tauri build --bundles {package}
```
