//
//  Sandbox.cpp
//  Thermite
//
//  Created by Jarad Delorenzo on 12/10/12.
//
//

#include "Sandbox.h"

using namespace cocos2d;

Sandbox::Sandbox() {
	this->setTouchEnabled(true);
    CCSize s = CCDirector::sharedDirector()->getWinSize();

	// init background
	CCTexture2D::setDefaultAlphaPixelFormat(kCCTexture2DPixelFormat_RGB565);
    CCSprite* background = CCSprite::create("bg.pvr.ccz", CCRectMake(0, 0, 1024, 768) );
    background->setAnchorPoint(ccp(0,0));
	this->addChild(background, 1);   


	try {
//		this->initBlocks();
//		this->initB2SeparatorExample();
		this->initBreakables();
	}
	catch(exception e) {
		CCLog("Oops");
	}
    
    scheduleUpdate();
}

Sandbox::~Sandbox() {
    	
}

CCScene* Sandbox::scene() {
    CCScene* scene = CCScene::create();
    CCLayer* layer = new Sandbox();
    scene->addChild(layer,0);
    layer->release();
    return scene;
}

void Sandbox::initBreakables() {
    CCSize s = CCDirector::sharedDirector()->getWinSize();

    m_pBreakables.push_front(new Breakable(this, "square.png", s.width/2, s.height/2));
}

void Sandbox::initB2SeparatorExample() {
        CCSize s = CCDirector::sharedDirector()->getWinSize();
        CCPoint p = CCPointMake(0.5*s.width, 0.5*s.height);

        b2Body *body;
        b2BodyDef *bodyDef = new b2BodyDef();
        b2FixtureDef *fixtureDef = new b2FixtureDef();
        b2Separator* sep = new b2Separator();

        bodyDef->type = b2_dynamicBody;
        bodyDef->position.Set(p.x/PTM_RATIO, p.y/PTM_RATIO);
        body = getWorld()->CreateBody(bodyDef);
        fixtureDef->restitution = 0.4f;
        fixtureDef->friction = 0.2f;
        fixtureDef->density = 4;
        
        vector<b2Vec2>* vec = new vector<b2Vec2>();
        vec->push_back(b2Vec2(-3, -3));
        vec->push_back(b2Vec2(3, -3));
        vec->push_back(b2Vec2(3, 0));
        vec->push_back(b2Vec2(0, 0));
        vec->push_back(b2Vec2(-3, 3));

        if(sep->Validate(*vec)==0)
        {
            CCLog("Yay! Those vertices are good to go!");
        }
        else
        {
            CCLog("Oh, I guess you effed something up :(");
        }
    
//        b2PolygonShape box;
//        box.SetAsBox(3, 3);
//        fixtureDef->shape = &box;
//        body->CreateFixture(fixtureDef);

        sep->Separate(body, fixtureDef, vec, PTM_RATIO);
        PhysicsSprite* ps = new PhysicsSprite();
        ps->setPosition( CCPointMake( p.x, p.y ) );
        ps->setPhysicsBody(body);

}

void Sandbox::initBlocks() {
    CCSize s = CCDirector::sharedDirector()->getWinSize();

    m_pBuildingBlocks.push_front(new BuildingBlock(this, 256, s.width/2, s.height/2));
    m_pBuildingBlocks.push_front(new BuildingBlock(this, 150, 100, 400));
}

CCPoint Sandbox::touchToPoint(CCTouch* pTouch) {
    return CCDirector::sharedDirector()->convertToGL(pTouch->getLocationInView());
}

void Sandbox::ccTouchesBegan(CCSet *pTouches, CCEvent *pEvent) {
/*
    for (auto it = pTouches->begin(); it != pTouches->end(); it++) {
        CCTouch* touch = dynamic_cast<CCTouch*>(*it);

        for(auto it = m_pBreakables.begin(); it != m_pBreakables.end(); it ++) {
            Breakable* b = dynamic_cast<Breakable*>(*it);
            if(b->isTouching(touchToPoint(touch))) {
                LegoBomb* bomb = new LegoBomb();
                b->applyBomb(touchToPoint(touch), bomb);
            }
        }
    }
*/
}

void Sandbox::ccTouchesEnded(CCSet *pTouches, CCEvent *pEvent) {
    
}

void Sandbox::ccTouchesMoved(CCSet *pTouches, CCEvent *pEvent) {
    
}
