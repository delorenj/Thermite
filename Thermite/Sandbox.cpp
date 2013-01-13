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
	this->colorLayer = new CCLayerColor;
	this->colorLayer->initWithColor( ccc4(180, 180, 180, 255) );
//	this->addChild(this->colorLayer, 1);
    CCSize s = CCDirector::sharedDirector()->getWinSize();
        
    CCLabelTTF *label = CCLabelTTF::create("Sandbox Mode", "Marker Felt", 32);
    this->addChild(label, 2);
    label->setColor(ccc3(0,0,255));
    label->setPosition(ccp( s.width/2, s.height-50));
    
    //this->initBlocks();
    this->initB2SeparatorExample();
    
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
    for (auto it = pTouches->begin(); it != pTouches->end(); it++) {
        CCTouch* touch = dynamic_cast<CCTouch*>(*it);

        for(auto it = m_pBuildingBlocks.begin(); it != m_pBuildingBlocks.end(); it ++) {
            BuildingBlock* b = dynamic_cast<BuildingBlock*>(*it);
            if(b->isTouchingBlock(touchToPoint(touch))) {
                LegoBomb* bomb = new LegoBomb();
                b->applyBomb(touchToPoint(touch), bomb);
            }
        }
    }
}

void Sandbox::ccTouchesEnded(CCSet *pTouches, CCEvent *pEvent) {
    
}

void Sandbox::ccTouchesMoved(CCSet *pTouches, CCEvent *pEvent) {
    
}