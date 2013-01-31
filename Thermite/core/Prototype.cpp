#include "Prototype.h"


Prototype::Prototype() {
	setTouchEnabled(true);
	CCSize size = CCDirector::sharedDirector()->getWinSize();
	srand ( time(NULL) );
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
	try {
		sep.Separate(body, &m_fixtureDef, vec, PTM_RATIO);
	} catch(b2SeparatorException& e) {
		CCLog("b2Separator Exception: %s", e.what());
	}


	//	Only need below to attach box2d body to a cocos2d sprite...
	PhysicsSprite* ps = new PhysicsSprite();
	ps->setTag(2);
    ps->setPosition( CCPointMake( m_centerPoint.x+300, m_centerPoint.y ) );
    ps->setPhysicsBody(body);
	body->SetUserData(ps);
	m_sprites[ps->getTag()] = ps;
}

void Prototype::testPlaceBomb(b2Body* body, const CCPoint touchPoint, const float radius) {
	CCLog("Breaking Body: %d", static_cast<PhysicsSprite*>(body->GetUserData())->getTag());

	b2Separator sep;
	vector<b2Vec2>* bombShape;
	b2Vec2 worldPoint =  b2Vec2(touchPoint.x/PTM_RATIO, touchPoint.y/PTM_RATIO);
	b2Vec2 localPoint = body->GetLocalPoint(worldPoint);
	bool valid = true;

	do {
		do {
			 bombShape = generateBlastShape(radius, 20, 0.5f);
		} while (sep.Validate(*bombShape) != 0);

		try{
			sep.Validate(*bombShape);
		} catch(b2SeparatorException& e) {
			CCLog("b2Separator Exception: %s", e.what());
		}

		for(int i=0; i<bombShape->size(); i++) {
			bombShape->at(i).x += localPoint.x;
			bombShape->at(i).y += localPoint.y;
		}

		try {
			sep.Separate(body, &m_fixtureDef, bombShape, PTM_RATIO);
			valid = true;
		} catch(b2SeparatorException& e) {
			valid = false;
			CCLog("b2Separator Exception: %s", e.what());
			continue;
		}
	} while(!valid);

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

vector<b2Vec2>* Prototype::generateBlastShape(float radius, int segments, float roughness) {
    vector<b2Vec2>* vec = new vector<b2Vec2>();
	float delta = 2.0f*b2_pi / segments;
	float radius_threshold = radius * roughness;
	float theta = 0;
	for(int i=0; i<segments; i++, theta+=delta) {
		float x,y,r;
		r = radius + CCRANDOM_MINUS1_1()*radius_threshold;
		x = r*cos(theta);
		y = r*sin(theta);
		vec->push_back(b2Vec2(x, y));
	}
	return vec;
}

CCPoint Prototype::touchToPoint(CCTouch* pTouch) {
    return CCDirector::sharedDirector()->convertToGL(pTouch->getLocationInView());
}

void Prototype::ccTouchesBegan(CCSet *pTouches, CCEvent *pEvent) {
	PhysicsSprite* sprite;

    for (CCSetIterator it = pTouches->begin(); it != pTouches->end(); it++) {
        CCTouch* touch = dynamic_cast<CCTouch*>(*it);
		CCPoint touchPoint = touchToPoint(touch);
		sprite = getPhysicsSpriteAtXY(touchPoint);

		if(sprite != NULL) {
			testPlaceBomb(sprite->getPhysicsBody(), touchPoint, 1.75f );
		}

    }
}

void Prototype::ccTouchesEnded(CCSet *pTouches, CCEvent *pEvent) {
    
}

void Prototype::ccTouchesMoved(CCSet *pTouches, CCEvent *pEvent) {
    
}
