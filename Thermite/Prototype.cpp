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

//	Only need below to attach box2d body to a cocos2d sprite...
	PhysicsSprite* ps = new PhysicsSprite();
	ps->setTag(1);
    ps->setPosition( CCPointMake( m_centerPoint.x+300, m_centerPoint.y ) );
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

	//	Only need below to attach box2d body to a cocos2d sprite...
	PhysicsSprite* ps = new PhysicsSprite();
	ps->setTag(2);
    ps->setPosition( CCPointMake( m_centerPoint.x+300, m_centerPoint.y ) );
    ps->setPhysicsBody(body);
	body->SetUserData(ps);
	m_sprites[ps->getTag()] = ps;
}

void Prototype::testBreakBody(b2Body* body, const CCPoint touchPoint, const float radius) {
	b2Separator* sep = new b2Separator();
	CCLog("Breaking Body: %d", static_cast<PhysicsSprite*>(body->GetUserData())->getTag());
	b2CircleShape bomb;
	CCPoint local = convertToNodeSpaceAR(touchPoint);
	bomb.m_radius = radius;
	bomb.m_p = b2Vec2(local.x/PTM_RATIO, local.y/PTM_RATIO);
	m_fixtureDef.shape = &bomb;
	body->CreateFixture(&m_fixtureDef);

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
			testBreakBody(sprite->getPhysicsBody(), touchPoint, 1.5f );
		}

    }
}

void Prototype::ccTouchesEnded(CCSet *pTouches, CCEvent *pEvent) {
    
}

void Prototype::ccTouchesMoved(CCSet *pTouches, CCEvent *pEvent) {
    
}