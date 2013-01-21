#include "Prototype.h"


Prototype::Prototype() {
	setTouchEnabled(true);
	CCSize size = CCDirector::sharedDirector()->getWinSize();

	m_bodyDef.type = b2_dynamicBody;
    m_fixtureDef.restitution = 0.4f;
    m_fixtureDef.friction = 0.2f;
    m_fixtureDef.density = 4;
    m_centerPoint = CCPointMake(0.5*size.width, 0.5*size.height);

	try {
		testSimple();
		testSeparator();
	}
	catch(exception e) {
		CCLog("Oops...%s", e.what());
	}

	scheduleUpdate();

}


Prototype::~Prototype() {

}

CCScene* Prototype::scene() {
    CCScene* scene = CCScene::create();
    CCLayer* layer = new Prototype();
    scene->addChild(layer,0);
    layer->release();
    return scene;
}

void Prototype::testSimple() {
	b2PolygonShape shape;
	m_bodyDef.position.Set(m_centerPoint.x/PTM_RATIO, m_centerPoint.y/PTM_RATIO);
	b2Body* body = getWorld()->CreateBody(&m_bodyDef);
	shape.SetAsBox(4, 4);
	m_fixtureDef.shape = &shape;
	body->CreateFixture(&m_fixtureDef);

	PhysicsSprite* ps = new PhysicsSprite();
	ps->setTag(1);
    ps->setPosition( CCPointMake( m_centerPoint.x, m_centerPoint.y ) );
    ps->setPhysicsBody(body);
	body->SetUserData(ps);
	m_sprites[ps->getTag()] = ps;

}

void Prototype::testSeparator() {
		b2Separator* sep = new b2Separator();
        vector<b2Vec2>* vec = new vector<b2Vec2>();
        vec->push_back(b2Vec2(-4, -4));
        vec->push_back(b2Vec2(4, -4));
        vec->push_back(b2Vec2(4, 0));
        vec->push_back(b2Vec2(0, 0));
        vec->push_back(b2Vec2(0, 4));
		vec->push_back(b2Vec2(-4, 4));

		m_bodyDef.position.Set((m_centerPoint.x-300)/PTM_RATIO, m_centerPoint.y/PTM_RATIO);
		b2Body* body = getWorld()->CreateBody(&m_bodyDef);

        sep->Separate(body, &m_fixtureDef, vec, PTM_RATIO);

		PhysicsSprite* ps = new PhysicsSprite();
		ps->setTag(2);
		ps->setPosition( CCPointMake( m_centerPoint.x-300, m_centerPoint.y ) );
		ps->setPhysicsBody(body);
		body->SetUserData(ps);
		m_sprites[ps->getTag()] = ps;

		vec->clear();
        vec->push_back(b2Vec2(0, 0));
        vec->push_back(b2Vec2(4, 0));
        vec->push_back(b2Vec2(4, 4));
        vec->push_back(b2Vec2(0, 4));
		m_bodyDef.position.Set((m_centerPoint.x-300+(4*PTM_RATIO))/PTM_RATIO, m_centerPoint.y/PTM_RATIO);
		body = getWorld()->CreateBody(&m_bodyDef);

        sep->Separate(body, &m_fixtureDef, vec, PTM_RATIO);

		ps = new PhysicsSprite();
		ps->setTag(3);
		ps->setPosition( CCPointMake( m_centerPoint.x-300+(4*PTM_RATIO), m_centerPoint.y ) );
		ps->setPhysicsBody(body);
		body->SetUserData(ps);
		m_sprites[ps->getTag()] = ps;

}



CCPoint Prototype::touchToPoint(CCTouch* pTouch) {
    return CCDirector::sharedDirector()->convertToGL(pTouch->getLocationInView());
}

void Prototype::ccTouchesBegan(CCSet *pTouches, CCEvent *pEvent) {
	PhysicsSprite* sprite;
    for (auto it = pTouches->begin(); it != pTouches->end(); it++) {
        CCTouch* touch = dynamic_cast<CCTouch*>(*it);
		CCPoint touchPoint = touchToPoint(touch);
		sprite = getPhysicsSpriteAtXY(touchPoint);
		if(sprite != NULL) {
			CCLog("Got Sprite: %d", sprite->getTag());
		}

    }
}

void Prototype::ccTouchesEnded(CCSet *pTouches, CCEvent *pEvent) {
    
}

void Prototype::ccTouchesMoved(CCSet *pTouches, CCEvent *pEvent) {
    
}