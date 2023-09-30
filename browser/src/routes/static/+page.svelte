<script lang="ts">
	import { goto } from '$app/navigation';
	import Modal from '$lib/components/Modal.svelte';
	import type { SubmitFunction } from '@sveltejs/kit';

	import { Icon, PlusCircle, ArrowPathRoundedSquare, Trash } from 'svelte-hero-icons';
	import { onMount } from 'svelte';
	import {
		dockerAppsDecoder,
		staticAppsDecoder,
		type DockerAppT,
		type StaticAppT
	} from '$lib/decoders';
	import DockerApp from '$lib/components/DockerApp.svelte';
	import StaticApp from '$lib/components/StaticApp.svelte';

	let apps: StaticAppT[] = [];

	const fetchApps = async () => {
		const res = await fetch(`http://127.0.0.1:8000/static/apps`);
		const rawJson = await res.json();

		console.log(rawJson);

		const decoded = staticAppsDecoder.decode(rawJson);

		if (decoded === null) {
			return;
		}

		apps = decoded;
	};

	onMount(async () => {
		await fetchApps();
	});

	let showNewModal = false;

	$: console.log(showNewModal);

	let newApp = async (event: SubmitEvent) => {
		console.log('form!');

		const data = new FormData(event.target as HTMLFormElement);

		const res = await fetch(`http://127.0.0.1:8000/static/new`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/x-www-form-urlencoded'
			},
			// @ts-expect-error
			body: new URLSearchParams(data)
		});

		if (res.status !== 200) {
			alert('Cannot create a new app');
			return;
		}

		showNewModal = false;

		window.location.reload();
	};
</script>

<main class="py-4 px-8">
	<Modal bind:showModal={showNewModal}>
		<h2 slot="header">Start new Static app</h2>

		<form on:submit|preventDefault={(event) => newApp(event)} class="flex flex-col">
			<label>
				Name
				<input type="text" name="name" />
			</label>
			<label>
				Entrypoint (absolute path)
				<input type="text" name="entrypoint" />
			</label>
			<label>
				Domain
				<input type="text" name="domain" />
			</label>

			<button type="submit">Start</button>
		</form>
	</Modal>

	<h1 class="font-mono text-lg font-semibold">
		Static apps
		<button
			on:click={() => (showNewModal = true)}
			class="w-6 h-6 align-middle inline-flex items-center justify-center bg-[#00FF0011] rounded-md"
			title="Add new static app"><Icon src={PlusCircle} size="18" /></button
		>
	</h1>
	<a class="font-mono text-sm underline" href="/">Go back</a>

	<section class="grid grid-cols-auto-fit gap-4 py-6 overflow-y-scroll">
		{#each apps as app}
			<StaticApp {app} refetch={fetchApps} />
		{:else}
			<p class="font-mono text-xs">There are no static apps):</p>
		{/each}
	</section>
</main>
