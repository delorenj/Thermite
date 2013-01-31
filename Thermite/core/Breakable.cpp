#include "Breakable.h"

using namespace cocos2d;

Breakable::Breakable(CCBox2DLayer* ctx, const char* image, float x, float y)
{
    m_pCtx = ctx;
	int size = 256;

    m_pPhysicsSprite = new PhysicsSprite();

//  CCSpriteBatchNode* bn = CCSpriteBatchNode::create(image, 256);
//	bn->setTag(123);
//  m_pSpriteTexture = bn->getTexture();
//  m_pCtx->addChild(bn, 3);

//	CCTexture2D::setDefaultAlphaPixelFormat(kCCTexture2DPixelFormat_RGBA4444);
//	CCSpriteBatchNode* spritesBgNode = CCSpriteBatchNode::batchNodeWithFile("Thermite.pvr.ccz");
//	m_pCtx->addChild(spritesBgNode);    
 
    CCPoint p = CCPointMake(x, y);
    //m_pPhysicsSprite->initWithTexture(m_pSpriteTexture, CCRectMake(0,0,size,size));
    //m_pPhysicsSprite->autorelease();
    
    m_pCtx->addChild(m_pPhysicsSprite, 3);    
    m_pPhysicsSprite->setPosition( CCPointMake( p.x, p.y ) );
    
    // Define the dynamic body.
    //Set up a 1m squared box in the physics world
    b2BodyDef bodyDef;
    bodyDef.type = b2_dynamicBody;
    bodyDef.position.Set(p.x/PTM_RATIO, p.y/PTM_RATIO);
    
    b2Body* body = ctx->getWorld()->CreateBody(&bodyDef);
    
    // Define another box shape for our dynamic body.
    b2PolygonShape dynamicBox;
    dynamicBox.SetAsBox(size/PTM_RATIO/2, size/PTM_RATIO/2);//These are mid points for our 1m box
    
    // Define the dynamic body fixture.
    b2FixtureDef fixtureDef;
    fixtureDef.shape = &dynamicBox;
    fixtureDef.density = 1.0f;
    fixtureDef.friction = 0.3f;
    body->CreateFixture(&fixtureDef);
    body->SetUserData(this);
    m_pPhysicsSprite->setPhysicsBody(body);
    
}

Breakable::~Breakable()
{
}


CCBox2DLayer* Breakable::getContext() {
    return m_pCtx;
}

bool Breakable::isTouching(cocos2d::CCPoint p) {
    return m_pPhysicsSprite->boundingBox().containsPoint(p);
}

void Breakable::applyBomb(cocos2d::CCPoint p, Bomb* bomb) {
	b2Separator* sep = new b2Separator();
	vector<vector<b2Vec2>* >* shapeVerts = new vector<vector <b2Vec2>* >();
    b2World* world = m_pCtx->getWorld();
    b2Body* origBody = m_pPhysicsSprite->getPhysicsBody();
    b2FixtureDef fixtureDef;
	b2Vec2 origBodyPos = origBody->GetPosition();
	b2Body *body;
	b2BodyDef *bodyDef;
	PhysicsSprite* ps;

	fixtureDef.density = 1.0f;
    fixtureDef.friction = 0.3f;
    bomb->setPosition(b2Vec2(p.x/PTM_RATIO, p.y/PTM_RATIO));
    CCLog("Explosion at Point: (%f, %f)", p.x, p.y);
    CCLog("Bomb Type: %s", bomb->getName());
    CCLog("b2Vec2: (%f, %f)", bomb->getPosition().x, bomb->getPosition().y);

	bodyDef = new b2BodyDef();
	bodyDef->type = b2_dynamicBody;
	bodyDef->position.Set(origBodyPos.x, origBodyPos.y);

    bomb->subdivide(origBody, *shapeVerts);

	world->DestroyBody(origBody);
    m_pCtx->removeChild(m_pPhysicsSprite, false);

        vector<b2Vec2>* vec = new vector<b2Vec2>();
        vec->push_back(b2Vec2(-3, -3));
        vec->push_back(b2Vec2(3, -3));
        vec->push_back(b2Vec2(3, 0));
        vec->push_back(b2Vec2(0, 0));
        vec->push_back(b2Vec2(-3, 3));

		body = world->CreateBody(bodyDef);
		sep->Separate(body, &fixtureDef, vec, PTM_RATIO);
		m_pPhysicsSprite = new PhysicsSprite();
//		m_pPhysicsSprite->initWithTexture(m_pSpriteTexture, CCRectMake(0,0,256,256));
		m_pPhysicsSprite->autorelease();
		m_pPhysicsSprite->setPosition( CCPointMake( origBodyPos.x, origBodyPos.y ) );
		m_pPhysicsSprite->setPhysicsBody(body);
		m_pCtx->addChild(m_pPhysicsSprite, 3);

	/*for(int i=0; i<shapeVerts->size(); i++)
	{
		vector<b2Vec2>* v = shapeVerts->at(i);
		CCLog("Shape %d:", i+1);
		for(int j=0; j<v->size(); j++) {
			b2Vec2 x = v->at(j);
			CCLog("\tVert %d: (%f, %f)", j, x.x, x.y);
		}

        if(sep->Validate(*v)==0)
        {
            CCLog("Yay! Those vertices are good to go!");
        }
        else
        {
            CCLog("Oh, I guess you effed something up :(");
        }

		body = world->CreateBody(bodyDef);
		sep->Separate(body, &fixtureDef, v, PTM_RATIO);
		ps = new PhysicsSprite();
		ps->setPhysicsBody(body);
		m_pCtx->addChild(ps, 3);

	}*/
	

    
    //for (auto it = m_subdivisions.begin(); it != m_subdivisions.end(); it++) {
    //    b2Body* body = dynamic_cast<b2Body*>(*it);
    //    PhysicsSprite* p = new PhysicsSprite();
    //    b2Vec2 b2Pos = body->GetPosition();
    //    p->setPosition(CCPointMake(b2Pos.x * PTM_RATIO, b2Pos.y * PTM_RATIO));
    //    p->setPhysicsBody(body);
    //}
    
}
