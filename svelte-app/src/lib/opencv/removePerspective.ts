import cv, { type Mat } from 'opencv-ts';

export default function removePerspective(
	image: Mat,
	points: number[],
	width: number = 800,
	height: number = 667
): Mat {
	// construct mats (required input for cv.getPerspectiveTransform) with four columns and two rows to represent the four 2d points
	let sourcePoints = cv.matFromArray(4, 2, cv.CV_32FC1, points);
	let outputPoints = cv.matFromArray(4, 2, cv.CV_32FC1, [0, 0, width, 0, width, height, 0, height]);

	// create the transformation matrix
	let M = cv.getPerspectiveTransform(sourcePoints, outputPoints);

	// create an empty mat for the output image and specify its size
	let result = new cv.Mat();
	let dsize = new cv.Size(width, height);

	// use the matrix to apply the transformation
	cv.warpPerspective(image, result, M, dsize, cv.INTER_LINEAR, cv.BORDER_CONSTANT, new cv.Scalar());

	// Deallocate all Mat instances. Instances of Size and Scalar do not need to be manually deallocated.
	M.delete();
	sourcePoints.delete();
	outputPoints.delete();

	return result;
}
