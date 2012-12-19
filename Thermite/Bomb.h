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
    
protected:
    int m_radius;
    const int m_maxRadius = 50;
    const int m_energy = 1000;
};

#endif
