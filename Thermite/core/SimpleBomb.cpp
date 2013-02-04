#include "SimpleBomb.h"


SimpleBomb::SimpleBomb(float radius)
{
	m_radius = radius;
	generateBlastShape(radius);
}


SimpleBomb::~SimpleBomb(void)
{
}

void SimpleBomb::generateBlastShape(float radius) {
	int segments = 3;
	float roughness = 1;
	Bomb::generateBlastShape(radius, segments, roughness);
}