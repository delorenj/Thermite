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
	this->addChild(this->colorLayer, 1);
    
    CCSize s = CCDirector::sharedDirector()->getWinSize();
        
    CCLabelTTF *label = CCLabelTTF::create("Sandbox Mode", "Marker Felt", 32);
    this->addChild(label, 2);
    label->setColor(ccc3(0,0,255));
    label->setPosition(ccp( s.width/2, s.height-50));
    
    this->initBlocks();
    
    scheduleUpdate();
}

Sandbox::~Sandbox() {
    	
}

CCScene* Sandbox::scene() {
    CCScene* scene = CCScene::create();
    CCLayer* layer = new Sandbox();
    scene->addChild(layer);
    layer->release();
    return scene;
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
                CCLog("Touching Block!");
            }
        }
    }
}

void Sandbox::ccTouchesEnded(CCSet *pTouches, CCEvent *pEvent) {
    
}

void Sandbox::ccTouchesMoved(CCSet *pTouches, CCEvent *pEvent) {
    
}