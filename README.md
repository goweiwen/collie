# Collie

Collie is a box-art and metadata scraper for your retro games. It is intended
to be run both directly from a wifi-enabled retro handheld or a computer with
the SD card mounted.

## Usage

1. Copy `collie` into your roms folder
2. Double-click to open it
3. A browser window will open for you to configure and start scraping

### macOS

If you see "cannot be opened because it is from an unidentified developer":
1. Right-click the file and select "Open"
2. Click "Open" in the dialog

### Windows

If you see "Windows protected your PC":
1. Click "More info"
2. Click "Run anyway"

### Command Line

For advanced usage, run `collie` from the terminal in your roms directory:
```
cd /path/to/roms
collie
```

Options:
- `-b, --bind <ADDR>` - Address to bind to (default: `127.0.0.1`)
- `-p, --port <PORT>` - Port to run on (default: `2435`)
- `--no-launch` - Don't open browser automatically

To allow access from other devices on your network:
```
collie --bind 0.0.0.0
```

After scraping, Collie will put the relevant information into gamelist.xml in
each of the console folders. By default, images will be scraped into the `Imgs`
folder.

Supported scrapers:
- ScreenScraper.fr
- TheGamesDB.net
- GameFAQs archive @ endangeredsoft.org
