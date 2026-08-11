# Changelog

## Next
- Remove macOS 10.15 support
- Update many dependencies
- Fix "electron.app.hide is not a function" error when closing app window on Windows/Linux

## 1.1.0 - 2026 Aug 8
- Add "Stop" menu item
- Fix volume shortcuts
- Fix pause icon not updating when audio playback errors
- Fix playtime saving

## 1.0.0 - 2026 Mar 10
- Add lyrics support
- Add "Show in Playlist" context menu item
- Remember queue on restart
- Keep scroll position when opening play history
- Make filtering case-insensitive
- Mobile: Add track streaming search links for Spotify and YouTube Music
- Mobile: Fast playlist loading
- Mobile: Add full-featured filtering like the desktop app has

## 0.21.0 - 2026 Map 6
- Add a basic Android app. It only lets you browse a Ferrum Library.json file. No playback or syncing yet.

## 0.20.1 - 2026 Feb 11
- Fix playlist dragging
- Fix playlist description placeholder colour

## 0.20.0 - 2025 Oct 4
- Add audio visualiser (thanks to help from @zachwinter)
- Add filtering for specific fields in search, like `artist:apashe` or `album:"mirai sekai"`. You can also right-click a column header to add a filter. Filtering by artist also finds featured artists, remixers etc in the song title.
- Add genre autocomplete
- Make macOS media key permission request non-intrusive
- Improve error and crashing behaviour
- 2x faster library loading at startup
- Slightly faster scroll performance
- Fix error when removing last column

## 0.19.9 - 2025 Jul 4
- Fix column size not updating when resizing window
- Fix column sorting not working

## 0.19.8 - 2025 Jun 26
- Fix scrolling not happening when using arrow keys
- Fix up/down arrow keys selecting incorrectly after filtering
- Fix file errors not clickable

## 0.19.7 - 2025 Jun 6
- Fix cover artwork flickering

## 0.19.6 - 2025 Jun 6
- Fix some cover artworks not loading

## 0.19.5 - 2025 Jun 6
- Fix playing indicator not showing
- Fix column sorting indicator not showing

## 0.19.4 - 2025 Jun 4
- Faster scroll performance

## 0.19.3 - 2024 Nov 26
- Fixes rare .ogg metadata corruption (https://github.com/Serial-ATA/lofty-rs/issues/469)

## 0.19.2 - 2024 Oct 26
- Fix queue panel deleting/moving incorrect tracks
- Fix queue not refreshing after toggling history

## 0.19.1 - 2024 Oct 25
- Fix folder playlists not closing
- Fix outline glitch

## 0.19.0 - 2024 Oct 8
- Drop support for Windows 7 and 8
- Drop support for macOS 10.13 and 10.14
- Add playback history to queue panel
- Add ability to rearrange, hide & show columns
- Add columns: Image, Album Artist, Composer, Grouping, BPM
- Make cover artworks load fast in the queue panel by caching them
- Preserve "Up next" when starting a new playback
- Add "Clear" button to "Up next" in queue panel
- Redesign queue panel
- Add queue panel slide-out transition
- Add `Ctrl+Tab` and `Ctrl+Shift+Tab` to select next/previous playlist
- Add `Back` and `Forward` shortcuts
- Selection no longer gets cleared when the page updates
- Show playlist folder tracks in order
- Make time & volume sliders look ok
- Small design updates to tracklist header & filter textbox
- Respect user's locale time
- Allow pressing play to start the current playlist
- Make sidebar not focusable by mouse
- Make sorting 3-4x faster
- Fix covers updates sometimes not being reflected in the queue panel
- Fix dragging tracks to up next/autoplay queue boundary
- Fix button edge not clickable due to zoom
- Fix dragging of covers in queue
- Make draglines stable during scroll
- Fix importing of playlist likes/dislikes from Apple Music. Any previously imported likes/dislikes are lost.

## 0.18.0 - 2024 Jun 7
- Add arm64 support for macOS and Windows
- Add View menu bar checkbox for album track grouping
- Use album track grouping when sorting by album, date added, comments, genre, year & artist
- Fix menu bar checkboxes

## 0.17.3 - 2024 Apr 30
- Sub-sort albums by track number when sorting by album (album track grouping)
- Add `Escape` shortcut for de-focusing the sidebar and filter
- Fix rare error with updating artwork
- Fix importing of iTunes playlist likes. These aren't visible in the app, but they do get imported. If you previously imported playlists, the likes got lost in Ferrum.

## 0.17.2 - 2024 Jan 22
- Fix wrong songs being removed from queue when multiple are selected
- Fix error when moving playlist into empty folder
- Improve accessibility
- Refine modal design
- Slightly faster image loading

## 0.17.1 - 2023 Aug 7
- Fix filtering only working for first words

## 0.17.0 - 2023 Jul 27
- Add `Ctrl+K` quick playlist search (macOS: `Cmd+K`)
- Add playlist reordering
- Sidebar selection improvements:
	- Add left arrow key shortcut for going to the parent playlist folder
	- Add alt+up/down shortcut
	- Fix up arrow selecting siblings only
	- Fix inability to move playlists into deeper levels

## 0.16.12 - 2023 Jun 14
- Improve filtering by perofrming unicode normalization

## 0.16.11 - 2023 Apr 30
- Fix shuffle not staying on after starting new queue

## 0.16.10 - 2023 Apr 12
- Fix slider dragging

## 0.16.9 - 2023 Apr 12
- Fix error loading library
- Fix errors causing crashes, show error messages instead

## 0.16.8 - 2023 Apr 12
- Fix play time sometimes double counted
- Fix play time timestamp not updated after pausing
- Fix button click transition

## 0.16.7 - 2023 Apr 6
- Fix blank screen

## 0.16.6 - 2023 Apr 6
- Prevent focus when clicking the player bar
- Don't count skips from clicking "Previous"
- Improve text selection in modals
- Change "Reveal in File Manager" shortcut to `Shift+Ctrl+R` (macOS: `Shift+Cmd+R`)
- Clicking "Previous" now replays the first track instead of stopping
- Fix plays and skips not counted for last track in queue
- Fix playing indicator staying after playback stops
- Fix playing track not included when repeating

## 0.16.5 - 2022 Jan 19
- Fix queue rearranging reversing tracks
- Fix error when rearranging many tracks in the queue

## 0.16.4 - 2022 Jan 7
- Fix media key shortcuts

## 0.16.3 - 2022 Nov 7
- Improve focus behavior

## 0.16.2 - 2022 Oct 23
- Hide app when window is closed on macOS
- Fix unnecessary audio reloading

## 0.16.1 - 2022 Oct 21
- Fix `Enter` not working Track Info window

## 0.16.0 - 2022 Oct 21
- Add drag-and-drop, selection and context menus in queue panel
- Add shuffle feature
- Add repeat feature
- Add warning when adding duplicate songs to a playlist
- Add `Playback > Next` and `Playback > Previous` menus, with shortcuts
- Add support for deleting playlists
- Rewritten iTunes import, should be much more reliabile
- iTunes import no longer overwrites the existing library
- Scroll sidebar to opened playlist
- Improve queue panel performance
- Don't clear selection unnecessarily
- Reload file when updating the currently playing song's info
- Require focus when using Song menubar
- Improve loading time of artwork in Track Info modal
- Fix `Song > Delete from Library` menu
- Fix incorrect handling of macOS `Ctrl+click` context menu
- Fix sidebar icons showing over dialogs
- Fix playlist liked/disliked types
- Fix page playlist info not refreshing when edited
- Fix tracks staying in queue after being deleted from library

## 0.15.0 - 2022 Sep 22
- Support pasting track artworks
- Add `Space` shortcut for picking track artwork

## 0.14.4 - 2022 Sep 1
- Fix adding tracks to playlists

## 0.14.3 - 2022 Sep 1
- Fix context menus
- Fix dragging of currently playing artwork

## 0.14.2 - 2022 Aug 31
- Fix context menus

## 0.14.1 - 2022 Aug 31
- Fix app launch error
- Fix app exit when launch fails

## 0.14.0 - 2022 Aug 31
- Add playlist editing
- Add context menu and drag-and-drop to currently playing track
- Restrict `Enter` shortcut when no tracks are selected
- Fix saving of updated track artworks
- Thinner sort column header font
- Fix plays column being too narrow
- Only show `Remove from playlist` context menu in playlists

## 0.13.1 - 2022 Jul 28
- Show scrollbar only when needed

## 0.13.0 - 2022 Jul 26
- Ability to create playlist folders
- Support moving playlists between folders
- You can now open a playlist folder and see all it's tracks
- Add songs to playlists using drag-and-drop
- Add `Up`/`Down` sidebar shortcuts
- Support for .opus files with multiple pictures
- Support showing and removing specific pictures for tracks with multiple pictures
- Remember opened sidebar folders when the app relaunches
- Save changes when going to next/previous track info window
- Update button, filter and sidebar design
- Trap focus in modals
- Store electron data in new `space.kasper.ferrum` app data folder

## 0.12.0 - 2022 Jul 4
- Add support for .opus files

## 0.11.0 - 2022 Jun 5
- Update logo to macOS 11 style
- Clear selection when clicking on empty rows

## 0.10.0 - 2021 Dec 21
- Add support for importing .m4a files
- Add `File > Import...` menu
- Only show one dialog at once

## 0.9.1 - 2021 Dec 15
- Fix "Get Info" shortcut

## 0.9.0 - 2021 Dec 15
- Add ability to delete tracks from library
- Add `Ctrl+U` (macOS: `Cmd+U`) and `View > Toggle Queue` to show/hide the queue
- Add support for changing track artworks using double-click and drag-and-drop
- Add `Ctrl+]` and `Ctrl+]` (macOS: `Cmd+]` and `Cmd+]`) for opening the next and previous track in the "Get Info" window
- Add metadata editing of bpm, track number, total tracks, disc number and total discs
- Add more items to the Song menu
- Add "Reveal in File Manager" menu item
- Add `Escape` shortcut for deselecting tracks
- Add support selecting tracks via `Ctrl+A` (macOS: `Cmd+A`), `Shift+Up/Down`, `Alt+Up/Down`, `Shift+Alt+Up/Down`
- Expand the titlebar drag region a bit
- Fix memory leak from opening queue
- Improve behavior of shift-selecting tracks
- Make sure newly selected tracks are visible by scrolling to them
- Improve media key handler registration

## 0.8.1 - 2021 Aug 9
- Fix error after updating metadata when filter is on

## 0.8.0 - 2021 Aug 8
- Add support for Linux (deb/rpm) and Windows (exe)
- Add volume controls
- Add ability to create playlists
- Add track metadata editing
- Add playlist header
- Glowy fancy sidebar playlist highlight
- Some smaller design improvements
- Improve scrolling performance
- Make `space` shortcut work more often
- The `Ferrum/Artworks` folder is no longer necessary and can be safely deleted
- Breaking change: Make `library.json` track artist field required. Previously, this field was left out if it was empty

## 0.7.0 - 2021 May 14
- Add queue panel
- Add multi-selection support
- Add support for dragging/rearranging tracks
- Add ability to add tracks to playlists (via context menu)
- Ability to remove tracks from playlists via backspace or context menu (for now only possible when no filter or custom sorting is used)
- Change date added to show date instead of number timestamp
- Fix selection not clearing when page changes
- Fix reading metadata for some m4a files
- Fix row colors changing as you scroll
- Fix sorting icon not updating when switching to sorting by "index"
- Fix error popups not always triggering
- Fix "currently playing" icon showing incorrectly

## 0.6.1 - 2021 Apr 6
- Fix media keys shortcuts

## 0.6.0 - 2021 Apr 6
- Added filter feature
- Added artworks to player
- Design changes
- Fixed accessibility permisions flow

## 0.5.0 - 2021 Mar 24
- Added "Add to Queue" context menu item
- Added "Play Next" context menu item
- Make player prettier
- Added "currently playing" indicator icon in track list
- Added enter shortcut for playing selected track
- Added support for selecting tracks with up/down arrow keys

## 0.4.1 - 2021 Feb 19
- Fixed year metadata not being imported for mp3s using id3v2.4

## 0.4.0 - 2021 Feb 19
- Added support for importing mp3s
- Added support for exclusively taking over media keys
- Added mediaSession support
- Made library.json track `name` field required
- Changed the number types of some library.json track fields
- Fixed tracklist not refreshing after things change

## 0.3.0 - 2021 Feb 13
- Converted some code to Rust
- Errors will now show up as popups
- Fixed error when opening a smaller playlist than what's currently visible
- Fixed some play times being incorrectly logged with `null` values
- Fixed play times often being logged with too big durations. For play times already logged incorrectly, run the script `src/scripts/fix_play_time.js` to fix those
- Fixed `tracks` and `playlists` properties of playlists missing during iTunes import

## 0.2.0 - 2021 Jan 3
- Added currently playing artist and title to player
- Added sorting
- Added macOS traffic light to window
- Removed shortcut for importing iTunes library
- Fixed album names not showing up
- Fixed play time sometimes being incorrectly logged
- Fixed empty tracklist when opening playlist while scrolled down

## 0.1.0 - 2021 Dec 31
- Initial release
