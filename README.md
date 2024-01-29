# Random Walk

Two dots taking a stroll. The size of the grid increases when their paths collide.

Their movement uses quadratic ease-in-out interpolation, and line segment intersections are determined by picking an ordered triplet and checking for a [counterclockwise orientation](https://www.geeksforgeeks.org/check-if-two-given-line-segments-intersect/).
