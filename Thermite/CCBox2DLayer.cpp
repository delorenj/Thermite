//
//  CCBox2DLayer.cpp
//  Thermite
//
//  Created by Jarad Delorenzo on 12/14/12.
//
//

#include "CCBox2DLayer.h"

using namespace cocos2d;

CCBox2DLayer::CCBox2DLayer() {
    m_pWorld = initWorld();
}

CCBox2DLayer::~CCBox2DLayer() {

}

b2World* CCBox2DLayer::getWorld() {
    return m_pWorld;
}

b2World* CCBox2DLayer::initWorld() {
    CCSize winSize = CCDirector::sharedDirector()->getWinSize();
    
    // Create a world
    b2Vec2 gravity = b2Vec2(0.0f, -10.0f);
    m_pWorld = new b2World(gravity);
    m_pWorld->SetAllowSleeping(true);
    
    m_pDebugDraw = new b2DebugDraw( PTM_RATIO );
    m_pWorld->SetDebugDraw(m_pDebugDraw);
    
    uint32 flags = 0;
    flags += b2Draw::e_shapeBit;
//            flags += b2Draw::e_jointBit;
//            flags += b2Draw::e_aabbBit;
//            flags += b2Draw::e_pairBit;
//            flags += b2Draw::e_centerOfMassBit;
    m_pDebugDraw->SetFlags(flags);
    
    
    // Define the ground body.
    b2BodyDef groundBodyDef;
    groundBodyDef.position.Set(0, 0); // bottom-left corner
    
    // Call the body factory which allocates memory for the ground body
    // from a pool and creates the ground box shape (also from a pool).
    // The body is also added to the world.
    b2Body* groundBody = m_pWorld->CreateBody(&groundBodyDef);
    
    // Define the ground box shape.
    b2EdgeShape groundBox;
    
    // bottom
    
    groundBox.Set(b2Vec2(0,0), b2Vec2(winSize.width/PTM_RATIO,0));
    groundBody->CreateFixture(&groundBox,0);
    
//    // top
//    groundBox.Set(b2Vec2(0,winSize.height/PTM_RATIO), b2Vec2(winSize.width/PTM_RATIO,winSize.height/PTM_RATIO));
//    groundBody->CreateFixture(&groundBox,0);
//    
//    // left
//    groundBox.Set(b2Vec2(0,winSize.height/PTM_RATIO), b2Vec2(0,0));
//    groundBody->CreateFixture(&groundBox,0);
//    
//    // right
//    groundBox.Set(b2Vec2(winSize.width/PTM_RATIO,winSize.height/PTM_RATIO), b2Vec2(winSize.width/PTM_RATIO,0));
//    groundBody->CreateFixture(&groundBox,0);
    
    return m_pWorld;
}

void CCBox2DLayer::update(float dt) {
    int velocityIterations = 8;
    int positionIterations = 1;
    
    // Instruct the world to perform a single step of simulation. It is
    // generally best to keep the time step and iterations fixed.
    m_pWorld->Step(dt, velocityIterations, positionIterations);
    
}

void CCBox2DLayer::draw() {
    //
    // IMPORTANT:
    // This is only for debug purposes
    // It is recommend to disable it
    //
    ccGLEnableVertexAttribs( kCCVertexAttribFlag_Position );
    kmGLPushMatrix();
    m_pWorld->DrawDebugData();
    kmGLPopMatrix();
}

