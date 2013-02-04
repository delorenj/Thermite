//
//  Bomb.h
//  Thermite
//
//  Created by Jarad Delorenzo on 12/18/12.
//
//

#ifndef Thermite_Bomb_h
#define Thermite_Bomb_h

#include "cocos2d.h"
#include "Box2D.h"
#include "Bomb.h"
#include "NonConvexHull.h"

class Bomb {

public:
    Bomb();
    ~Bomb();
    
    virtual const char* getName() = 0;
    int getRadius();
    int setRadius(int radius);
    int getMaxRadius();
    int getEnergy();
    b2Vec2 getPosition();
    void setPosition(b2Vec2 p);
    
protected:
    int det(int x1, int y1, int x2, int y2, int x3, int y3); 
	virtual void generateBlastShape(float radius, int segments, float roughness);
	b2Vec2 getCrossoverVertex(const b2Fixture& fixture, const b2Vec2& p1, const b2Vec2& p2);

	b2FixtureDef m_fixtureDef;
	b2BodyDef m_bodyDef;
    b2Vec2 m_position;
    int m_radius;
	NonConvexHull* m_pForwardHull;
	NonConvexHull* m_pReverseHull;
    static const int m_maxRadius = 50;
    static const int m_energy = 1000;
};

#endif
