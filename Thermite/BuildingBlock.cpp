//
//  BuildingBlock.cpp
//  Thermite
//
//  Created by Jarad Delorenzo on 12/10/12.
//
//

#include "BuildingBlock.h"

using namespace cocos2d;

BuildingBlock::BuildingBlock(CCBox2DLayer* ctx, float size, float x, float y) {
    m_pCtx = ctx;
    
    CCSpriteBatchNode *bn = CCSpriteBatchNode::create("square.png", size);
    m_pSpriteTexture = bn->getTexture();
    m_pCtx->addChild(bn, 3);
    
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
    
    this->setPhysicsBody(body);
    
}

BuildingBlock::~BuildingBlock() {
    
}

bool BuildingBlock::init(b2World* world) {
    return false;
}

bool BuildingBlock::isTouchingBlock(cocos2d::CCPoint p) {
    if(this->boundingBox().containsPoint(p)) {
            return true;
    }
    return false;
}

