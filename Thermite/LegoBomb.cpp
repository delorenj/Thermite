//
//  LegoBomb.cpp
//  Thermite
//
//  Created by Jarad Delorenzo on 12/18/12.
//
//

#include "LegoBomb.h"

using namespace cocos2d;

LegoBomb::LegoBomb() {
    
}

LegoBomb::~LegoBomb() {
    
}

const char* LegoBomb::getName() {
    return "Lego Bomb";
}

b2Vec2 LegoBomb::getEdgeBreakPoint(b2Body* body, b2Vec2 clickPoint, int angle, int targetFactor ) {
    int cutAngle = angle;
    b2Vec2 p1 = b2Vec2((clickPoint.x+(0.1)+(2000*targetFactor)*cos(cutAngle)),(clickPoint.y+(2000*targetFactor)*sin(cutAngle)));
    b2Vec2 p2 = clickPoint;
    b2RayCastInput input;
    input.p1 = p1;
    input.p2 = p2;
    input.maxFraction = 1;
    float closestFraction = 1;
    bool intersected = false;

    for (auto f = body->GetFixtureList(); f; f = f->GetNext()) {
        b2RayCastOutput output;
        if (!f->RayCast(&output, input, 0))
            continue;
        intersected = true;
        if (closestFraction > output.fraction)
            closestFraction = output.fraction;
    }
    if (!intersected) {
        CCLOG("NO INTERSECTION FOUND");
        return b2Vec2(-1,-1);
    } else {
        b2Vec2 hitPoint = input.p1 + closestFraction * (input.p2 - input.p1);
        CCLOG("HIT AT: (%f, %f)", hitPoint.x, hitPoint.y);
        return hitPoint;
    }
}

void LegoBomb::subdivide(b2Body* body, vector<vector<b2Vec2>* > &shapeVerts) {
    b2Vec2 center = body->GetLocalPoint(this->getPosition());
    vector<b2Vec2>* poly1 = new vector<b2Vec2>();
    vector<b2Vec2>* poly2 = new vector<b2Vec2>();

    CCLog("Center (world): %f, %f", getPosition().x, getPosition().y);
    CCLog("Center (local): %f, %f", center.x, center.y);
    
    b2Fixture* pfix = body->GetFixtureList();
    b2PolygonShape* pshape = (b2PolygonShape*)pfix->GetShape();
    int numVertices = pshape->GetVertexCount();

    b2Vec2 p1 = getEdgeBreakPoint(body, getPosition(), 0, 1);
    b2Vec2 p2 = getEdgeBreakPoint(body, getPosition(), -180, 1);

    p1 = body->GetLocalPoint(p1);
    p2 = body->GetLocalPoint(p2);

    
	//MANUAL
	poly1->push_back(pshape->GetVertex(0));
	poly1->push_back(pshape->GetVertex(1));
    poly1->push_back(p1);
    poly1->push_back(center);
    poly1->push_back(p2);
	poly1->push_back(pshape->GetVertex(3));

    poly2->push_back(p1);
    poly2->push_back(center);
    poly2->push_back(p2);
	poly2->push_back(pshape->GetVertex(2));


    for (int i=0; i<numVertices; i++) {
		//d=det(p1.x, p1.y, center.x, center.y, verticesVec[i].x, verticesVec[i].y);
		//if (d>0) {
		//	shape1Vertices.push(verticesVec[i]);
		//} else {
		//	shape2Vertices.push(verticesVec[i]);
		//}
		b2Vec2 v = pshape->GetVertex(i);
		CCLog("Vertex %d: (%f, %f)", i, v.x, v.y);
    }

	shapeVerts.push_back(poly1);
	shapeVerts.push_back(poly2);

}