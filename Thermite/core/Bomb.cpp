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