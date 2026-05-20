# Tunedesk

## DISCLAIMER
Tunedesk is an IPTV client only and does not provide any links or content itself.

## Features:
- Import your channels from M3U links or xtream
- Cached data for faster response
- Search channels across profiles

## Install

### Ubuntu/Debian
Download the tunedesk.deb from the releases
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
