
use super::{raster::Raster, level_curves::{LevelCurveMap, LevelCurve}};
use super::point::Point;


//
// Additional functions
//

fn calcInverseWeightedAverage(weighted_values: &Vec<(f64, f64)>) -> f64 {

    let mut res: f64 = 0.0;
    let mut sum_weight: f64 = 0.0;

    for i in 0..weighted_values.len() {
        res += weighted_values[i].0 * weighted_values[i].1;
        sum_weight += weighted_values[i].1;
    }

    res/sum_weight
}

fn calcDistanceBetweenCells(row0: usize, col0: usize, row1: usize, col1: usize) -> f64 {
    f64::sqrt((row1 as f64-row0 as f64).powi(2) + (col1 as f64-col0 as f64).powi(2))
}

#[derive(Debug)]
pub struct ModelConstructor<'a> {
	contour_margin: f64,
    level_curve_map: LevelCurveMap,
	is_svc: Vec<Vec<bool>>,
	raster: &'a mut Raster,
}

// @Pauline are you planning to implement this trait for other structs as well?
// Otherwise using a trait is unnecessary, you can just implement functions directly for a struct without using traits.
// (Traits are only really meant for defining shared behavior. If you want a "contract" for a function that still has to be implemented, you can use the todo macro: https://doc.rust-lang.org/std/macro.todo.html)
// - Julia


trait HelperFunctions {
	fn initialize();
	fn check_svc( ix: usize, iy: usize) -> bool;
	// fn getClostestCOntourPoint(p, levelcurvemap, );
	fn local_tin(p: Vec<f64>);
	fn calc_heights_nvcs();
}

impl<'a> ModelConstructor<'a> {
	//TODO add levelmap to function parameters
	fn construct_map(&mut self, raster: &'a mut Raster, _contour_margin: i64)  {
		let x = raster.columns;
		let y = raster.rows;
		self.raster = raster;
		self.is_svc = vec![vec![false; x]; y];
		
        // Set the edges of the raster to zero
        self.setRasterEdgesToZero();

		for i in 0 .. x {
			for j in 0 .. y{
				//TODO remove line here once checkSVC is implemented
				if self.check_svc( i, j) {
					if self.raster.altitudes[i][j].is_none() {
						//local_tin(cellCentre)
					}
				}
			}		
		}
		//calc_heights_nvc()

	}


    // Function: initialize
    fn setRasterEdgesToZero(&mut self) {

        // Initialize all the outer-most raster values to zero
        // (According to requirement that the edges of the mountain must have level 0)
        for i in 0..self.raster.rows {
            for j in 0..self.raster.columns {
                // If this box is on the outer edge
                if i == 0 || j == 0 || i == self.raster.rows-1 || j == self.raster.columns-1 {
                    // Set its value to zero
                    self.raster.set(i, j, 0.0);
                    
                    // Set this box to an svc-box
                    self.is_svc[i][j] = true;
                }
            }
        }

    }

    // Function: Calculate Height of NVCs
    fn calc_heights_nvc(&mut self) {
        for row in 0..self.raster.rows {
            for col in 0..self.raster.columns {

                if self.raster.get(row, col).is_none() {

                    let mut neighbours: Vec<(f64, f64)> = Vec::new();

                    neighbours.push(self.findSVCNorth(row, col));
                    neighbours.push(self.findSVCNorthEast(row, col));
                    neighbours.push(self.findSVCNorthWest(row, col));
                    neighbours.push(self.findSVCSouth(row, col));
                    neighbours.push(self.findSVCSouthEast(row, col));
                    neighbours.push(self.findSVCSouthWest(row, col));
                    neighbours.push(self.findSVCEast(row, col));
                    neighbours.push(self.findSVCWest(row, col));

                    self.raster.set(row, col, calcInverseWeightedAverage(&neighbours));

                }

            }
        }
    }

    //
    // A set of functions for finding the nearest SVC box in directions north (west, east) and south (west, east)
    //

    // Function: find SVC north
    fn findSVCNorth(&self, i: usize, j: usize) -> (f64, f64) {

        let mut row = i;
        let     col = j;

        while row < self.raster.rows && col < self.raster.columns && row >= 0 && col >= 0 {

            // If this box is svc, return its position
            if self.is_svc[row][col] {
                return (self.raster.get(row, col).expect("SVC Without Value was Found"), calcDistanceBetweenCells(i, j, row, col));
            }

            // Walk one step further
            row -= 1;

        }

        // If no value can be found, return (0, 0)
        (0.0, 0.0)

    }

    // Function: find SVC south
    fn findSVCSouth(&self, i: usize, j: usize) -> (f64, f64) {

        let mut row = i;
        let     col = j;

        while row < self.raster.rows && col < self.raster.columns && row >= 0 && col >= 0 {

            // If this box is svc, return its position
            if self.is_svc[row][col] {
                return (self.raster.get(row, col).expect("SVC Without Value was Found"), calcDistanceBetweenCells(i, j, row, col));
            }

            // Walk one step further
            row += 1;

        }

        // If no value can be found, return (0, 0)
        (0.0, 0.0)

    }

    // Function: find SVC west
    fn findSVCWest(&self, i: usize, j: usize) -> (f64, f64) {

        let     row = i;
        let mut col = j;

        while row < self.raster.rows && col < self.raster.columns && row >= 0 && col >= 0 {

            // If this box is svc, return its position
            if self.is_svc[row][col] {
                return (self.raster.get(row, col).expect("SVC Without Value was Found"), calcDistanceBetweenCells(i, j, row, col));
            }

            // Walk one step further
            col -= 1;

        }

        // If no value can be found, return (0, 0)
        (0.0, 0.0)

    }

    // Function: find SVC east
    fn findSVCEast(&self, i: usize, j: usize) -> (f64, f64) {

        let     row = i;
        let mut col = j;

        while row < self.raster.rows && col < self.raster.columns && row >= 0 && col >= 0 {

            // If this box is svc, return its position
            if self.is_svc[row][col] {
                return (self.raster.get(row, col).expect("SVC Without Value was Found"), calcDistanceBetweenCells(i, j, row, col));
            }

            // Walk one step further
            col += 1;

        }

        // If no value can be found, return (0, 0)
        (0.0, 0.0)

    }

    // Function: find SVC north west
    fn findSVCNorthWest(&self, i: usize, j: usize) -> (f64, f64) {

        let mut row = i;
        let mut col = j;

        while row < self.raster.rows && col < self.raster.columns && row >= 0 && col >= 0 {

            // If this box is svc, return its position
            if self.is_svc[row][col] {
                return (self.raster.get(row, col).expect("SVC Without Value was Found"), calcDistanceBetweenCells(i, j, row, col));
            }

            // Walk one step further
            row -= 1;
            col -= 1;

        }

        // If no value can be found, return (0, 0)
        (0.0, 0.0)

    }

    // Function: find SVC north east
    fn findSVCNorthEast(&self, i: usize, j: usize) -> (f64, f64) {

        let mut row = i;
        let mut col = j;

        while row < self.raster.rows && col < self.raster.columns && row >= 0 && col >= 0 {

            // If this box is svc, return its position
            if self.is_svc[row][col] {
                return (self.raster.get(row, col).expect("SVC Without Value was Found"), calcDistanceBetweenCells(i, j, row, col));
            }

            // Walk one step further
            row -= 1;
            col += 1;

        }

        // If no value can be found, return (0, 0)
        (0.0, 0.0)

    }

    // Function: find SVC south east
    fn findSVCSouthEast(&self, i: usize, j: usize) -> (f64, f64) {

        let mut row = i;
        let mut col = j;

        while row < self.raster.rows && col < self.raster.columns && row >= 0 && col >= 0 {

            // If this box is svc, return its position
            if self.is_svc[row][col] {
                return (self.raster.get(row, col).expect("SVC Without Value was Found"), calcDistanceBetweenCells(i, j, row, col));
            }

            // Walk one step further
            row += 1;
            col += 1;

        }

        // If no value can be found, return (0, 0)
        (0.0, 0.0)

    }

    // Function: find SVC south west
    fn findSVCSouthWest(&self, i: usize, j: usize) -> (f64, f64) {

        let mut row = i;
        let mut col = j;

        while row < self.raster.rows && col < self.raster.columns && row >= 0 && col >= 0 {

            // If this box is svc, return its position
            if self.is_svc[row][col] {
                return (self.raster.get(row, col).expect("SVC Without Value was Found"), calcDistanceBetweenCells(i, j, row, col));
            }

            // Walk one step further
            row += 1;
            col -= 1;

        }

        // If no value can be found, return (0, 0)
        (0.0, 0.0)

    }

	fn check_svc(&mut self, row: usize, col: usize) -> bool {

		//TODO z point is now "0" but doesnt really exist
		//define which points are corner and center of current cell 
		let corner: Point = Point{ x : (row as f64 ) * self.raster.row_height,
								   y: (col as f64 ) * self.raster.column_width,
								   z: 0.0 } ;
		let center: Point = Point{ x : (row as f64 ) + 0.5 * self.raster.row_height,
								   y: (col as f64 ) + 0.5 * self.raster.column_width,
								   z: 0.0 } ;

		//find point on a contour line closest to center of cell
		let optional : Option<&Point> = self.level_curve_map.find_closest_point_on_level_curve(&center);

		match optional {
			 Some(p) =>
				//todo, check row_height etc is correct

				//check closest point is outside of cell
				if(p.x < corner.x || p.x > corner.x + self.raster.row_height || p.y < corner.y || p.y > corner.y + self.raster.column_width){
					return false;
				}
				//check if center of cell is within distance [contour margin] of closest contour point, if it is we consider it 'exactly' on the contour line
				else if (center.x - p.x  ).abs() < self.contour_margin  &&  (center.y - p.y  ).abs() < self.contour_margin {
					self.is_svc[row][col] = true;
					self.raster.altitudes[row][col] = Some(p.z);
					true 
				}
				//if center of cell is not in distance [contour margin], its height must be interpolated
				else {
					self.is_svc[row][col] = true;
					true
				} , 

			 None => false
		}
	}

}

