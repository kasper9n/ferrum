<div align="center">
	<img src="assets/Logo%201024.png" width="80">
</div>
<h3 align="center">Ferrum</h3>
<p align="center">
	Music library and player
	<br/>
	<a href="https://ferrum.kasper.space"><b>Download for Mac, Linux, Windows or Android</b></a>
</p>

<br/>

https://github.com/user-attachments/assets/b73cdcf8-8abb-4d43-a5eb-582cad7e3075

## Features
- Playlists & folders
- Filtering
- Queue
- Metadata editing
- Fast. Navigation, sorting & filtering should be instant even for libraries with 100 000 tracks
- Quick playlist navigation with <kbd>Cmd</kbd><kbd>K</kbd> or <kbd>Ctrl</kbd><kbd>K</kbd>

## Android

The Android app is not useful yet, unless you just want you browse your library. There's no playback or syncing. You have to manually copy over your Library.json file.

<a href="https://apps.obtainium.imranr.dev/redirect?r=obtainium://app/%7B%22id%22%3A%22space.kasper.ferrum%22%2C%22url%22%3A%22https%3A%2F%2Fgithub.com%2Fkasper9n%2Fferrum%22%2C%22author%22%3A%22kasper9n%22%2C%22name%22%3A%22Ferrum%22%2C%22preferredApkIndex%22%3A0%2C%22additionalSettings%22%3A%22%7B%5C%22includePrereleases%5C%22%3Afalse%2C%5C%22fallbackToOlderReleases%5C%22%3Atrue%2C%5C%22filterReleaseTitlesByRegEx%5C%22%3A%5C%22%5C%22%2C%5C%22filterReleaseNotesByRegEx%5C%22%3A%5C%22%5C%22%2C%5C%22verifyLatestTag%5C%22%3Afalse%2C%5C%22sortMethodChoice%5C%22%3A%5C%22date%5C%22%2C%5C%22useLatestAssetDateAsReleaseDate%5C%22%3Afalse%2C%5C%22releaseTitleAsVersion%5C%22%3Afalse%2C%5C%22trackOnly%5C%22%3Afalse%2C%5C%22versionExtractionRegEx%5C%22%3A%5C%22%5C%22%2C%5C%22matchGroupToUse%5C%22%3A%5C%22%5C%22%2C%5C%22versionDetection%5C%22%3Atrue%2C%5C%22releaseDateAsVersion%5C%22%3Afalse%2C%5C%22useVersionCodeAsOSVersion%5C%22%3Afalse%2C%5C%22apkFilterRegEx%5C%22%3A%5C%22%5C%22%2C%5C%22invertAPKFilter%5C%22%3Afalse%2C%5C%22autoApkFilterByArch%5C%22%3Atrue%2C%5C%22appName%5C%22%3A%5C%22%5C%22%2C%5C%22appAuthor%5C%22%3A%5C%22%5C%22%2C%5C%22shizukuPretendToBeGooglePlay%5C%22%3Afalse%2C%5C%22allowInsecure%5C%22%3Afalse%2C%5C%22exemptFromBackgroundUpdates%5C%22%3Afalse%2C%5C%22skipUpdateNotifications%5C%22%3Afalse%2C%5C%22about%5C%22%3A%5C%22%5C%22%2C%5C%22refreshBeforeDownload%5C%22%3Afalse%2C%5C%22includeZips%5C%22%3Afalse%2C%5C%22zippedApkFilterRegEx%5C%22%3A%5C%22%5C%22%7D%22%2C%22overrideSource%22%3A%22GitHub%22%7D"><img alt="Get it on Obtainium" height="80" src="https://github.com/user-attachments/assets/713d71c5-3dec-4ec4-a3f2-8d28d025a9c6"></a>

## Dev instructions

### Get started

1. Install Node.js
2. Install Rust
3. Run `npm install`

### Structure

App (Electron)
- Source: `src/electron.js` and `src/electron/`
- Build output: `build/app/`

Web (the frontend)
- Source code: napi and `src/`
- Build output: `build/web/`

Napi (the native Rust backend)
- Source code: `src-native/`
- Build output: `ferrum-addon/addon.node`

### Commands
- `npm run dev`: Start app in dev mode
- `npm run dev-release`: Start app in dev mode (optimized)
- `npm run build`: Build
- `npm run lint`: Lint
- `npm run format`: Format

### Publish new version
1. Run `npm run check`
2. Update `CHANGELOG.md`
3. Run `npm version --no-git-tag-version <version>` to bump the version number
4. Create a git tag in the format `v#.#.#`
5. Add release notes to the generated GitHub release and publish it
