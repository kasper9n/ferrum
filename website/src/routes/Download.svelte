<script lang="ts">
	import { fly, scale } from 'svelte/transition'
	import ButtonPopup from './ButtonPopup.svelte'
	import Bowser from 'bowser'
	import { onMount } from 'svelte'
	import type { RestEndpointMethodTypes } from '@octokit/plugin-rest-endpoint-methods'

	type Version = {
		os: string
		arch: string
		ending: string
	}
	const macos_arm: Version = { os: 'macOS', arch: 'arm64', ending: '-mac-arm64.dmg' }
	const macos_x64: Version = { os: 'macOS', arch: 'x64', ending: '-mac-x64.dmg' }
	const windows_x64: Version = { os: 'Windows', arch: 'x64', ending: '-win-x64.exe' }
	const windows_arm: Version = { os: 'Windows', arch: 'arm64', ending: '-win-arm64.exe' }
	const linux_deb: Version = { os: 'Linux', arch: '.deb', ending: '-linux-amd64.deb' }
	const linux_rpm: Version = { os: 'Linux', arch: '.rpm', ending: 'linux-x86_64.rpm' }
	const android_obtainium: Version = { os: 'Obtainium', arch: '', ending: '' }
	const android_arm64: Version = { os: 'Android', arch: 'arm64', ending: '-android-arm64.apk' }
	const android_x86_64: Version = { os: 'Android', arch: 'x64', ending: '-android-x86_64.apk' }

	const desktop_versions: Version[] = [
		macos_arm,
		macos_x64,
		windows_x64,
		windows_arm,
		linux_deb,
		linux_rpm,
	]
	const android_versions: Version[] = [android_obtainium, android_arm64, android_x86_64]

	const obtainium_url =
		'https://apps.obtainium.imranr.dev/redirect?r=obtainium://app/%7B%22id%22%3A%22space.kasper.ferrum%22%2C%22url%22%3A%22https%3A%2F%2Fgithub.com%2Fkasper9n%2Fferrum%22%2C%22author%22%3A%22kasper9n%22%2C%22name%22%3A%22Ferrum%22%2C%22preferredApkIndex%22%3A0%2C%22additionalSettings%22%3A%22%7B%5C%22includePrereleases%5C%22%3Afalse%2C%5C%22fallbackToOlderReleases%5C%22%3Atrue%2C%5C%22filterReleaseTitlesByRegEx%5C%22%3A%5C%22%5C%22%2C%5C%22filterReleaseNotesByRegEx%5C%22%3A%5C%22%5C%22%2C%5C%22verifyLatestTag%5C%22%3Afalse%2C%5C%22sortMethodChoice%5C%22%3A%5C%22date%5C%22%2C%5C%22useLatestAssetDateAsReleaseDate%5C%22%3Afalse%2C%5C%22releaseTitleAsVersion%5C%22%3Afalse%2C%5C%22trackOnly%5C%22%3Afalse%2C%5C%22versionExtractionRegEx%5C%22%3A%5C%22%5C%22%2C%5C%22matchGroupToUse%5C%22%3A%5C%22%5C%22%2C%5C%22versionDetection%5C%22%3Atrue%2C%5C%22releaseDateAsVersion%5C%22%3Afalse%2C%5C%22useVersionCodeAsOSVersion%5C%22%3Afalse%2C%5C%22apkFilterRegEx%5C%22%3A%5C%22%5C%22%2C%5C%22invertAPKFilter%5C%22%3Afalse%2C%5C%22autoApkFilterByArch%5C%22%3Atrue%2C%5C%22appName%5C%22%3A%5C%22%5C%22%2C%5C%22appAuthor%5C%22%3A%5C%22%5C%22%2C%5C%22shizukuPretendToBeGooglePlay%5C%22%3Afalse%2C%5C%22allowInsecure%5C%22%3Afalse%2C%5C%22exemptFromBackgroundUpdates%5C%22%3Afalse%2C%5C%22skipUpdateNotifications%5C%22%3Afalse%2C%5C%22about%5C%22%3A%5C%22%5C%22%2C%5C%22refreshBeforeDownload%5C%22%3Afalse%2C%5C%22includeZips%5C%22%3Afalse%2C%5C%22zippedApkFilterRegEx%5C%22%3A%5C%22%5C%22%7D%22%2C%22overrideSource%22%3A%22GitHub%22%7D'

	let suggested_desktop = windows_x64

	onMount(() => {
		const browser = Bowser.getParser(window.navigator.userAgent)
		const os_name = browser.getOSName()
		if (os_name === 'macOS' || os_name === 'iOS') {
			suggested_desktop = macos_arm
		} else if (os_name === 'Windows') {
			suggested_desktop = windows_x64
		} else if (os_name === 'Linux' || os_name === 'Chrome OS') {
			suggested_desktop = linux_deb
		}
	})

	let download_error = ''
	let loading = false
	type LatestRelease = RestEndpointMethodTypes['repos']['getLatestRelease']['response']['data']

	function get_asset(assets: LatestRelease['assets'], version: Version) {
		for (const asset of assets) {
			if (asset.name.toLowerCase().endsWith(version.ending)) return asset
		}
		return null
	}

	async function inner_download(version: Version) {
		download_error = ''
		if (version.os === 'Obtainium') {
			window.open(obtainium_url, '_blank', 'noopener,noreferrer')
			return
		}
		try {
			const response = await fetch(
				'https://api.github.com/repos/kasper9n/ferrum/releases/latest',
			)
			const json: LatestRelease = await response.json()
			if (response.status !== 200) {
				download_error = 'Could not download'
				console.error('Error response from GitHub', json)
				return
			}
			const asset = get_asset(json.assets, version)
			if (asset === null) {
				download_error = 'No file found'
				console.error('No file found')
				return
			}
			location.href = asset.browser_download_url
		} catch (error) {
			download_error = 'Network error'
			console.error(error)
		}
	}

	async function download(version: Version) {
		loading = true
		await inner_download(version)
		loading = false
	}
</script>

<!-- Hidden SVG sprite sheet for all logos -->
<svg style="display:none" aria-hidden="true">
	<!-- Tabler filled Apple -->
	<symbol id="icon-apple" viewBox="0 0 24 24" fill="currentColor">
		<path stroke="none" d="M0 0h24v24H0z" fill="none" />
		<path
			d="M15.079 5.999l.239 .012c1.43 .097 3.434 1.013 4.508 2.586a1 1 0 0 1 -.344 1.44c-.05 .028 -.372 .158 -.497 .217a4.15 4.15 0 0 0 -.722 .431c-.614 .461 -.948 1.009 -.942 1.694c.01 .885 .339 1.454 .907 1.846c.208 .143 .436 .253 .666 .33c.126 .043 .426 .116 .444 .122a1 1 0 0 1 .662 .942c0 2.621 -3.04 6.381 -5.286 6.381c-.79 0 -1.272 -.091 -1.983 -.315l-.098 -.031c-.463 -.146 -.702 -.192 -1.133 -.192c-.52 0 -.863 .06 -1.518 .237l-.197 .053c-.575 .153 -.964 .226 -1.5 .248c-2.749 0 -5.285 -5.093 -5.285 -9.072c0 -3.87 1.786 -6.92 5.286 -6.92c.297 0 .598 .045 .909 .128c.403 .107 .774 .26 1.296 .508c.787 .374 .948 .44 1.009 .44h.016c.03 -.003 .128 -.047 1.056 -.457c1.061 -.467 1.864 -.685 2.746 -.616l-.24 -.012z"
		/>
		<path d="M14 1a1 1 0 0 1 1 1a3 3 0 0 1 -3 3a1 1 0 0 1 -1 -1a3 3 0 0 1 3 -3z" />
	</symbol>
	<!-- Tabler filled Windows -->
	<symbol id="icon-windows" viewBox="0 0 24 24" fill="currentColor">
		<path stroke="none" d="M0 0h24v24H0z" fill="none" />
		<path
			d="M21 13v5c0 1.57 -1.248 2.832 -2.715 2.923l-.113 .003l-.042 .018a1 1 0 0 1 -.336 .056l-.118 -.008l-4.676 -.585v-7.407zm-10 0v7.157l-5.3 -.662c-1.514 -.151 -2.7 -1.383 -2.7 -2.895v-3.6zm0 -9.158v7.158h-8v-3.6c0 -1.454 1.096 -2.648 2.505 -2.87zm10 2.058v5.1h-8v-7.409l4.717 -.589c1.759 -.145 3.283 1.189 3.283 2.898"
		/>
	</symbol>
	<!-- Custom Tux -->
	<symbol id="icon-linux" viewBox="3 2 18 20" fill="currentColor">
		<path
			d="M6.09159 20.5316C7.0409 20.645 8.10753 21.2599 9.00016 21.3685C9.89759 21.4818 10.1753 20.7574 10.1753 20.7574C10.1753 20.7574 11.1852 20.5316 12.2469 20.5056C13.3096 20.4758 14.3156 20.7266 14.3156 20.7266C14.3156 20.7266 14.5107 21.1734 14.8749 21.3685C15.2391 21.5674 16.0231 21.5943 16.5256 21.0648C17.0291 20.5316 18.3724 19.8599 19.1267 19.44C19.8858 19.0191 19.7465 18.3773 19.2699 18.1822C18.7934 17.9872 18.4032 17.6797 18.433 17.0897C18.459 16.5046 18.0122 16.1145 18.0122 16.1145C18.0122 16.1145 18.4032 14.8269 18.0391 13.7603C17.6749 12.6986 16.4738 10.9911 15.5504 9.70743C14.6271 8.41988 15.4111 6.93343 14.5704 5.03377C13.7296 3.13126 11.5494 3.24371 10.3743 4.05463C9.19913 4.8656 9.55942 6.87668 9.6161 7.83085C9.67279 8.78017 9.64204 9.45948 9.53347 9.7036C9.4249 9.95148 8.66679 10.8518 8.1633 11.6061C7.66079 12.3642 7.29656 13.9295 6.92856 14.5752C6.56822 15.2171 6.81999 15.8022 6.81999 15.8022C6.81999 15.8022 6.56827 15.8887 6.36936 16.3096C6.17427 16.7256 5.78416 16.9245 5.08176 17.0601C4.38416 17.2032 4.38416 17.6529 4.55233 18.1564C4.72142 18.6589 4.55233 18.9404 4.35725 19.5823C4.16222 20.2241 5.13845 20.4191 6.09159 20.5316ZM15.9597 17.6038C16.4584 17.8219 17.1752 17.5182 17.3933 17.3001C17.6105 17.083 17.7642 16.7601 17.7642 16.7601C17.7642 16.7601 17.9823 16.8687 17.9602 17.2136C17.9371 17.5634 18.1101 18.062 18.4368 18.235C18.7635 18.407 19.2622 18.6472 19.0037 18.8874C18.7404 19.1276 17.2838 19.7138 16.8485 20.1711C16.4171 20.6256 15.8502 20.9975 15.5052 20.8879C15.1564 20.7794 14.8518 20.3028 15.0018 19.6052C15.1564 18.9105 15.2871 18.1485 15.265 17.7132C15.242 17.278 15.1564 16.6919 15.265 16.6054C15.3736 16.5198 15.5466 16.5611 15.5466 16.5611C15.5466 16.5611 15.46 17.3867 15.9597 17.6038ZM12.7235 5.84474C13.2039 5.84474 13.5912 6.32131 13.5912 6.90743C13.5912 7.32348 13.3961 7.68383 13.1107 7.8568C13.0387 7.82703 12.9637 7.79337 12.8811 7.75874C13.054 7.67325 13.1741 7.45514 13.1741 7.20337C13.1741 6.87383 12.9714 6.60286 12.7167 6.60286C12.4688 6.60286 12.2622 6.87377 12.2622 7.20337C12.2622 7.32354 12.292 7.4436 12.341 7.53777C12.1911 7.4772 12.0556 7.42537 11.9471 7.38405C11.8904 7.24085 11.8567 7.07943 11.8567 6.90748C11.8568 6.32137 12.243 5.84474 12.7235 5.84474ZM11.5339 7.67617C11.7703 7.71748 12.4198 7.99903 12.6601 8.08548C12.9003 8.16811 13.1664 8.32183 13.1405 8.4756C13.1107 8.63411 12.9868 8.63411 12.6601 8.83303C12.3372 9.02811 11.632 9.46337 11.4062 9.49314C11.1813 9.52291 11.0535 9.39605 10.8132 9.24137C10.573 9.08377 10.1224 8.71577 10.2358 8.52074C10.2358 8.52074 10.5884 8.25074 10.7422 8.1124C10.8959 7.9692 11.2937 7.63194 11.5339 7.67617ZM10.4981 6.01388C10.8767 6.01388 11.1851 6.46451 11.1851 7.01988C11.1851 7.1208 11.1736 7.21491 11.1554 7.30908C11.0612 7.33886 10.9671 7.38788 10.8767 7.46668C10.8325 7.50417 10.7911 7.53777 10.7537 7.57526C10.8132 7.46286 10.8363 7.30143 10.8094 7.13228C10.7575 6.83251 10.5548 6.61057 10.3559 6.64034C10.156 6.674 10.0368 6.9488 10.0849 7.25246C10.1378 7.55994 10.3366 7.78183 10.5394 7.74823C10.5509 7.7444 10.5615 7.74057 10.573 7.73668C10.476 7.83085 10.3856 7.91348 10.2915 7.98074C10.0176 7.85291 9.8149 7.47051 9.8149 7.01988C9.81496 6.46068 10.1186 6.01388 10.4981 6.01388ZM8.38902 13.2838C8.77913 12.6689 9.03084 11.3246 9.42096 10.8778C9.81496 10.4319 10.1186 9.48165 9.98021 9.06177C9.98021 9.06177 10.821 10.0678 11.4062 9.90251C11.9923 9.73337 13.3096 8.75428 13.5047 8.92245C13.6998 9.09154 15.3774 12.7813 15.5465 13.9564C15.7156 15.1306 15.4341 16.0281 15.4341 16.0281C15.4341 16.0281 14.7923 15.8589 14.7096 16.249C14.627 16.643 14.627 18.0699 14.627 18.0699C14.627 18.0699 13.7593 19.271 12.416 19.4698C11.0727 19.6649 10.4 19.5227 10.4 19.5227L9.64576 18.6588C9.64576 18.6588 10.2319 18.5724 10.1492 17.9834C10.0666 17.3982 8.35816 16.5872 8.05067 15.8589C7.7433 15.1305 7.9941 13.8997 8.38902 13.2838ZM5.0673 17.6903C5.13456 17.402 6.00507 17.4021 6.33947 17.1993C6.67387 16.9966 6.74107 16.4143 7.01113 16.2606C7.27724 16.103 7.76924 16.6622 7.97199 16.9773C8.1709 17.2848 8.9329 18.6291 9.24422 18.9634C9.55936 19.3007 9.84862 19.7475 9.75827 20.1491C9.67279 20.5507 9.19907 20.8438 9.19907 20.8438C8.77536 20.9745 7.59347 20.4643 7.05639 20.2394C6.5193 20.0136 5.1529 19.9463 4.97707 19.7475C4.79645 19.5447 5.06353 19.0979 5.13462 18.6742C5.19799 18.2456 4.99913 17.9795 5.0673 17.6903Z"
		/>
	</symbol>
	<!-- Tabler outline desktop -->
	<symbol
		id="icon-desktop"
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="2"
		stroke-linecap="round"
		stroke-linejoin="round"
	>
		<path stroke="none" d="M0 0h24v24H0z" fill="none" />
		<path d="M3 5a1 1 0 0 1 1 -1h16a1 1 0 0 1 1 1v10a1 1 0 0 1 -1 1h-16a1 1 0 0 1 -1 -1v-10" />
		<path d="M7 20h10" /><path d="M9 16v4" /><path d="M15 16v4" />
	</symbol>
	<!-- Tabler outline Android -->
	<symbol
		id="icon-android"
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="2"
		stroke-linecap="round"
		stroke-linejoin="round"
	>
		<path stroke="none" d="M0 0h24v24H0z" fill="none" />
		<path d="M4 10l0 6" /><path d="M20 10l0 6" />
		<path d="M7 9h10v8a1 1 0 0 1 -1 1h-8a1 1 0 0 1 -1 -1v-8a5 5 0 0 1 10 0" />
		<path d="M8 3l1 2" /><path d="M16 3l-1 2" />
		<path d="M9 18l0 3" /><path d="M15 18l0 3" />
	</symbol>
	<!-- Obtainium real logo -->
	<symbol id="icon-obtainium" viewBox="30.204 75 512.59 471.09">
		<path
			fill="currentColor"
			d="M380.1 542.6c-6.5-3.4-44.3-24.4-221.4-122.7-49.3-27.4-96.8-53.7-105.5-58.5-19.8-10.9-22.8-13.6-22.8-20.1 0-3.4 7.9-25.4 26.4-73.5 14.5-37.7 27.4-70.8 28.5-73.5 2.7-6.1 7-11.3 11.2-13.5 3.9-2 102.4-38 107.5-39.2 5.4-1.3 10.1 1.2 12.9 7.1 1.3 2.8 6.8 17.2 12.2 32.1s13.4 36.8 17.9 48.8c10.1 27.5 10.5 31.4 3.5 35.5-1.9 1.1-14.1 5.9-27.1 10.6s-25.4 9.6-27.6 10.9c-5 2.9-5.7 7.4-1.8 11 1.6 1.4 26.6 15.7 55.6 31.7 81.2 44.8 76.7 42.7 82.1 36.9 2.7-2.9 51.1-126.2 53.9-137.2 1.2-4.7 1.1-5.3-1.5-7.4-3.5-2.8-3-2.9-34.8 8.3-31.6 11.2-34.6 11.4-38.8 3.1-2.2-4.4-36.2-97.8-38.3-105.3-1.4-4.9.3-9.5 4.4-12.2 1.7-1.1 26.5-10.6 55-21.1 42.4-15.6 53.1-19.1 58.7-19.2 7.1-.1 8 .4 138.4 72.5 9 5 13.9 10.3 13.9 15.1 0 1.9-32.3 87.7-71.8 190.5-55.7 145.1-72.5 187.6-75 189.8-4.3 3.9-7.7 3.7-15.7-.5z"
		/>
	</symbol>
	<!-- Tabler outline package -->
	<symbol
		id="icon-apk"
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="2"
		stroke-linecap="round"
		stroke-linejoin="round"
	>
		<path stroke="none" d="M0 0h24v24H0z" fill="none" />
		<path d="M12 3l8 4.5l0 9l-8 4.5l-8 -4.5l0 -9l8 -4.5" />
		<path d="M12 12l8 -4.5" /><path d="M12 12l0 9" /><path d="M12 12l-8 -4.5" />
		<path d="M16 5.25l-8 4.5" />
	</symbol>
</svg>

<div class="flex w-full flex-col items-center gap-3">
	{#if download_error}
		<p
			class="error-glow text-sm font-semibold text-red-400"
			transition:fly={{ y: -4, duration: 200 }}
		>
			{download_error}
		</p>
	{/if}

	<div
		class="flex w-full max-w-xl flex-col items-stretch rounded-2xl border border-white/10 bg-white/[0.04] sm:flex-row"
	>
		<!-- ── Desktop ── -->
		<div class="relative z-10 flex flex-1 flex-col gap-2.5 p-5">
			<div
				class="flex items-center gap-1.5 text-[10px] font-semibold tracking-widest text-white/35 uppercase"
			>
				<svg
					width="15"
					height="15"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					aria-hidden="true"><use href="#icon-desktop" /></svg
				>
				Desktop
			</div>
			<p
				class="-mt-1 text-xs leading-snug text-white/0 select-none not-sm:hidden"
				aria-hidden="true"
			>
				.
			</p>

			<ButtonPopup let:toggle let:isOpen let:close>
				<div
					class="relative flex h-[38px] items-center border bg-white/[0.06] text-sm font-medium transition-[border-color,border-radius] duration-200 {isOpen
						? 'rounded-t-[10px] border-white/20'
						: 'rounded-xl border-white/[0.12]'} {loading ? 'pointer-events-none opacity-60' : ''}"
				>
					<button
						type="button"
						class="flex h-full cursor-pointer items-center gap-1.5 pr-3.5 pl-3 whitespace-nowrap text-white/70 transition-colors duration-150 outline-none hover:text-white"
						on:click={() => download(suggested_desktop)}
					>
						<svg
							width="15"
							height="15"
							fill="currentColor"
							class="shrink-0 opacity-80"
							aria-hidden="true"
						>
							<use
								href={suggested_desktop.os === 'macOS'
									? '#icon-apple'
									: suggested_desktop.os === 'Windows'
										? '#icon-windows'
										: '#icon-linux'}
							/>
						</svg>
						Download for {suggested_desktop.os}
						<span class="text-xs text-white/40">{suggested_desktop.arch}</span>
					</button>
					<div class="h-5 w-px shrink-0 bg-white/15"></div>
					<button
						type="button"
						class="flex h-full cursor-pointer items-center justify-center px-3 text-white/50 transition-colors duration-150 outline-none hover:text-white/90"
						on:click={toggle}
						aria-label="Other desktop platforms"
					>
						<svg
							fill="currentColor"
							viewBox="0 0 24 24"
							width="11"
							height="11"
							class="chevron"
							class:rotated={isOpen}
							aria-hidden="true"
						>
							<path d="M0 7.33l2.829-2.83 9.175 9.339 9.167-9.339 2.829 2.83-11.996 12.17z" />
						</svg>
					</button>
				</div>

				<div
					slot="popup"
					class="dropdown absolute top-full right-0 left-0 overflow-hidden rounded-b-[10px] border border-t-0 border-white/20 bg-white/5 backdrop-blur-md"
					style="z-index: 9999;"
					transition:scale={{ start: 0.95, opacity: 0, duration: 160 }}
				>
					{#each desktop_versions as version}
						<button
							type="button"
							class="dropdown-item flex h-9 w-full cursor-pointer items-center gap-2 bg-transparent px-3.5 text-left text-[13.5px] text-white/80 outline-none hover:bg-white/5 hover:text-white"
							on:click={() => {
								close()
								download(version)
							}}
						>
							<svg
								width="15"
								height="15"
								fill="currentColor"
								class="shrink-0 opacity-70"
								aria-hidden="true"
							>
								<use
									href={version.os === 'macOS'
										? '#icon-apple'
										: version.os === 'Windows'
											? '#icon-windows'
											: '#icon-linux'}
								/>
							</svg>
							<span class="flex-1">{version.os}</span>
							<span class="text-xs text-white/35">{version.arch}</span>
						</button>
					{/each}
				</div>
			</ButtonPopup>
		</div>

		<!-- vertical divider -->
		<div class="shrink-0 bg-white/10 not-sm:mx-3.5 not-sm:h-px sm:my-3.5 sm:w-px"></div>

		<!-- ── Android ── -->
		<div class="relative z-[5] flex flex-1 flex-col gap-2.5 p-5">
			<div
				class="flex items-center gap-1.5 text-[10px] font-semibold tracking-widest text-white/35 uppercase"
			>
				<svg width="15" height="15" fill="currentColor" aria-hidden="true"
					><use href="#icon-android" /></svg
				>
				Android
			</div>
			<p class="-mt-1 text-xs leading-snug text-white/30">Currently library browser only</p>

			<ButtonPopup let:toggle let:isOpen let:close>
				<div
					class="relative flex h-[38px] items-center border bg-white/[0.06] text-sm font-medium transition-[border-color,border-radius] duration-200 {isOpen
						? 'rounded-t-[10px] border-white/20'
						: 'rounded-xl border-white/[0.12]'} {loading ? 'pointer-events-none opacity-60' : ''}"
				>
					<a
						href={obtainium_url}
						target="_blank"
						rel="noopener noreferrer"
						class="flex h-full grow cursor-pointer items-center gap-1.5 pr-3.5 pl-3 whitespace-nowrap text-white/70 transition-colors duration-150 outline-none hover:text-white"
					>
						<svg
							width="14"
							height="14"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
							class="shrink-0 opacity-75"
							aria-hidden="true"
						>
							<use href="#icon-obtainium" />
						</svg>
						Get it on Obtainium
					</a>
					<div class="h-5 w-px shrink-0 bg-white/15"></div>
					<button
						type="button"
						class="flex h-full cursor-pointer items-center justify-center px-3 text-white/50 transition-colors duration-150 outline-none hover:text-white/90"
						on:click={toggle}
						aria-label="Other Android options"
					>
						<svg
							fill="currentColor"
							viewBox="0 0 24 24"
							width="11"
							height="11"
							class="chevron"
							class:rotated={isOpen}
							aria-hidden="true"
						>
							<path d="M0 7.33l2.829-2.83 9.175 9.339 9.167-9.339 2.829 2.83-11.996 12.17z" />
						</svg>
					</button>
				</div>

				<div
					slot="popup"
					class="dropdown absolute top-full right-0 left-0 overflow-hidden rounded-b-[10px] border border-t-0 border-white/20 bg-white/5 backdrop-blur-xl"
					style="z-index: 9999;"
					transition:scale={{ start: 0.95, opacity: 0, duration: 160 }}
				>
					{#each android_versions as version}
						{#if version.os === 'Obtainium'}
							<a
								href={obtainium_url}
								target="_blank"
								rel="noopener noreferrer"
								class="dropdown-item flex h-9 w-full cursor-pointer items-center gap-2 bg-transparent px-3.5 text-left text-[13.5px] text-white/80 no-underline transition-colors duration-100 outline-none hover:bg-white/[0.06] hover:text-white"
								on:click={close}
							>
								<svg
									width="15"
									height="15"
									fill="currentColor"
									class="shrink-0 opacity-70"
									aria-hidden="true"
								>
									<use href="#icon-obtainium" />
								</svg>
								<span class="flex-1">Obtainium</span>
								<span class="text-xs text-white/35">recommended</span>
							</a>
						{:else}
							<button
								type="button"
								class="dropdown-item flex h-9 w-full cursor-pointer items-center gap-2 bg-transparent px-3.5 text-left text-[13.5px] text-white/80 transition-colors duration-100 outline-none hover:bg-white/[0.06] hover:text-white"
								on:click={() => {
									close()
									download(version)
								}}
							>
								<svg
									width="15"
									height="15"
									fill="currentColor"
									class="shrink-0 opacity-70"
									aria-hidden="true"
								>
									<use href="#icon-apk" />
								</svg>
								<span class="flex-1">APK</span>
								<span class="text-xs text-white/35">{version.arch}</span>
							</button>
						{/if}
					{/each}
				</div>
			</ButtonPopup>
		</div>
	</div>
</div>

<style>
	.error-glow {
		text-shadow: 0 0 8px hsl(0 100% 60% / 50%);
	}
	.chevron {
		transition: transform 0.2s ease;
	}
	.chevron.rotated {
		transform: rotate(180deg);
	}
	.dropdown-item + .dropdown-item {
		border-top: 1px solid rgba(255, 255, 255, 0.05);
	}
</style>
