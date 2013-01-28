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
	b2Separator sep;
    vector<b2Vec2>* vec = new vector<b2Vec2>();
    vec->push_back(b2Vec2(-4, -4));
    vec->push_back(b2Vec2(4, -4));
    vec->push_back(b2Vec2(4, 0));
    vec->push_back(b2Vec2(0, 0));
    vec->push_back(b2Vec2(0, 4));
	vec->push_back(b2Vec2(-4, 4));

	m_bodyDef.position.Set((m_centerPoint.x-300)/PTM_RATIO, m_centerPoint.y/PTM_RATIO);
	b2Body* body = getWorld()->CreateBody(&m_bodyDef);

    sep.Separate(body, &m_fixtureDef, vec, PTM_RATIO);

	//	Only need below to attach box2d body to a cocos2d sprite...
	PhysicsSprite* ps = new PhysicsSprite();
	ps->setTag(2);
    ps->setPosition( CCPointMake( m_centerPoint.x+300, m_centerPoint.y ) );
    ps->setPhysicsBody(body);
	body->SetUserData(ps);
	m_sprites[ps->getTag()] = ps;
}

void Prototype::testPlaceBomb(b2Body* body, const CCPoint touchPoint, const float radius) {
	b2Separator sep;
	CCLog("Breaking Body: %d", static_cast<PhysicsSprite*>(body->GetUserData())->getTag());
	vector<b2Vec2>* bombShape = generateBlastShape(radius);

	try{
		switch(sep.Validate(*bombShape)) {
		case 1:
			throw(new exception("Overlapping lines"));
			break;
		case 2:
			throw(new exception("Points are not in clockwise order"));
			break;
		case 3:
			throw(new exception("Overlapping lines AND points are not in clockwise order"));
			break;
		case 0: 
			CCLog("YAY?");
			break;
		}
	} catch(exception e) {
		CCLog("Illegal Bomb Blast: %s", e);
	}

	b2Vec2 worldPoint =  b2Vec2(touchPoint.x/PTM_RATIO, touchPoint.y/PTM_RATIO);
	b2Vec2 localPoint = body->GetLocalPoint(worldPoint);
	for(int i=0; i<bombShape->size(); i++) {
		bombShape->at(i).x += localPoint.x;
		bombShape->at(i).y += localPoint.y;
	}
	sep.Separate(body, &m_fixtureDef, bombShape, PTM_RATIO);
	//m_fixtureDef.shape = bombShape;
	//b2Fixture* bomb = body->CreateFixture(&m_fixtureDef);
	for(b2Fixture* fix = body->GetFixtureList(); fix; fix=fix->GetNext()) {
		CCLog("TESTING FIXTURE:");
		b2Shape::Type shapeType = fix->GetType();
		if(shapeType != b2Shape::e_polygon) {
			CCLog("Non-polygon encountered. Continuing...");
			continue;
		}
		b2PolygonShape* shape = (b2PolygonShape*)fix->GetShape();
		for(int i=0; i<shape->GetVertexCount(); i++) {
			b2Vec2 vert = shape->GetVertex(i);
			b2Vec2 wp = body->GetWorldPoint(vert);
//			bool pointIn = bomb->TestPoint(wp);
			CCLog("\tVertex %d: (%f, %f)", i, wp.x, wp.y); 
		}
	}	
}

vector<b2Vec2>* Prototype::generateBlastShape(float radius) {
    vector<b2Vec2>* vec = new vector<b2Vec2>();
	vec->push_back(b2Vec2(-0.9f, -0.5f));
	vec->push_back(b2Vec2(-1.7f, -1.35f));
	vec->push_back(b2Vec2(1.3f, -2.7f));
	vec->push_back(b2Vec2(1.1f, -1.2f));
	vec->push_back(b2Vec2(2.6f, 0));
	vec->push_back(b2Vec2(1.4f, 2.0f));
	vec->push_back(b2Vec2(0, 1.3f));
	vec->push_back(b2Vec2(-2.0f, 1.0f));
	return vec;
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
			testPlaceBomb(sprite->getPhysicsBody(), touchPoint, 1.5f );
		}

    }
}

void Prototype::ccTouchesEnded(CCSet *pTouches, CCEvent *pEvent) {
    
}

void Prototype::ccTouchesMoved(CCSet *pTouches, CCEvent *pEvent) {
    
}