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
	void subdivide(b2Body* body, vector<vector<b2Vec2>* > &shapeVerts);

private:
    b2Vec2 getEdgeBreakPoint(b2Body*, b2Vec2, int, int);
    
    static const int m_maxRadius = 100;
};

#endif
