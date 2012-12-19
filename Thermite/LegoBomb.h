//
//  LegoBomb.h
//  Thermite
//
//  Created by Jarad Delorenzo on 12/18/12.
//
//

#ifndef Thermite_LegoBomb_h
#define Thermite_LegoBomb_h

#include "cocos2d.h"
#include "Box2D.h"
#include "Bomb.h"

class LegoBomb : public Bomb {
    
public:
    LegoBomb();
    ~LegoBomb();
    
    const char* getName();
    list<b2Body*> subdivide(b2Body* body);
    
private:
    const int m_maxRadius = 100;
};

#endif
