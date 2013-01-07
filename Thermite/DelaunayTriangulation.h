#include <Box2D.h>
#include <iostream>
#include <stdlib.h> // for C qsort 
#include <cmath>
#include <time.h> // for random


const double EPSILON = 0.000001;

struct ITRIANGLE{
	int p1, p2, p3;
};

struct IEDGE{
	int p1, p2;
};

struct XYZ{
	double x, y;
};


int XYZCompare(const void *v1, const void *v2);

int CircumCircle(double xp, double yp, double x1, double y1, double x2, 
				 double y2, double x3, double y3, double &xc, double &yc, double &r);

int Triangulate(int nv, XYZ pxyz[], ITRIANGLE v[], int &ntri);

int CreateDelaunayTriangulation(b2Vec2 * verts, int n_verts, b2Vec2* extraPoints, int pointCount, b2Vec2* triangles);