import containerQueries from '@tailwindcss/container-queries';
import forms from '@tailwindcss/forms';
import plugin from 'tailwindcss/plugin';
import flowbitePlugin from 'flowbite/plugin';
import type { Config } from 'tailwindcss';

export default {
	content: ['./src/**/*.{html,js,svelte,ts}', './node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			colors: {
				primary: {
					50: '#F9DCE5',
					100: '#F7CAD8',
					200: '#F1A7BE',
					300: '#E9779A',
					400: '#E24676',
					500: '#CF2157',
					600: '#9E1943',
					700: '#6E122E',
					800: '#3E0A1A',
					900: '#0D0206',
					950: '#000000'
				}
			},
		},
	},
	darkMode: "class",
	plugins: [flowbitePlugin, forms, containerQueries]
} satisfies Config;
