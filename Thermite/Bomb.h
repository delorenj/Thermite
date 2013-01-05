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
#include "BuildingBlock.h"

class BuildingBlock;

class Bomb {

public:
    Bomb();
    ~Bomb();
    
    virtual const char* getName() = 0;
    virtual list<b2Body*> subdivide(b2Body*) = 0;

    int getRadius();
    int setRadius(int radius);
    int getMaxRadius();
    int getEnergy();
    b2Vec2 getPosition();
    void setPosition(b2Vec2 p);
    
protected:
    int Bomb::det(int x1, int y1, int x2, int y2, int x3, int y3); 
    b2Vec2 m_position;
    int m_radius;
    const int m_maxRadius = 50;
    const int m_energy = 1000;
};

#endif
