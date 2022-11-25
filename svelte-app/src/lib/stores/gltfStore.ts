/**
 * Svelte stores, store reactive global data for more info:
 * https://svelte.dev/docs#run-time-svelte-store
 *
 * This stores gltf data
 */

import { writable } from 'svelte/store';
import type { CurveTree } from '$lib/stores/contourLineStore';
import { craterLocations } from '$lib/stores/locationStore';
import { difficultyStore } from '$lib/stores/difficultyStore';
import { targetLocations } from '$lib/stores/locationStore';
import { debugMode } from '$lib/stores/debugStore';
import { get } from 'svelte/store';

import ApiSettings from '$lib/data/apiSettings';
import type Draggable from '$lib/data/draggable';

import init, * as wasm from 'wasm';

type Vec2 = [number, number];
type Vec3 = [number, number, number];
export interface Model {
	gltf: string;
	gltf_url: string;
	lava_gltf: string;
	lava_gltf_url: string;
	lava_paths: Vec3[][];
	craters: Vec2[];
}

export interface AltitudeGradientPair {
	x: number;
	y: number;
	altitude: number;
	gradient: Vec3;
}

// GLTF STORE helper functions

/***
 * Converts a radian to degrees
 * @param radian - radian in range [-inf, inf]
 */
function radToDeg(radians: number) {
	return radians * (180 / Math.PI);
}

function adjustAlititude(altAndgrad: AltitudeGradientPair) {
	let altitude = altAndgrad.altitude;

	// Take a small modifier that will increase the altitude by a fraction of the largest absolute gradient
	altitude += 1 * altAndgrad.gradient.map((g) => Math.abs(g)).reduce((a, b) => Math.max(a, b));

	// Increment by 1 to prevent the altitude from being under the model
	return altitude * 1.07;
}

export function gltfStringToUrl(gltf: string): string {
	const gltfBlob = new Blob([gltf], { type: 'application/json' });
	const gltfUrl = URL.createObjectURL(gltfBlob);
	return gltfUrl;
}

/**
 *  Factory for creating a gltf store
 * @returns target store with method subscribe, add and remove
 */
function createGltfStore() {
	const { subscribe, set } = writable<Model>();
	let api: wasm.ModelConstructionApi;
	let innerWindowSize: { width: number; height: number };

	let isSetup = false;
	let model: Model;

	return {
		subscribe,
		set,
		setup: async (curveTree: CurveTree, lava_path_forking: number) => {
			// if wasm is not yet setup, do so
			if (!isSetup) {
				await init();
				isSetup = true;
			}

			innerWindowSize = curveTree.size;

			// Create a wasm tree out of openCV contour tree
			const tree = new wasm.OpenCVTree({
				pixels_per_curve: curveTree.curves,
				parent_relations: curveTree.hierarchy
			});

			// Set api and parameters
			api = new wasm.ModelConstructionApi();
			// svc distance = (the largest image dimension/ number of rows or columns) / 1.5
			const api_settings = new ApiSettings(
				/*				 OpenCV tree */ tree,
				/*						Rows */ 45,
				/*					 Columns */ 45,
				/*					   Width */ curveTree.size.width,
				/*					  Height */ curveTree.size.height,
				/*	  Curve Point Separation */ 5,
				/*		  		SVC Distance */ Math.max(curveTree.size.width, curveTree.size.height) / (45 * 1.5),
				/*	Catmull Clark Iterations */ 1,
				/*			Lava Path Length */ 20,
				/*		   Lava Path Forking */ lava_path_forking,
				/*		Smoothing Operations */ [
					new wasm.SmoothingOperationApplySmoothToLayer(0, 0.3, 5, 1, false),
					new wasm.SmoothingOperationApplySmoothToMiddleLayers(0.7, 3, 5, false),
					new wasm.SmoothingOperationIncreaseAltitudeForMountainTops(0.25, true),
					new wasm.SmoothingOperationApplySmoothToMountainTops(1, 8, 1, false)
				]
			);

			api_settings.apply_to_api(api);
		},
		build: (curveTree: CurveTree) => {
			// Call the wasm api to build the model

			model = api.build().to_js() as Model;

			model.gltf_url = gltfStringToUrl(model.gltf);
			model.lava_gltf_url = gltfStringToUrl(model.lava_gltf);

			model.craters = model.craters.map((c) => [
				(c[0] * curveTree.size.width) / 100,
				(c[1] * curveTree.size.height) / 100
			]);

			// (re-)set the crater locations
			craterLocations.set(model.craters);

			// set the gltf store to the gltf string
			set(model);
		},
		getAlitituteAndGradient: (marker: Draggable, noAdjustAxis = false): AltitudeGradientPair => {
			if (!api) return { x: 0, y: 0, altitude: 0, gradient: [0, 0, 0] };

			let [adjustedX, adjustedY] = [marker.x, marker.y];

			if (!noAdjustAxis) {
				// Rust creates a 100*100 grid, so we need to convert the marker coordinates to this grid
				adjustedX = (marker.x / innerWindowSize.width) * 100;
				adjustedY = (marker.y / innerWindowSize.height) * 100;
			}

			// ask api to get altitude and gradient for a certain point
			const altitudeGradientPair = api
				.get_altitude_and_gradient_for_point(adjustedX, adjustedY)
				.to_js() as AltitudeGradientPair;

			// Get radians from rust however Aframe expects degrees
			altitudeGradientPair.gradient.map((rad) => radToDeg(rad));

			// Apply modifier to altitude
			altitudeGradientPair.altitude = adjustAlititude(altitudeGradientPair);

			return altitudeGradientPair;
		},
		computePlayerPoints: (max_points_total: number) => {
			if (!api) return 0;

			// TODO: add cache
			const { width, height } = innerWindowSize;

			return api.compute_player_points(
				new wasm.LavaPathTurbineInput({
					lava_paths: model.lava_paths,
					turbines: get(targetLocations).map((l) => [(l.x / width) * 100, (l.y / height) * 100]),
					max_lava_distance: get(difficultyStore).max_lava_distance,
					max_points_total: max_points_total
				})
			);
		}
	};
}
export const gltfStore = createGltfStore();
