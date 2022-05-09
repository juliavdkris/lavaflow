//
// Class: LevelCurves
//
use super::level_curve_tree::LevelCurveTree;
use super::point::Point;

#[derive(Debug)]
pub struct LevelCurve {
	altitude: f32,
	points: Vec<Point>,
}

impl LevelCurve {
	pub fn new(altitude: f32) -> Self {
		Self { altitude, points: Vec::new() }
	}

	pub fn add_point(&mut self, a: Point) {
		self.points.push(a);
	}

	pub fn add_all_points(&mut self, xs: Vec<Point>) {
		for mut p in xs {
			p.z = self.altitude;
			self.points.push(p);
		}
	}

	pub fn get_points(&self) -> &Vec<Point> {
		&self.points
	}

	pub fn find_closest_point_and_distance_on_level_curve(&self, a: &Point) -> (Option<&Point>, f32) {
		if self.points.is_empty() {
			return (None, f32::INFINITY);
		}

		// Get the distance to the first point in the list, as a starting point.
		// let mut min_dist_sqr: f32 = Point::dist_sqr(&self.points[0], a);
		let mut min_dist_sqr: f32 = Point::xy_dist_sqr(&self.points[0], a);
		let mut min_dist_sqr_point: &Point = &self.points[0];

		// Loop over every point in the list and find the smallest distance.
		// You don't have to keep track of which point had this smallest distance.
		for p in &self.points {
			// let current_dist_sqr = Point::dist_sqr(p, a);
			let current_dist_sqr = Point::xy_dist_sqr(p, a);

			if current_dist_sqr < min_dist_sqr {
				min_dist_sqr = current_dist_sqr;
				min_dist_sqr_point = p;
			}
		}

		// Return the smallest distance found
		(Some(min_dist_sqr_point), f32::sqrt(min_dist_sqr))
	}

	pub fn find_closest_point_on_level_curve(&self, a: &Point) -> Option<&Point> {
		return self.find_closest_point_and_distance_on_level_curve(a).0;
	}

	pub fn dist_to_point(&self, a: &Point) -> f32 {
		return self.find_closest_point_and_distance_on_level_curve(a).1;
	}
}

//
// Class: LevelCurveMap
// This class gathers multiple level curves and provides functionality for working with
// the system as a whole
//

#[derive(Debug)]
pub struct LevelCurveSet {
	altitude_step: f32,
	level_curves: Vec<LevelCurve>,
}

impl LevelCurveSet {
	// Construct a new LevelCurveMap, by specifying the altitude per level
	pub fn new(altitude_step: f32) -> Self {
		Self {
			altitude_step,
			level_curves: Vec::new(),
		}
	}

	// Add a new level curve to the map
	pub fn add_level_curve(&mut self, a: LevelCurve) {
		self.level_curves.push(a);
	}

	// Retrieve the list of level curves
	pub fn get_level_curves(&self) -> &Vec<LevelCurve> {
		&self.level_curves
	}

	// Find points (minimum_x_cooridinate, minimum_y_coordinate) , (maximum_x_cooridinate, maximum_y_coordinate) of coordinates in levelcurveset ,
	// for the puropose of genererating a raster to cover whole area of levelcurves
	pub fn get_bounding_points(&self) -> (Point, Point){
		let mut min = Point{x : std::f32::MAX , y:std::f32::MAX, z : 0.0};
		let mut max = Point{x : 0.0, y: 0.0, z : 0.0};
		for curve in &self.level_curves {
			for point in &curve.points {
				if(point.x < min.x){
					min.x = point.x;
				}
				if(point.y < min.y){
					min.y = point.y;
				}
				if(point.x > max.x){
					max.x = point.x;
				}
				if(point.y > max.y){
					max.y = point.y;
				}
			}
		}
		(min, max)
	}

	// Finding the closest point on any level curve that's stored in this map
	pub fn find_closest_point_on_level_curve(&self, a: &Point) -> Option<&Point> {
		// If this map doesn't contain any level-curves, return None
		if self.level_curves.is_empty() {
			return None;
		}

		// Find the baseline tuple, storing a (Point, Distance)
		let mut min_dist = self.level_curves[0].find_closest_point_and_distance_on_level_curve(a);

		// Loop over every level-curve, find the point that lies closest to the specified point a
		for lc in &self.level_curves {
			let current_dist = lc.find_closest_point_and_distance_on_level_curve(a);

			if current_dist.1 < min_dist.1 {
				min_dist = current_dist;
			}
		}

		// Return the point
		min_dist.0
	}
	///
	/// transforms `levelCurveTree` to `levelCurveMap` structure, while reducing amount of total points from pixelStructure
	///
	/// # Arguments
	///
	/// * `tree` - `levelCurveTree` datastructure containing information from scanning step
	/// * `altitude_step` - increase in height per contour line
	/// * `desired_dist` - minimum desired distance between points in final conout map
	/// * `current_height` - to track height when traversing tree, initial call should start with 1
	///
	#[allow(non_snake_case)]
	pub fn transform_to_LevelCurveMap<'a>(&self, tree: &'a mut LevelCurveTree<'a>, altitude_step: f32, desired_dist: f32, current_height: usize) -> LevelCurveSet {
		let mut ret: LevelCurveSet = LevelCurveSet::new(altitude_step);

		let mut current_level_curve = LevelCurve::new(altitude_step * current_height as f32);

		// TODO: dont use unwrap
		let first_pixel = tree.get_first_pixel().unwrap();
		let mut last_saved = first_pixel;
		let mut last_visited = first_pixel;
		let mut current_pixel = first_pixel;

		// untill we rencounter the first pixel, search direct neightborhood (directly adjacent pixels) of current pixel for next pixel
		// Assumption: there are no breaks in the line
		// break for loop in line 165

		loop {
			// Assumption: pixels have directly connected neighbors (diagonals do not count as adjacent)
			// Assumption: line is exactly 1 pixel wide
			// TODO; check if in actual input every pixel has adjacent pixel
			let neighbors = vec![
				(current_pixel.0 - 1, current_pixel.1),
				(current_pixel.0 + 1, current_pixel.1),
				(current_pixel.0, current_pixel.1 - 1),
				(current_pixel.0, current_pixel.1 + 1),
			];
			for (x, y) in neighbors {
				// TODO: check how this holds for corner cases
				if (x, y) != current_pixel && (x, y) != last_visited && tree.contains_pixel(x, y) {
					// if dist to last saved and current pixel is desired length, save current pixel, else move on
					if pixel_dist(&(x, y), &last_saved) >= desired_dist {
						current_level_curve.add_point(Point {
							x: x as f32,
							y: y as f32,
							z: current_level_curve.altitude,
						});
						last_saved = (x, y);
					}

					last_visited = current_pixel;
					current_pixel = (x, y);
				}
			}
			if current_pixel == first_pixel {
				break;
			}
		}

		// for every child get levelcurvemap and add to ret

		for mut child in tree.get_children() {
			let childmap = self.transform_to_LevelCurveMap(&mut child, altitude_step, desired_dist, current_height + 1);
			for curve in childmap.level_curves {
				ret.add_level_curve(curve);
			}
		}

		ret
	}
}
// TODO: find better method
fn pixel_dist(a: &(u64, u64), b: &(u64, u64)) -> f32 {
	((a.0 as f32 - b.0 as f32).powi(2) + (a.1 as f32 - b.1 as f32).powi(2)).sqrt()
}
