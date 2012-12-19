//
//  LegoBomb.cpp
//  Thermite
//
//  Created by Jarad Delorenzo on 12/18/12.
//
//

#include "LegoBomb.h"

LegoBomb::LegoBomb() {
    
}

LegoBomb::~LegoBomb() {
    
}

const char* LegoBomb::getName() {
    return "Lego Bomb";
}

list<b2Body*> LegoBomb::subdivide(b2Body* body) {
    
    return list<b2Body*>();
}