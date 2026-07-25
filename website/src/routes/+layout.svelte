<script lang="ts">
	import { page } from '$app/stores'
	import '../app.css'
	import Revealed from './Revealed.svelte'
</script>

<svelte:head>
	<title>Ferrum</title>
	<meta name="description" content="Music library and player" />

	<meta property="og:type" content="website" />
	<meta property="og:url" content="https://ferrum.kasper.space/" />
	<meta property="og:title" content="Ferrum" />
	<meta property="og:description" content="Music library and player" />
	<meta property="og:image" content="https://ferrum.kasper.space/website-screenshot.jpg" />

	<meta name="twitter:card" content="summary_large_image" />
	<meta property="twitter:domain" content="ferrum.kasper.space" />
	<meta property="twitter:url" content="https://ferrum.kasper.space/" />
	<meta name="twitter:title" content="Ferrum" />
	<meta name="twitter:description" content="Music library and player" />
	<meta name="twitter:image" content="https://ferrum.kasper.space/website-screenshot.jpg" />
</svelte:head>

<nav
	class="mx-auto flex max-w-5xl items-center py-4 px-3 text-base text-white/60 sm:justify-between sm:px-6"
>
	<Revealed
		class="ease-out-cubic sm:w-32"
		options={{ opacity: 0, x: -10, duration: 1000, delay: 300 }}
	>
		<a
			href="/"
			class="flex items-center text-lg font-medium text-white sm:p-1.5 sm:text-xl"
			style="-webkit-tap-highlight-color: hsla(0, 0%, 100%, 0.1);"
		>
			<img class="mr-2 h-8" src="/favicon.svg" alt="" />
			<span class="mr-1.5 max-[350px]:hidden">Ferrum</span>
		</a>
	</Revealed>
	<Revealed
		class="ease-out-cubic mx-1.5"
		options={{ opacity: 0, y: -10, duration: 1000, delay: 500 }}
	>
		<a
			class="mx-1.5 py-2 text-sm transition-colors duration-150 hover:text-white sm:text-base"
			class:font-medium={$page.url.pathname === '/'}
			class:text-white={$page.url.pathname === '/'}
			href="/">Overview</a
		>
		<a
			class="mx-1.5 py-2 text-sm transition-colors duration-150 hover:text-white sm:text-base"
			class:font-medium={$page.url.pathname === '/changelog'}
			class:text-white={$page.url.pathname === '/changelog'}
			href="/changelog">Changelog</a
		>
	</Revealed>
	<Revealed
		class="ease-out-cubic ml-auto flex justify-end sm:ml-0 sm:w-32"
		options={{ opacity: 0, x: 10, duration: 1000, delay: 300 }}
	>
		<a
			class="icon github sm:p-1.5"
			href="https://github.com/kasper9n/ferrum"
			aria-label="GitHub"
			style="-webkit-tap-highlight-color: hsla(0, 0%, 100%, 0.1);"
			><svg height="24" width="24" viewBox="-2 -2 28 28" xmlns="http://www.w3.org/2000/svg"
				><path
					d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"
					fill="none"
					stroke="#ffffff"
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
				/></svg
			></a
		>
	</Revealed>
</nav>

<slot />

<style lang="sass">
	:global(html)
		background-color: #000000
		-webkit-tap-highlight-color: transparent
	:global(body)
		font-family: -apple-system, BlinkMacSystemFont, Segoe UI, Helvetica, Arial, sans-serif, Apple Color Emoji, Segoe UI Emoji
		font-size: 18px
		color: #f2f2f2
</style>
