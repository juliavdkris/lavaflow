use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use crate::utils::log;

use super::constructor::ModelConstructor;
use super::gltf_conversion::generate_gltf;
use super::level_curve_tree::LevelCurveTree;
use super::level_curves::LevelCurveSet;
use super::raster::Raster;

/// Struct representing a tree coming from OpenCV, that has not yet been converted to our internal tree structure
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct OpenCVTree {
	pixels_per_curve: Vec<Vec<(u64, u64)>>,
	parent_relations: Vec<isize>,
}

#[wasm_bindgen]
impl OpenCVTree {
	#[wasm_bindgen(constructor)]
	pub fn new(val: JsValue) -> Result<OpenCVTree, JsValue> {
		val.into_serde().map_err(|_| JsValue::from("Could not parse input from JavaScript as a valid OpenCVTree"))
	}

	pub fn debug(&self) -> String {
		format!("{self:?}")
	}
}

/// Struct used to nicely package settings for the `generate_3d_model` function.
/// - `contour_margin` - Margin that defines when a point is considered 'on' a contour line, high value results in more staircase-like appearance, low value might lead to innacurate result.
/// NOTE: margin must be above max(raster height, column width) so long as local_tin() is not implemented
/// - `columns` - desired number columns used for raster
/// - `rows` - desired number rows used for raster
/// - `altitude_step` - fixed increase in height per level curve
#[wasm_bindgen]
#[derive(Debug)]
pub struct ModelGenerationSettings {
	pub contour_margin: f32,
	pub columns: usize,
	pub rows: usize,
	pub altitude_step: f32,
	pub desired_dist: f32,
}

#[wasm_bindgen]
impl ModelGenerationSettings {
	#[wasm_bindgen(constructor)]
	pub fn new(contour_margin: f32, columns: usize, rows: usize, altitude_step: f32, desired_dist: f32) -> Self {
		Self {
			contour_margin,
			columns,
			rows,
			altitude_step,
			desired_dist,
		}
	}

	pub fn debug(&self) -> String {
		format!("{self:?}")
	}
}

/// Supermethod that takes in an openCV tree and outputs an GTLF model.
/// - `tree`- input from the image processing step, a representation of level curves. To be converted to 3D model
#[wasm_bindgen]
pub fn generate_3d_model(open_cv_tree: &OpenCVTree, settings: &ModelGenerationSettings) -> Result<String, JsValue> {

	crate::utils::set_panic_hook();

	// Unpack function argument structs & build OpenCV tree struct
	let parent_relations = open_cv_tree
		.parent_relations
		.iter()
		.map(|r| match r {
			-1 => None,
			_ => Some(*r as usize),
		})
		.collect();
	let mut tree = LevelCurveTree::from_open_cv(&open_cv_tree.pixels_per_curve, &parent_relations);
	let ModelGenerationSettings {
		contour_margin,
		columns,
		rows,
		altitude_step,
		desired_dist,
	} = *settings;

	log!("The tree: {:?}", tree);

	// convert openCV tree to levelCurveMap (input for construction algorithm)
	let mut level_curve_map = LevelCurveSet::new(altitude_step).transform_to_LevelCurveMap(&mut tree, altitude_step, desired_dist, 1).map_err(|_| String::from("Could not transform LevelCurveMap"))?;

	log!("The level_curve_map: {:?}", level_curve_map);

	//find maximum and minimum cooridinates in level curve model
	let (min, max) = level_curve_map.get_bounding_points();

	//to keep border of 10% of each axis around model
	let border_x = 0.1 * (max.x - min.x);
	let border_y = 0.1 * (max.y - min.y);

	//ensure none of the level curve points have negative coordinates
	level_curve_map.align_with_origin(&min, border_x, border_y);

	//find maxum cooridinates in level curve model
	let max = level_curve_map.get_bounding_points().1;

	log!("The configured height is: {}", (max.y - min.y) + border_y);

	//create raster based on level curve model and desired rows and columns
	let mut raster = Raster::new((max.x - min.x) + border_x, (max.y - min.y) + border_y, rows, columns);

	// create new modelConstructor (module containing 3D-model construction algorithm)
	let mut model_constructor = ModelConstructor::new(&mut raster, contour_margin, &level_curve_map);

	// determine heights
	model_constructor.construct().map_err(|e| e.to_string())?;

	// convert height raster to flat list of x,y,z points for GLTF format
	// every cell had 4 corners, becomes two triangles
	let mut final_points: Vec<[f32; 3]> = Vec::new();

	for row in 1..raster.rows {
		for col in 0..raster.columns {
			final_points.push([((row-1) as f32)*raster.row_height, (col as f32)*raster.column_width, raster.altitudes[row][col].ok_or("Point not found")?]);
			final_points.push([((row) as f32)*raster.row_height, (col as f32)*raster.column_width, raster.altitudes[row][col].ok_or("Point not found")?]);
		}
	}

	// generate_gltf(final_points).map_err(JsValue::from)
	// Ok(format!("{:?}", final_points))
	Ok(format!(""))
}
