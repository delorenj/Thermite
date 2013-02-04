#ifndef Thermite_SimpleBomb_h
#define Thermite_SimpleBomb_h

#include "Bomb.h"

class SimpleBomb : public Bomb {
    
public:
    SimpleBomb(float radius);
    ~SimpleBomb();
    
	const char* getName() { return "Simple Bomb"; }
	void generateBlastShape(float radius);

private:
    b2Vec2 getEdgeBreakPoint(b2Body*, b2Vec2, int, int);
    
    static const int m_maxRadius = 100;
};

#endif
