//
//  LegoBomb.h
//  Thermite
//
//  Created by Jarad Delorenzo on 12/18/12.
//
//

#ifndef Thermite_LegoBomb_h
#define Thermite_LegoBomb_h

#include "Bomb.h"

class LegoBomb : public Bomb {
    
public:
    LegoBomb();
    ~LegoBomb();
    
    const char* getName();
    list<b2Body*> subdivide(b2Body* block);
    
private:
    b2Vec2 getEdgeBreakPoint(b2Body*, b2Vec2, int, int);
    
    const int m_maxRadius = 100;
};

#endif
