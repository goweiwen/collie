# Collie

Collie is a box-art and metadata scraper for your retro games. It is intended
to be run both directly from a wifi-enabled retro handheld or a computer with
the SD card mounted.

## Usage

Run `collie` in your roms directory, or specify it like so:
```
collie --port 2435 --roms-path /mnt/SDCARD/Roms
```

Collie will launch a web server at the specified port for you to configure your
scraper settings, start the scraper, and monitor the progress.

After scraping, Collie will put the relevant information into gamelist.xml in
each of the console folders. By default, images will be scraped into the `Imgs`
folder.

Supported scrapers:
- ScreenScraper.fr
- TheGamesDB.net
- GameFAQs archive @ endangeredsoft.org
