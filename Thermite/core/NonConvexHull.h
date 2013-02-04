#pragma once

#include <forward_list>
#include <vector>
#include <Box2D.h>

typedef forward_list<b2Vec2>::iterator VertexNode;

class NonConvexHull {
public:
	NonConvexHull(vector<b2Vec2>& shape);
	~NonConvexHull();
	
	NonConvexHull* getSubHull(forward_list<b2Vec2>& splice);
	forward_list<b2Vec2> getVertices() { return m_ccwVertices; }


private:
	forward_list<b2Vec2> m_ccwVertices;
};

