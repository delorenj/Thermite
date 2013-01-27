#pragma once

#include "cocos2d.h"
#include "CCBox2DLayer.h"
#include "PhysicsSprite.h"

using namespace cocos2d;

class Prototype : public CCBox2DLayer
{
public:
	Prototype();
	~Prototype();

	CCPoint touchToPoint(CCTouch* pTouch);
    static CCScene* scene();

    void ccTouchesBegan(CCSet* pTouches, CCEvent* pEvent);
    void ccTouchesMoved(CCSet* pTouches, CCEvent* pEvent);
    void ccTouchesEnded(CCSet* pTouches, CCEvent* pEvent);

private:
	b2BodyDef m_bodyDef;
	b2FixtureDef m_fixtureDef;
	CCPoint m_centerPoint;

	void testSimple();
	void testSeparator();
	void testPlaceBomb(b2Body* body, const CCPoint touchPoint, const float radius);
	
	vector<b2Vec2>* generateBlastShape(float radius);
};


