#include "NonConvexHull.h"


NonConvexHull::NonConvexHull(vector<b2Vec2>& shape) {
	auto node = m_ccwVertices.before_begin();
	for(auto it = shape.begin(); it != shape.end(); it++) {
		node = m_ccwVertices.emplace_after(node, *it);
	}
}

NonConvexHull::NonConvexHull(const NonConvexHull& other) {
	m_ccwVertices = other.getVertices();
}

NonConvexHull::~NonConvexHull() {
}


NonConvexHull* NonConvexHull::getSubHull(const forward_list<b2Vec2>& splice) {
	return NULL;
}

