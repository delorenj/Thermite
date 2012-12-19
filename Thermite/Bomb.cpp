//
//  Bomb.cpp
//  Thermite
//
//  Created by Jarad Delorenzo on 12/18/12.
//
//

#include "Bomb.h"

Bomb::Bomb() {

}

Bomb::~Bomb() {
    
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