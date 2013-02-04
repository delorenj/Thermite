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
    body->SetUserData(this);
	m_pPhysicsSprite->setPhysicsBody(body);
	m_pCtx->addSprite(*m_pPhysicsSprite);
}

Breakable::Breakable(CCBox2DLayer* ctx, vector<b2Vec2>& shape, float x, float y, bool structure)
{
    m_pCtx = ctx;

	m_pHull = new forward_list<b2Vec2>();

	auto currVer = m_pHull->before_begin();
	for(vector<b2Vec2>::iterator it = shape.begin(); it != shape.end(); it++) {
		currVer = m_pHull->emplace_after(currVer, *it);
	}

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
	sep.Separate(body, &fixtureDef, *m_pHull, PTM_RATIO);
    body->SetUserData(this);
	m_pPhysicsSprite->setPhysicsBody(body);
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

void Breakable::applyBomb(cocos2d::CCPoint p, Bomb& bomb) {    
}
