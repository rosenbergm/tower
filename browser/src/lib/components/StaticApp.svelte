<script lang="ts">
	import type { StaticAppT } from '$lib/decoders';

	import { Icon, ArrowPathRoundedSquare, ArrowUpCircle, Trash } from 'svelte-hero-icons';

	export let app: StaticAppT;
	export let refetch: () => Promise<void>;

	let creating = false;
	let refreshing = false;
	let upgrading = false;

	const deleteApp = async () => {
		const res = await fetch(`http://127.0.0.1:8000/docker/${app.name}`, {
			method: 'DELETE'
		});

		if (res.status !== 200) {
			alert('Cannot delete app');
			return;
		}

		window.location.reload();
	};

	const updateToLatest = async () => {
		const res = await fetch(`http://127.0.0.1:8000/docker/${app.name}/update/manual`, {
			method: 'POST'
		});

		if (res.status !== 200) {
			alert('Cannot update app');
			return;
		}

		window.location.reload();
	};

	const restart = async () => {
		const res = await fetch(`http://127.0.0.1:8000/docker/${app.name}/restart`, {
			method: 'POST'
		});

		if (res.status !== 200) {
			alert('Cannot restart app');
			return;
		}

		window.location.reload();
	};
</script>

<article class="p-4 border-[#00FF00] border-2 rounded-md">
	<table class="w-full">
		<tbody>
			<tr>
				<td class="font-mono text-xs uppercase">App name</td>
				<td class="font-mono text-xs" align="right">{app.name}</td>
			</tr>
			<tr>
				<td class="font-mono text-xs uppercase">Domain</td>
				<td class="font-mono text-xs" align="right"
					><a class="underline" href={'https://' + app.domain} target="_blank">{app.domain}</a></td
				>
			</tr>
			<tr>
				<td class="font-mono text-xs uppercase">Mountpoint</td>
				<td class="font-mono text-xs" align="right">
					{app.mountpoint ?? 'Not mounted'}
				</td>
			</tr>
			<tr>
				<td class="font-mono text-xs uppercase">Entrypoint</td>
				<td class="font-mono text-xs" align="right">{app.entrypoint ?? 'Not mounted'}</td>
			</tr>
		</tbody>
	</table>
	<table class="w-full mt-2">
		<tbody>
			<tr class="pt-2">
				<td class="font-mono text-xs uppercase">Actions</td>
				<td align="right">
					<div class="flex flex-row flex-wrap justify-end gap-2">
						<button
							on:click={restart}
							class={`w-6 h-6 flex items-center justify-center bg-[#00FF0011] rounded-md ${
								refreshing ? 'animate-pulse' : ''
							}`}
							title="Redeploy"><Icon src={ArrowPathRoundedSquare} size="18" /></button
						>
						<button
							on:click={updateToLatest}
							class={`w-6 h-6 flex items-center justify-center bg-[#00FF0011] rounded-md ${
								upgrading ? 'animate-pulse' : ''
							}`}
							title="Update to latest"><Icon src={ArrowUpCircle} size="18" /></button
						>
						<button
							on:click={deleteApp}
							class="w-6 h-6 flex items-center justify-center bg-[#FF000011] rounded-md"
							title="Delete"><Icon src={Trash} size="18" /></button
						>
					</div>
				</td>
			</tr>
		</tbody>
	</table>
</article>
