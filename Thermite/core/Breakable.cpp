#include "Breakable.h"

using namespace cocos2d;

Breakable::Breakable(CCBox2DLayer* ctx, float w, float h, float x, float y, bool structure)
{
    m_pCtx = ctx;

    b2BodyDef bodyDef;
    m_pPhysicsSprite = new PhysicsSprite();
    CCPoint p = CCPointMake(x, y);
    m_pPhysicsSprite->setPosition( CCPointMake( p.x, p.y ) );
    b2PolygonShape box;
    box.SetAsBox(w/PTM_RATIO/2, h/PTM_RATIO/2);

	b2FixtureDef fixtureDef;
    fixtureDef.shape = &box;

	if(structure) {
		bodyDef.type = b2_staticBody;
	} else {
		bodyDef.type = b2_dynamicBody;
		fixtureDef.density = 1.0f;
		fixtureDef.friction = 0.3f;
	}

    bodyDef.position.Set(p.x/PTM_RATIO, p.y/PTM_RATIO);
    
    b2Body* body = m_pCtx->getWorld()->CreateBody(&bodyDef);
    body->CreateFixture(&fixtureDef);
    body->SetUserData(m_pPhysicsSprite);
	m_pPhysicsSprite->setPhysicsBody(body);
	m_pPhysicsSprite->SetUserData(this);
	m_pCtx->addSprite(*m_pPhysicsSprite);
}

Breakable::Breakable(CCBox2DLayer* ctx, vector<b2Vec2>& shape, float x, float y, bool structure)
{
    m_pCtx = ctx;

	m_pHull = new NonConvexHull(shape);

    b2BodyDef bodyDef;
    m_pPhysicsSprite = new PhysicsSprite();
    CCPoint p = CCPointMake(x, y);
    m_pPhysicsSprite->setPosition( CCPointMake( p.x, p.y ) );
    b2FixtureDef fixtureDef;

	if(structure) {
		bodyDef.type = b2_staticBody;
	} else {
		bodyDef.type = b2_dynamicBody;
		fixtureDef.density = 1.0f;
		fixtureDef.friction = 0.3f;
	}

    bodyDef.position.Set(p.x/PTM_RATIO, p.y/PTM_RATIO);
    
    b2Body* body = m_pCtx->getWorld()->CreateBody(&bodyDef);
	b2Separator sep;
	sep.Separate(body, &fixtureDef, m_pHull->getVertices(), PTM_RATIO);
    body->SetUserData(m_pPhysicsSprite);
	m_pPhysicsSprite->setPhysicsBody(body);
	m_pPhysicsSprite->SetUserData(this);
	m_pCtx->addSprite(*m_pPhysicsSprite);
}

Breakable::~Breakable()
{
	delete m_pPhysicsSprite;
}


CCBox2DLayer* Breakable::getContext() const {
    return m_pCtx;
}

bool Breakable::isTouching(const cocos2d::CCPoint p) const{
    return m_pPhysicsSprite->boundingBox().containsPoint(p);
}

void Breakable::applyBomb(Bomb& bomb) {
	//	Breakable structure will split into 2 or more Breakables:
	//		1 Structure
	//		1+ Broken piece(s)
	//
	//	The outcome of the above result will depend on the number of crossover vertices between
	//	bomb and structure:
	//		2 crossovers = 1 structure, 1 broken piece
	//		4 crossovers = 1 structure, 2 broken pieces
	//		2^n crossovers = 1 structure, n broken pieces
	//
	// (above comment only applies to the current bomb types implemented. The above may change)

}
