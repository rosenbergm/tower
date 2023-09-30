<script lang="ts">
	import { goto } from '$app/navigation';
	import Modal from '$lib/components/Modal.svelte';
	import type { SubmitFunction } from '@sveltejs/kit';
	import type { PageData } from './$types';

	import { Icon, PlusCircle, ArrowPathRoundedSquare, Trash } from 'svelte-hero-icons';
	import { onMount } from 'svelte';
	import { dockerAppsDecoder, type DockerAppT } from '$lib/decoders';
	import DockerApp from '$lib/components/DockerApp.svelte';

	let apps: DockerAppT[] = [];

	const fetchApps = async () => {
		const res = await fetch(`http://127.0.0.1:8000/docker/apps`);
		const rawJson = await res.json();

		console.log(rawJson);

		const decoded = dockerAppsDecoder.decode(rawJson);

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

		const res = await fetch(`http://127.0.0.1:8000/docker/new`, {
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
		<h2 slot="header" class="font-mono text-lg font-semibold">Create new Docker app</h2>

		<form on:submit|preventDefault={(event) => newApp(event)} class="flex flex-col">
			<label>
				Name
				<input type="text" name="name" />
			</label>
			<label>
				Image
				<input type="text" name="image_url" />
			</label>
			<label>
				Exposing port (the port you Dockerfile is exposing)
				<input type="number" name="exposing_port" />
			</label>
			<label>
				Domain
				<input type="text" name="domain" />
			</label>
			<button type="submit">Start</button>
		</form>
	</Modal>

	<h1 class="font-mono text-lg font-semibold">
		Docker apps
		<button
			on:click={() => (showNewModal = true)}
			class="w-6 h-6 align-middle inline-flex items-center justify-center bg-[#00FF0011] rounded-md"
			title="Add new docker app"><Icon src={PlusCircle} size="18" /></button
		>
	</h1>
	<a class="font-mono text-sm underline" href="/">Go back</a>

	<section class="grid grid-cols-auto-fit gap-4 py-6 overflow-y-scroll">
		{#each apps as app}
			<DockerApp {app} refetch={fetchApps} />
		{:else}
			<p class="font-mono text-xs">There are no Docker apps):</p>
		{/each}
	</section>
</main>
