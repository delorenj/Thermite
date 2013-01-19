//
//  BuildingBlock.cpp
//  Thermite
//
//  Created by Jarad Delorenzo on 12/10/12.
//
//

#include "BuildingBlock.h"
#define TEXTURE_SIZE 256

using namespace cocos2d;

BuildingBlock::BuildingBlock(CCBox2DLayer* ctx, float size, float x, float y) {
    m_pCtx = ctx;
    
     CCSpriteBatchNode *bn = CCSpriteBatchNode::create("square.png", TEXTURE_SIZE);
    m_pSpriteTexture = bn->getTexture();
    m_pCtx->addChild(bn, 3);

//	CCTexture2D::setDefaultAlphaPixelFormat(kCCTexture2DPixelFormat_RGBA4444);
//	CCSpriteBatchNode* spritesBgNode = CCSpriteBatchNode::batchNodeWithFile("Thermite.pvr.ccz");
//	m_pCtx->addChild(spritesBgNode);    
 
	PhysicsSprite::init();
    CCPoint p = CCPointMake(x, y);
    this->initWithTexture(m_pSpriteTexture, CCRectMake(0,0,size,size));
    this->autorelease();
    
    m_pCtx->addChild(this, 3);
    
    this->setPosition( CCPointMake( p.x, p.y ) );
    
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
    
    this->setPhysicsBody(body);
    
}

BuildingBlock::~BuildingBlock() {
    
}

CCBox2DLayer* BuildingBlock::getContext() {
    return m_pCtx;
}

bool BuildingBlock::isTouchingBlock(cocos2d::CCPoint p) {
    return this->boundingBox().containsPoint(p);
}

void BuildingBlock::applyBomb(cocos2d::CCPoint p, Bomb* bomb) {
    bomb->setPosition(b2Vec2(p.x/PTM_RATIO, p.y/PTM_RATIO));
    CCLog("Explosion at Point: (%f, %f)", p.x, p.y);
    CCLog("Bomb Type: %s", bomb->getName());
    CCLog("b2Vec2: (%f, %f)", bomb->getPosition().x, bomb->getPosition().y);
    b2World* world = m_pCtx->getWorld();
    b2Body* origBody = getPhysicsBody();
    m_subdivisions = bomb->subdivide(origBody);
    world->DestroyBody(origBody);
    m_pCtx->removeChild(this, false);
    
    for (auto it = m_subdivisions.begin(); it != m_subdivisions.end(); it++) {
        b2Body* body = dynamic_cast<b2Body*>(*it);
        PhysicsSprite* p = new PhysicsSprite();
        b2Vec2 b2Pos = body->GetPosition();
        p->setPosition(CCPointMake(b2Pos.x * PTM_RATIO, b2Pos.y * PTM_RATIO));
        p->setPhysicsBody(body);
    }
    
}
