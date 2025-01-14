<script lang="ts">
	import { Plus } from 'lucide-svelte';
	import { DarkMode, Navbar, Button } from 'flowbite-svelte';
	import '../app.css';
</script>

<Navbar>
	<a href="/import"
		><Button pill={true} class="!p-2" size="xl">
			<Plus />
		</Button></a
	>
	<a href="/" class="text-xl font-bold">Chronicle</a>
	<DarkMode />
</Navbar>
<div class="container mx-auto px-4">
	<slot />
</div>
