//
//  Bomb.cpp
//  Thermite
//
//  Created by Jarad Delorenzo on 12/18/12.
//
//

#include "Bomb.h"

Bomb::Bomb() {
	m_bodyDef.type = b2_dynamicBody;
	m_fixtureDef.isSensor = true;
    m_fixtureDef.restitution = 0.4f;
    m_fixtureDef.friction = 0.2f;
    m_fixtureDef.density = 4;

}

Bomb::~Bomb() {
    if(m_pForwardHull) {
		delete m_pForwardHull;
	}

	if(m_pReverseHull) {
		delete m_pReverseHull;
	}
}

void Bomb::generateBlastShape(float radius, int segments, float roughness) {
    vector<b2Vec2>* vec = new vector<b2Vec2>();
	float delta = 2.0f*b2_pi / segments;
	float radius_threshold = radius * roughness;
	float theta = 0;
	for(int i=0; i<segments; i++, theta+=delta) {
		float x,y,r;
		r = radius + CCRANDOM_MINUS1_1()*radius_threshold;
		x = r*cos(theta);
		y = r*sin(theta);
		vec->push_back(b2Vec2(x, y));
	}
	
	m_pForwardHull = new NonConvexHull(*vec);
	reverse(vec->begin(), vec->end());
	m_pReverseHull = new NonConvexHull(*vec);

}

b2Vec2 Bomb::getCrossoverVertex(const b2Fixture& fixture, const b2Vec2& p1, const b2Vec2& p2) {
    b2RayCastInput input;
    input.p1 = p1;
    input.p2 = p2;
    input.maxFraction = 1;
    float closestFraction = 1;
    bool intersected = false;
    b2RayCastOutput output;

    if (!fixture.RayCast(&output, input, 0)) {
       cocos2d::CCLog("No intersection found...This should not have happened.");
	   throw exception();
	}

    if (closestFraction > output.fraction)
        closestFraction = output.fraction; 

    b2Vec2 hitPoint = input.p1 + closestFraction * (input.p2 - input.p1);
    return hitPoint;
}


int Bomb::getRadius() {
    return m_radius;
}

int Bomb::setRadius(int radius) {
    if(radius > m_maxRadius) {
        m_radius = m_maxRadius;
    } else if(radius < 1) {
        m_radius = 0;
    } else {
        m_radius = radius;
    }

    return m_radius;
}

int Bomb::getMaxRadius() {
    return m_maxRadius;
}

int Bomb::getEnergy() {
    return m_energy;
}

b2Vec2 Bomb::getPosition() {
    return m_position;
}

void Bomb::setPosition(b2Vec2 p) {
    m_position = p;
}

int Bomb::det(int x1, int y1, int x2, int y2, int x3, int y3) {
    // This is a function which finds the determinant of a 3x3 matrix.
    // If you studied matrices, you'd know that it returns a positive number if three given points are in clockwise order, negative if they are in anti-clockwise order and zero if they lie on the same line.
    // Another useful thing about determinants is that their absolute value is two times the face of the triangle, formed by the three given points.
    return x1*y2+x2*y3+x3*y1-y1*x2-y2*x3-y3*x1;
}